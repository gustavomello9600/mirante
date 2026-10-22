use crate::model::{
    Article, Callout, Report, ReportFeature, ReportParagraph, ReportSection, Stats,
};
use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};

static INLINE_CITATIONS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\s*\[\d+(?:\s*,\s*\d+)*\]").unwrap());
static INLINE_CITATION_NUMBERS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\[(\d+(?:\s*,\s*\d+)*)\]").unwrap());

#[derive(Clone)]
pub struct LlmConfig {
    pub provider: String,
    pub base_url: String,
    pub model: String,
    pub api_key: String,
}

impl std::fmt::Debug for LlmConfig {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LlmConfig")
            .field("provider", &self.provider)
            .field("base_url", &self.base_url)
            .field("model", &self.model)
            .field("api_key", &"<redacted>")
            .finish()
    }
}

impl LlmConfig {
    pub fn from_env() -> Option<Self> {
        let provider = std::env::var("MIRANTE_LLM_PROVIDER")
            .unwrap_or_else(|_| "none".to_string())
            .to_lowercase();
        if provider == "none" || provider.is_empty() {
            return None;
        }
        let api_key = std::env::var("MIRANTE_LLM_API_KEY").unwrap_or_default();
        if api_key.is_empty() && provider != "mock" {
            tracing::warn!(
                "MIRANTE_LLM_PROVIDER definido sem MIRANTE_LLM_API_KEY; usando extrativo"
            );
            return None;
        }
        let base_url =
            std::env::var("MIRANTE_LLM_BASE_URL").unwrap_or_else(|_| match provider.as_str() {
                "openrouter" => "https://openrouter.ai/api/v1".to_string(),
                "openai" => "https://api.openai.com/v1".to_string(),
                "mock" => "http://127.0.0.1:18080/v1".to_string(),
                _ => "https://openrouter.ai/api/v1".to_string(),
            });
        let model =
            std::env::var("MIRANTE_LLM_MODEL").unwrap_or_else(|_| "openai/gpt-4o-mini".to_string());
        Some(Self {
            provider,
            base_url,
            model,
            api_key,
        })
    }

    pub fn label(&self) -> String {
        format!("{} · {}", self.provider, self.model)
    }
}

#[derive(Debug, Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    temperature: f32,
    max_tokens: u32,
    response_format: ResponseFormat,
    messages: Vec<Message>,
}

#[derive(Debug, Serialize)]
struct ResponseFormat {
    #[serde(rename = "type")]
    kind: &'static str,
}

#[derive(Debug, Serialize)]
struct Message {
    role: &'static str,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ChoiceMessage,
}

#[derive(Debug, Deserialize)]
struct ChoiceMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct LlmReport {
    #[serde(default)]
    feature: Option<LlmFeature>,
    #[serde(default)]
    sections: Vec<LlmSection>,
    #[serde(default)]
    limitations: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct LlmFeature {
    #[serde(default)]
    kicker: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    dek: String,
    #[serde(default)]
    paragraphs: Vec<LlmParagraph>,
}

#[derive(Debug, Deserialize)]
struct LlmSection {
    heading: String,
    #[serde(default)]
    paragraphs: Vec<LlmParagraph>,
    #[serde(default)]
    callout: Option<LlmCallout>,
}

#[derive(Debug, Deserialize)]
struct LlmParagraph {
    text: String,
    #[serde(default)]
    citations: Vec<usize>,
    #[serde(default)]
    label: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LlmCallout {
    label: String,
    title: String,
    body: String,
}

pub async fn generate(
    config: &LlmConfig,
    query: &str,
    articles: &[Article],
    stats: &Stats,
    dek: &str,
) -> Result<Report> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()?;

    let shown = articles.len().min(80);
    let article_lines = articles
        .iter()
        .take(shown)
        .enumerate()
        .map(|(index, article)| {
            let mut line = format!(
                "[{}] {} | {} | {} | {}",
                index + 1,
                article.source,
                article.lean.label(),
                article.factuality.label(),
                article.title
            );
            if let Some(excerpt) = &article.excerpt {
                line.push_str(&format!("\n   Trecho lido: {excerpt}"));
            }
            line
        })
        .collect::<Vec<_>>()
        .join("\n");

