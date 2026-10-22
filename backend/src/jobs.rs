use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use chrono::Utc;
use serde::Serialize;

use crate::model::{Analysis, ProgressUpdate};

pub const STEPS_TOTAL: usize = 6;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobView {
    pub id: String,
    pub query: String,
    pub status: String,
    pub step: usize,
    pub steps_total: usize,
    pub progress: f64,
    pub label: String,
    pub detail: String,
    pub current: Option<usize>,
    pub total: Option<usize>,
    pub eta_seconds: Option<u64>,
    pub started_at: String,
    pub analysis: Option<Analysis>,
    pub error: Option<String>,
}

struct JobRecord {
    view: JobView,
    started: Instant,
    finished: Option<Instant>,
}

#[derive(Clone, Default)]
pub struct JobStore {
    inner: Arc<Mutex<HashMap<String, JobRecord>>>,
}

impl JobStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create(&self, id: &str, query: &str) {
        let mut guard = self.inner.lock().unwrap();
        guard.retain(|_, job| {
            job.finished
                .map(|finished| finished.elapsed().as_secs() < 900)
                .unwrap_or_else(|| job.started.elapsed().as_secs() < 1800)
        });
        guard.insert(
            id.to_string(),
            JobRecord {
                view: JobView {
                    id: id.to_string(),
                    query: query.to_string(),
                    status: "queued".to_string(),
                    step: 0,
                    steps_total: STEPS_TOTAL,
                    progress: 0.0,
                    label: "Preparando".to_string(),
                    detail: "Organizando a busca".to_string(),
                    current: None,
                    total: None,
                    eta_seconds: Some(30),
                    started_at: Utc::now().to_rfc3339(),
                    analysis: None,
                    error: None,
                },
                started: Instant::now(),
                finished: None,
            },
        );
    }

    pub fn update(&self, id: &str, update: ProgressUpdate) {
        let mut guard = self.inner.lock().unwrap();
        let Some(job) = guard.get_mut(id) else {
            return;
        };
        let progress = update.progress.clamp(0.0, 99.0);
        // Estimativa estável: 30 s para o fluxo completo, ajustada ao progresso.
        // O ETA real de leitura já chega pelo current/total da etapa 4.
        let eta = Some(((100.0 - progress) / 100.0 * 30.0).ceil().clamp(2.0, 120.0) as u64);
        job.view.status = "running".to_string();
        job.view.step = update.step;
        job.view.progress = progress;
        job.view.label = update.label;
        job.view.detail = update.detail;
        job.view.current = update.current;
        job.view.total = update.total;
        job.view.eta_seconds = eta;
    }

    pub fn complete(&self, id: &str, analysis: Analysis) {
        let mut guard = self.inner.lock().unwrap();
        if let Some(job) = guard.get_mut(id) {
            job.view.status = "done".to_string();
            job.view.step = STEPS_TOTAL;
            job.view.progress = 100.0;
            job.view.label = "Relance pronto".to_string();
            job.view.detail = "Citações e números conferidos".to_string();
            job.view.current = None;
            job.view.total = None;
            job.view.eta_seconds = Some(0);
            job.view.analysis = Some(analysis);
            job.view.error = None;
            job.finished = Some(Instant::now());
        }
    }

    pub fn fail(&self, id: &str, message: String) {
        let mut guard = self.inner.lock().unwrap();
        if let Some(job) = guard.get_mut(id) {
            job.view.status = "error".to_string();
            job.view.error = Some(message);
            job.view.eta_seconds = None;
            job.finished = Some(Instant::now());
        }
    }

    pub fn get(&self, id: &str) -> Option<JobView> {
        let guard = self.inner.lock().unwrap();
        guard.get(id).map(|job| job.view.clone())
    }
}
