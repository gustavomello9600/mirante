mod analysis;
mod api;
mod catalog;
mod cluster;
mod fetch;
mod fulltext;
mod jobs;
mod llm;
mod model;
mod report;
mod store;
mod text;

use std::sync::Arc;

use anyhow::{Context, Result};
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

use crate::api::AppState;
use crate::catalog::Catalog;
use crate::fetch::Fetcher;
use crate::llm::LlmConfig;
use crate::store::Store;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .compact()
        .init();

    let port: u16 = std::env::var("MIRANTE_PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(8091);
    let dist = std::env::var("MIRANTE_DIST").unwrap_or_else(|_| "./dist".to_string());
    let db_path = std::env::var("MIRANTE_DB").unwrap_or_else(|_| "./data/mirante.db".to_string());
    let cache_ttl: i64 = std::env::var("MIRANTE_CACHE_TTL")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(1800);

    let store = Store::open(&db_path).await.context("inicializar store")?;
    let catalog = Arc::new(Catalog::load());
    let fetcher = Fetcher::new().context("inicializar fetcher")?;
    let llm = LlmConfig::from_env();

    let state = AppState {
        store,
        catalog,
        fetcher,
        llm: llm.clone(),
        cache_ttl,
        semaphore: Arc::new(tokio::sync::Semaphore::new(3)),
        jobs: crate::jobs::JobStore::new(),
        dist,
    };

    let app = api::router(state);
    let listener = TcpListener::bind(("0.0.0.0", port))
        .await
        .with_context(|| format!("bind 0.0.0.0:{port}"))?;

    tracing::info!(
        port,
        "mirante no ar · llm={}",
        llm.as_ref()
            .map(|config| config.label())
            .unwrap_or_else(|| "extractive".to_string())
    );

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("servidor axum")?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("instalar handler de Ctrl+C");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("instalar handler de SIGTERM")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