    let system = "Você é o editor de cobertura do Mirante. Escreva em português do Brasil, com voz editorial, precisa e calma. Não invente fatos, números, intenções, nomes de veículos ou citações. Use apenas as manchetes, os trechos marcados como “Trecho lido” e os números fornecidos. Quando houver “Trecho lido”, você pode dizer que a matéria foi lida; não afirme que a análise usou apenas manchetes. Toda afirmação factual precisa citar ao menos um artigo pelo campo citations; NÃO escreva [1] no texto. Se a amostra não sustentar uma conclusão, diga isso. 'Sem avaliação' significa ausência de rating, não neutralidade; não trate como cobertura neutra. A peça principal deve integrar as fontes e perspectivas, não ser uma colagem de manchetes. Evite conectivos genéricos como “Além disso” e “Por outro lado”; prefira uma linha argumentativa. Responda somente JSON válido.";
    let user = format!(
        r#"Tema: {query}
Janela: {window} dias
Amostra: {n} artigos de {m} veículos
Viés: esquerda {left}, centro {center}, direita {right}, sem avaliação {unknown}
Factualidade: alta {high}, mista {mixed}, baixa {low}, sem avaliação {fact_unknown}
Leitura profunda: {deep} matérias com trechos extraídos do texto

Manchetes:
{articles}

Gere um relance com uma matéria própria e duas seções:
1. "feature" — a matéria principal do Mirante, escrita por você, com 4 a 6 parágrafos. Ela deve integrar as fontes e perspectivas da amostra numa leitura contínua: onde as coberturas convergem, onde divergem, que ênfases e interesses cada campo ilumina, quais fatos e números sustentam a comparação e o que a amostra não permite concluir. Não percorra artigo por artigo e não repita a estrutura "um lado disse, o outro disse": construa uma peça própria, com tese editorial e evidências. Quando houver fontes de grupos diferentes sobre o mesmo fato, confronte-as no mesmo parágrafo. Título curto em sentence case e dek de uma frase. Cada parágrafo factual deve ter citations.
2. "O que aconteceu" — 1 parágrafo curto e factual sobre a cobertura, com citações. A matéria principal vem logo depois desta seção.
3. "O que ficou de fora" — obrigatória, com um callout que traga um ângulo subrepresentado e o número de artigos; se não houver evidência, diga que a amostra não sustenta a conclusão. Não omita esta seção.

NÃO gere a seção "Como cada lado enquadra": ela foi substituída pela lista de matérias por espectro na interface.

Formato JSON:
{{
  "feature": {{
    "kicker": "A matéria do Mirante",
    "title": "...",
    "dek": "...",
    "paragraphs": [{{"text": "...", "citations": [1,2]}}]
  }},
  "sections": [
    {{"heading": "O que aconteceu", "paragraphs": [{{"text": "...", "citations": [1,2]}}]}},
    {{"heading": "O que ficou de fora", "paragraphs": [{{"text": "...", "citations": [1,2]}}], "callout": {{"label":"...","title":"...","body":"..."}}}}
  ],
  "limitations": ["...", "..."]
}}

Regras: citations são índices de 1 a {shown}. Não cite artigo inexistente. Não use markdown. Não escreva números que não estejam no bloco acima."#,
        query = query,
        shown = shown,
        window = stats.window_days,
        n = stats.total_articles,
        m = stats.total_sources,
        left = stats.bias.left,
        center = stats.bias.center,
        right = stats.bias.right,
        unknown = stats.bias.unknown,
        high = stats.factuality.high,
        mixed = stats.factuality.mixed,
        low = stats.factuality.low,
        fact_unknown = stats.factuality.unknown,
        deep = stats.deep_read,
        articles = article_lines,
    );

    let request = ChatRequest {
        model: &config.model,
        temperature: 0.2,
        max_tokens: 3200,
        response_format: ResponseFormat {
            kind: "json_object",
        },
        messages: vec![
            Message {
                role: "system",
                content: system.to_string(),
            },
            Message {
                role: "user",
                content: user,
            },
        ],
    };

