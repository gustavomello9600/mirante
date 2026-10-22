use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

use crate::analysis;
use crate::catalog::Catalog;
use crate::fetch::Fetcher;
use crate::jobs::JobStore;
use crate::llm::LlmConfig;
use crate::model::{Analysis, ApiError, Health, ProgressFn, ProgressUpdate, Topic};
use crate::store::Store;
use crate::text::{hash_id, normalize_query};

#[derive(Clone)]
pub struct AppState {
    pub store: Store,
    pub catalog: Arc<Catalog>,
    pub fetcher: Fetcher,
    pub llm: Option<LlmConfig>,
    pub cache_ttl: i64,
    pub semaphore: Arc<tokio::sync::Semaphore>,
    pub jobs: JobStore,
    pub dist: String,
}

#[derive(Debug)]
pub struct AppError {
    status: StatusCode,
    message: String,
    detail: Option<String>,
}

impl AppError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
            detail: None,
        }
    }

    fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.into(),
            detail: None,
        }
    }

    fn internal(error: anyhow::Error) -> Self {
        tracing::error!(%error, "erro interno");
        // Detalhes podem conter URLs, caminhos e respostas de terceiros.
        // Ficam no log do servidor, salvo quando MIRANTE_DEBUG=true.
        let detail = if debug_enabled() {
            Some(error.to_string())
        } else {
            None
        };
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Não foi possível concluir a análise agora.".to_string(),
            detail,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ApiError {
                error: self.message,
                detail: self.detail,
            }),
        )
            .into_response()
    }
}

