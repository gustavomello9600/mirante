use serde::Deserialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Lean {
    Left,
    Center,
    Right,
    Unknown,
}

impl Lean {
    pub fn label(self) -> &'static str {
        match self {
            Lean::Left => "Esquerda",
            Lean::Center => "Centro",
            Lean::Right => "Direita",
            Lean::Unknown => "Sem avaliação",
        }
    }

    pub fn tag_class(self) -> &'static str {
        match self {
            Lean::Left => "tag-left",
            Lean::Center => "tag-center",
            Lean::Right => "tag-right",
            Lean::Unknown => "tag-unknown",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Factuality {
    High,
    Mixed,
    Low,
    Unknown,
}

impl Factuality {
    pub fn label(self) -> &'static str {
        match self {
            Factuality::High => "Alta",
            Factuality::Mixed => "Mista",
            Factuality::Low => "Baixa",
            Factuality::Unknown => "Sem avaliação",
        }
    }

    pub fn tag_class(self) -> &'static str {
        match self {
            Factuality::High => "tag-fact-high",
            Factuality::Mixed => "tag-fact-mixed",
            Factuality::Low => "tag-fact-low",
            Factuality::Unknown => "tag-unknown",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BiasCounts {
    pub left: usize,
    pub center: usize,
    pub right: usize,
    pub unknown: usize,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FactCounts {
    pub high: usize,
    pub mixed: usize,
    pub low: usize,
    pub unknown: usize,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceCount {
    pub name: String,
    pub domain: String,
    pub count: usize,
    pub lean: Lean,
    pub factuality: Factuality,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    pub total_articles: usize,
    pub total_sources: usize,
    pub rated_articles: usize,
    pub unknown_articles: usize,
    pub bias: BiasCounts,
    pub factuality: FactCounts,
    pub window_days: u32,
    pub date_min: Option<String>,
    pub date_max: Option<String>,
    pub top_sources: Vec<SourceCount>,
    pub clusters: usize,
    pub deep_read: usize,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Article {
    pub id: String,
    pub title: String,
    pub url: String,
    pub source: String,
    pub domain: String,
    pub published_at: Option<String>,
    pub lean: Lean,
    pub factuality: Factuality,
    pub ownership: Option<String>,
    pub rating_source: String,
    pub cluster_id: usize,
    pub cluster_title: String,
    pub snippet: Option<String>,
    pub relevance: f64,
    pub deep_read: bool,
    pub excerpt: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cluster {
    pub id: usize,
    pub title: String,
    pub size: usize,
    pub article_ids: Vec<String>,
    pub lean_counts: BiasCounts,
    pub factuality_counts: FactCounts,
    pub top_terms: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportParagraph {
    pub text: String,
    pub citations: Vec<usize>,
    pub label: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Callout {
    pub label: String,
    pub title: String,
    pub body: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportSection {
    pub heading: String,
    pub body: Vec<ReportParagraph>,
    pub callout: Option<Callout>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportFeature {
    pub kicker: String,
    pub title: String,
    pub dek: String,
    pub body: Vec<ReportParagraph>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub title: String,
    pub dek: String,
    #[serde(default)]
    pub feature: Option<ReportFeature>,
    pub sections: Vec<ReportSection>,
    pub limitations: Vec<String>,
    pub generated_by: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelinePoint {
    pub date: String,
    pub left: usize,
    pub center: usize,
    pub right: usize,
    pub unknown: usize,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScatterPoint {
    pub name: String,
    pub domain: String,
    pub lean: Lean,
    pub factuality: Factuality,
    pub count: usize,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Analysis {
    pub query: String,
    pub generated_at: String,
    pub mode: String,
    pub cached: bool,
    pub stats: Stats,
    pub articles: Vec<Article>,
    pub clusters: Vec<Cluster>,
    pub report: Report,
    pub timeline: Vec<TimelinePoint>,
    pub scatter: Vec<ScatterPoint>,
    pub limitations: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Topic {
    pub label: String,
    pub query: String,
    pub description: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceInfo {
    pub name: String,
    pub domain: String,
    pub lean: Lean,
    pub factuality: Factuality,
    pub ownership: Option<String>,
    pub rating_source: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiErrorBody {
    pub error: String,
    pub detail: Option<String>,
}