    let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));
    let mut builder = client.post(url).json(&request);
    if !config.api_key.is_empty() {
        builder = builder.bearer_auth(&config.api_key);
    }
    if config.provider == "openrouter" {
        let referer = std::env::var("MIRANTE_PUBLIC_URL")
            .unwrap_or_else(|_| env!("CARGO_PKG_REPOSITORY").to_string());
        builder = builder
            .header("HTTP-Referer", referer)
            .header("X-Title", "Mirante");
    }

    let response = builder.send().await.context("chamada LLM")?;
    let status = response.status();
    let body = response.text().await?;
    if !status.is_success() {
        anyhow::bail!(
            "LLM status {}: {}",
            status,
            body.chars().take(300).collect::<String>()
        );
    }

    let parsed: ChatResponse = serde_json::from_str(&body).context("resposta LLM")?;
    let content = parsed
        .choices
        .first()
        .map(|choice| choice.message.content.as_str())
        .unwrap_or("");
    let llm: LlmReport = serde_json::from_str(content).context("JSON do relance")?;

    let LlmReport {
        feature,
        sections: raw_sections,
        limitations: model_limitations,
    } = llm;

    let clean_paragraphs = |paragraphs: Vec<LlmParagraph>| -> Vec<ReportParagraph> {
        let mut body = Vec::new();
        for paragraph in paragraphs {
            if paragraph.text.trim().is_empty() {
                continue;
            }
            let mut citations: Vec<usize> = paragraph.citations;
            for capture in INLINE_CITATION_NUMBERS.captures_iter(&paragraph.text) {
                for raw in capture[1].split(',') {
                    if let Ok(index) = raw.trim().parse::<usize>() {
                        citations.push(index);
                    }
                }
            }
            citations.retain(|index| *index >= 1 && *index <= shown);
            citations.sort_unstable();
            citations.dedup();
            citations.truncate(6);
            let cleaned = INLINE_CITATIONS.replace_all(paragraph.text.trim(), "");
            let cleaned = cleaned
                .replace(" .", ".")
                .replace(" ,", ",")
                .trim()
                .to_string();
            if cleaned.is_empty() {
                continue;
            }
            body.push(ReportParagraph {
                text: cleaned,
                citations,
                label: paragraph.label,
            });
        }
        body
    };

    let feature = feature.and_then(|feature| {
        let mut body = clean_paragraphs(feature.paragraphs);
        for paragraph in &mut body {
            paragraph.label = None;
        }
        if body.len() < 2 || feature.title.trim().is_empty() {
            tracing::warn!("LLM não entregou a matéria principal; relance segue sem feature");
            return None;
        }
        Some(ReportFeature {
            kicker: if feature.kicker.trim().is_empty() {
                "A matéria do Mirante".to_string()
            } else {
                feature.kicker.trim().to_string()
            },
            title: feature.title.trim().to_string(),
            dek: feature.dek.trim().to_string(),
            body,
        })
    });

    let mut sections = Vec::new();
    for section in raw_sections {
        let body = clean_paragraphs(section.paragraphs);
        if body.is_empty() {
            continue;
        }
        sections.push(ReportSection {
            heading: section.heading,
            body,
            callout: section.callout.map(|callout| Callout {
                label: callout.label,
                title: callout.title,
                body: callout.body,
            }),
        });
    }
    sections.retain(|section| !section.heading.to_lowercase().contains("lado"));

    if sections.len() < 2 {
        anyhow::bail!("relance LLM incompleto");
    }

    let mut limitations = model_limitations;
    limitations.push(format!(
        "Relance gerado por {} a partir de manchetes e trechos; números calculados pelo Mirante.",
        config.label()
    ));

    Ok(Report {
        title: "O relance".to_string(),
        dek: dek.to_string(),
        feature,
        sections,
        limitations,
        generated_by: format!("llm:{}", config.label()),
    })
}

#[cfg(test)]
mod tests {
    use super::LlmConfig;

    #[test]
    fn debug_redacts_api_key() {
        let config = LlmConfig {
            provider: "openrouter".to_string(),
            base_url: "https://example.com/v1".to_string(),
            model: "test-model".to_string(),
            api_key: "super-secret-test-key".to_string(),
        };
        let debug = format!("{config:?}");
        assert!(!debug.contains("super-secret-test-key"));
        assert!(debug.contains("<redacted>"));
    }
}