fn debug_enabled() -> bool {
    std::env::var("MIRANTE_DEBUG")
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

pub fn router(state: AppState) -> Router {
    let api = Router::new()
        .route("/health", get(health))
        .route("/search", get(search))
        .route("/jobs", post(create_job))
        .route("/jobs/{id}", get(job_status))
        .route("/topics", get(topics))
        .route("/sources", get(sources));

    let index = format!("{}/index.html", state.dist);
    let files = ServeDir::new(&state.dist).fallback(ServeFile::new(index));

    Router::new()
        .nest("/api", api)
        .fallback_service(files)
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health(State(state): State<AppState>) -> Result<Json<Health>, AppError> {
    let analyses = state.store.count().await.map_err(AppError::internal)?;
    Ok(Json(Health {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        analyses,
        sources: state.catalog.len(),
        llm: state
            .llm
            .as_ref()
            .map(|config| config.label())
            .unwrap_or_else(|| "extractive".to_string()),
    }))
}

#[derive(Debug, Deserialize)]
struct SearchParams {
    q: Option<String>,
    window: Option<u32>,
    refresh: Option<bool>,
}

async fn search(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Analysis>, AppError> {
    let raw_query = params.q.unwrap_or_default();
    let query = normalize_query(&raw_query);
    if query.len() < 2 {
        return Err(AppError::bad_request(
            "Digite um tema com pelo menos 2 caracteres.",
        ));
    }
    if query.len() > 180 {
        return Err(AppError::bad_request(
            "O tema é longo demais. Use até 180 caracteres.",
        ));
    }
    let window = params.window.unwrap_or(7).clamp(1, 30);
    let refresh = params.refresh.unwrap_or(false);
    let llm_label = state
        .llm
        .as_ref()
        .map(|config| config.label())
        .unwrap_or_else(|| "extractive".to_string());
    let key = hash_id(&format!("{}|{}|{}", query, window, llm_label));

    if !refresh {
        if let Ok(Some(payload)) = state.store.get(&key, state.cache_ttl).await {
            if let Ok(mut cached) = serde_json::from_str::<Analysis>(&payload) {
                cached.cached = true;
                return Ok(Json(cached));
            }
        }
    }

    let _permit = state
        .semaphore
        .acquire()
        .await
        .map_err(|_| AppError::internal(anyhow::anyhow!("semáforo fechado")))?;

    let raw = state
        .fetcher
        .fetch(&query, window)
        .await
        .map_err(AppError::internal)?;

    let mut result = analysis::build(
        &query,
        window,
        raw,
        &state.catalog,
        state.llm.as_ref(),
        None,
    )
    .await;

    if let Ok(payload) = serde_json::to_string(&result) {
        if let Err(error) = state.store.put(&key, &query, &payload).await {
            tracing::warn!(%error, "não foi possível gravar o cache");
        }
    }

    result.cached = false;
    Ok(Json(result))
}

#[derive(Debug, Deserialize)]
struct StartJob {
    q: Option<String>,
    query: Option<String>,
    window: Option<u32>,
    refresh: Option<bool>,
}

async fn create_job(
    State(state): State<AppState>,
    Json(body): Json<StartJob>,
) -> Result<Json<crate::jobs::JobView>, AppError> {
    let raw_query = body.q.or(body.query).unwrap_or_default();
    let query = normalize_query(&raw_query);
    if query.len() < 2 {
        return Err(AppError::bad_request(
            "Digite um tema com pelo menos 2 caracteres.",
        ));
    }
    if query.len() > 180 {
        return Err(AppError::bad_request(
            "O tema é longo demais. Use até 180 caracteres.",
        ));
    }
    let window = body.window.unwrap_or(7).clamp(1, 30);
    let refresh = body.refresh.unwrap_or(false);
    let id = uuid::Uuid::new_v4().to_string();
    state.jobs.create(&id, &query);
    let state_clone = state.clone();
    let job_id = id.clone();
    let query_clone = query.clone();
    tokio::spawn(async move {
        run_job(state_clone, job_id, query_clone, window, refresh).await;
    });
    state
        .jobs
        .get(&id)
        .map(Json)
        .ok_or_else(|| AppError::internal(anyhow::anyhow!("job não criado")))
}

async fn job_status(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<crate::jobs::JobView>, AppError> {
    state
        .jobs
        .get(&id)
        .map(Json)
        .ok_or_else(|| AppError::not_found("Análise não encontrada ou expirada."))
}

async fn run_job(state: AppState, id: String, query: String, window: u32, refresh: bool) {
    if let Err(error) = run_job_inner(&state, &id, &query, window, refresh).await {
        tracing::error!(%error, %query, "job falhou");
        let message = if debug_enabled() {
            error.to_string()
        } else {
            "A análise falhou no servidor. Tente novamente em instantes.".to_string()
        };
        state.jobs.fail(&id, message);
    }
}

async fn run_job_inner(
    state: &AppState,
    id: &str,
    query: &str,
    window: u32,
    refresh: bool,
) -> anyhow::Result<()> {
    let llm_label = state
        .llm
        .as_ref()
        .map(|config| config.label())
        .unwrap_or_else(|| "extractive".to_string());
    let key = hash_id(&format!("{}|{}|{}", query, window, llm_label));

    if !refresh {
        if let Some(payload) = state.store.get(&key, state.cache_ttl).await? {
            if let Ok(mut analysis) = serde_json::from_str::<Analysis>(&payload) {
                analysis.cached = true;
                state.jobs.complete(id, analysis);
                return Ok(());
            }
        }
    }

    let _permit = state.semaphore.acquire().await?;
    state.jobs.update(
        id,
        ProgressUpdate {
            step: 1,
            label: "Buscando manchetes".to_string(),
            detail: "Consultando Google News e veículos avaliados".to_string(),
            progress: 8.0,
            current: None,
            total: None,
        },
    );

    let raw = state.fetcher.fetch(query, window).await?;

    state.jobs.update(
        id,
        ProgressUpdate {
            step: 2,
            label: "Avaliando as fontes".to_string(),
            detail: format!("{} manchetes encontradas", raw.len()),
            progress: 22.0,
            current: Some(raw.len()),
            total: None,
        },
    );

    let jobs = state.jobs.clone();
    let job_id = id.to_string();
    let progress: ProgressFn = Arc::new(move |update: ProgressUpdate| {
        jobs.update(&job_id, update);
    });

    let mut analysis = analysis::build(
        query,
        window,
        raw,
        &state.catalog,
        state.llm.as_ref(),
        Some(progress),
    )
    .await;

    if let Ok(payload) = serde_json::to_string(&analysis) {
        if let Err(error) = state.store.put(&key, query, &payload).await {
            tracing::warn!(%error, "não foi possível gravar o cache");
        }
    }

    analysis.cached = false;
    state.jobs.complete(id, analysis);
    Ok(())
}

async fn topics() -> Json<Vec<Topic>> {
    Json(vec![
        Topic {
            label: "Reforma tributária".to_string(),
            query: "reforma tributária".to_string(),
            description: "O que muda, quem paga e o que ficou fora do texto.".to_string(),
        },
        Topic {
            label: "Regulação da IA".to_string(),
            query: "regulação da inteligência artificial".to_string(),
            description: "Direitos autorais, supervisão e a corrida por infraestrutura."
                .to_string(),
        },
        Topic {
            label: "Eleições 2026".to_string(),
            query: "eleições 2026".to_string(),
            description: "Pesquisas, alianças e a agenda que cada campo escolhe pautar."
                .to_string(),
        },
        Topic {
            label: "Tarifa de energia".to_string(),
            query: "tarifa de energia".to_string(),
            description: "Reajuste, transmissão e o impacto na conta de quem menos consome."
                .to_string(),
        },
        Topic {
            label: "Data centers".to_string(),
            query: "data centers".to_string(),
            description: "Energia, água e território na expansão da infraestrutura digital."
                .to_string(),
        },
        Topic {
            label: "Segurança pública".to_string(),
            query: "segurança pública".to_string(),
            description: "Operações, letalidade e o debate sobre política de drogas.".to_string(),
        },
        Topic {
            label: "Clima extremo".to_string(),
            query: "mudança climática eventos extremos".to_string(),
            description: "Enchentes, calor e adaptação nas cidades.".to_string(),
        },
        Topic {
            label: "Mercado de trabalho".to_string(),
            query: "mercado de trabalho emprego".to_string(),
            description: "Informalidade, renda e as vagas que a cobertura não mostra.".to_string(),
        },
    ])
}

async fn sources(State(state): State<AppState>) -> Json<Vec<crate::model::SourceInfo>> {
    Json(state.catalog.sources())
}
