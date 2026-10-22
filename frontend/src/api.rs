use gloo_net::http::Request;
use wasm_bindgen::JsValue;

use crate::model::{ApiErrorBody, JobView, SourceInfo, Topic};

fn api_base() -> String {
    js_sys::Reflect::get(&js_sys::global(), &JsValue::from_str("MIRANTE_API_BASE"))
        .ok()
        .and_then(|value| value.as_string())
        .unwrap_or_default()
        .trim_end_matches('/')
        .to_string()
}

fn api_url(path: &str) -> String {
    format!("{}{}", api_base(), path)
}

async fn parse_error(response: gloo_net::http::Response) -> String {
    let status = response.status();
    match response.json::<ApiErrorBody>().await {
        Ok(body) => body.error,
        Err(_) => format!("A análise falhou (HTTP {status}). Tente novamente."),
    }
}

pub async fn start_job(query: &str, window: u32) -> Result<JobView, String> {
    let payload = serde_json::json!({
        "q": query,
        "window": window,
        "refresh": false
    });
    let response = Request::post(&api_url("/api/jobs"))
        .header("content-type", "application/json")
        .body(payload.to_string())
        .map_err(|error| format!("Não foi possível preparar a busca: {error}"))?
        .send()
        .await
        .map_err(|error| format!("Não foi possível falar com o servidor: {error}"))?;
    if !response.ok() {
        return Err(parse_error(response).await);
    }
    response
        .json::<JobView>()
        .await
        .map_err(|error| format!("Resposta inválida do servidor: {error}"))
}

pub async fn get_job(id: &str) -> Result<JobView, String> {
    let response = Request::get(&api_url(&format!("/api/jobs/{id}")))
        .send()
        .await
        .map_err(|error| format!("Não foi possível acompanhar a análise: {error}"))?;
    if !response.ok() {
        return Err(parse_error(response).await);
    }
    response
        .json::<JobView>()
        .await
        .map_err(|error| format!("Resposta inválida do servidor: {error}"))
}

pub async fn topics() -> Result<Vec<Topic>, String> {
    let response = Request::get(&api_url("/api/topics"))
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !response.ok() {
        return Err(parse_error(response).await);
    }
    response
        .json::<Vec<Topic>>()
        .await
        .map_err(|error| error.to_string())
}

pub async fn sources() -> Result<Vec<SourceInfo>, String> {
    let response = Request::get(&api_url("/api/sources"))
        .send()
        .await
        .map_err(|error| error.to_string())?;
    if !response.ok() {
        return Err(parse_error(response).await);
    }
    response
        .json::<Vec<SourceInfo>>()
        .await
        .map_err(|error| error.to_string())
}
