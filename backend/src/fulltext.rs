use std::collections::HashSet;
use std::time::Duration;

use futures::StreamExt;
use once_cell::sync::Lazy;
use scraper::{Html, Selector};
use serde_json::json;

use crate::model::{Article, Lean, ProgressFn, ProgressUpdate};

static ARTICLE_P: Lazy<Selector> = Lazy::new(|| Selector::parse("article p").unwrap());
static MAIN_P: Lazy<Selector> = Lazy::new(|| Selector::parse("main p").unwrap());
static ROLE_MAIN_P: Lazy<Selector> = Lazy::new(|| Selector::parse("[role=main] p").unwrap());
static ANY_P: Lazy<Selector> = Lazy::new(|| Selector::parse("p").unwrap());

pub fn max_articles() -> usize {
    std::env::var("MIRANTE_FULLTEXT_MAX")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(12)
        .clamp(4, 24)
}

pub fn enabled() -> bool {
    std::env::var("MIRANTE_FULLTEXT_ENABLED")
        .map(|value| value != "false" && value != "0")
        .unwrap_or(true)
}

#[derive(Debug)]
struct Extracted {
    final_url: String,
    excerpt: String,
}

pub async fn enrich(articles: &mut [Article], max: usize, progress: Option<&ProgressFn>) -> usize {
    if !enabled() || articles.is_empty() {
        return 0;
    }

    let selected = select_indices(articles, max);
    if selected.is_empty() {
        return 0;
    }

    let total = selected.len();
    report(
        progress,
        ProgressUpdate {
            step: 4,
            label: "Lendo matérias em profundidade".to_string(),
            detail: format!("Selecionando {total} matérias para leitura completa"),
            progress: 50.0,
            current: Some(0),
            total: Some(total),
        },
    );

    let client = match reqwest::Client::builder()
        .user_agent(crate::fetch::user_agent())
        .timeout(Duration::from_secs(7))
        .redirect(reqwest::redirect::Policy::limited(6))
        .cookie_store(true)
        .build()
    {
        Ok(client) => client,
        Err(_) => return 0,
    };

    let tasks: Vec<(usize, String)> = selected
        .iter()
        .map(|index| (*index, articles[*index].url.clone()))
        .collect();
    let mut stream = futures::stream::iter(tasks)
        .map(|(index, url)| {
            let client = client.clone();
            async move { (index, fetch_one(client, url).await) }
        })
        .buffer_unordered(6);

    let mut read = 0usize;
    let mut completed = 0usize;
    while let Some((index, extracted)) = stream.next().await {
        completed += 1;
        if let Some(extracted) = extracted {
            read += 1;
            let article = &mut articles[index];
            if !extracted.final_url.contains("news.google.com")
                && extracted.final_url.starts_with("http")
            {
                article.url = extracted.final_url;
            }
            article.deep_read = true;
            article.excerpt = Some(extracted.excerpt.clone());
            article.relevance = (article.relevance + 0.12).min(1.0);
        }

        report(
            progress,
            ProgressUpdate {
                step: 4,
                label: "Lendo matérias em profundidade".to_string(),
                detail: format!("Lendo {completed} de {total} matérias"),
                progress: 50.0 + 25.0 * (completed as f64 / total as f64),
                current: Some(completed),
                total: Some(total),
            },
        );
    }

    report(
        progress,
        ProgressUpdate {
            step: 4,
            label: "Lendo matérias em profundidade".to_string(),
            detail: format!("{read} de {total} matérias lidas na íntegra"),
            progress: 76.0,
            current: Some(read),
            total: Some(total),
        },
    );

    read
}

fn select_indices(articles: &[Article], max: usize) -> Vec<usize> {
    let mut ranked: Vec<usize> = (0..articles.len())
        .filter(|index| !articles[*index].url.contains("news.google.com"))
        .collect();
    if ranked.is_empty() {
        return Vec::new();
    }
    ranked.sort_by(|a, b| {
        let direct_a = !articles[*a].url.contains("news.google.com");
        let direct_b = !articles[*b].url.contains("news.google.com");
        direct_b.cmp(&direct_a).then_with(|| {
            articles[*b]
                .relevance
                .partial_cmp(&articles[*a].relevance)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    });

    let mut selected = Vec::new();
    let mut chosen = HashSet::new();
    let mut domains = HashSet::new();

    // Garante até 3 matérias por espectro, com diversidade de fontes.
    for lean in [Lean::Left, Lean::Center, Lean::Right, Lean::Unknown] {
        let mut count = 0usize;
        for index in ranked
            .iter()
            .copied()
            .filter(|index| articles[*index].lean == lean)
        {
            if selected.len() >= max || count >= 3 {
                break;
            }
            let domain = articles[index].domain.clone();
            if domains.insert(domain) {
                selected.push(index);
                chosen.insert(index);
                count += 1;
            }
        }
    }

    // Preenche com os mais relevantes, permitindo repetir fonte só no fim.
    for index in ranked {
        if selected.len() >= max {
            break;
        }
        if chosen.insert(index) {
            selected.push(index);
        }
    }

    selected
}

async fn resolve_google_news_url(_client: &reqwest::Client, url: &str) -> Option<String> {
    let path = url.split('?').next().unwrap_or(url);
    let id = path.rsplit('/').next()?.to_string();
    if id.is_empty() {
        return None;
    }
    let client = reqwest::Client::builder()
        .user_agent(
            "Mozilla/5.0 (X11; Linux aarch64) AppleWebKit/537.36 Chrome/125.0 Safari/537.36",
        )
        .timeout(Duration::from_secs(8))
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .ok()?;
    let mut page = format!("https://news.google.com/rss/articles/{id}?hl=en-US&gl=US&ceid=US:en");
    let mut html = String::new();
    for _ in 0..3 {
        let response = client.get(&page).send().await.ok()?;
        if response.status().is_redirection() {
            let location = response
                .headers()
                .get(reqwest::header::LOCATION)?
                .to_str()
                .ok()?;
            let next = reqwest::Url::parse(&page).ok()?.join(location).ok()?;
            if next.host_str() != Some("news.google.com") {
                return None;
            }
            page = next.to_string();
            continue;
        }
        if !response.status().is_success() {
            return None;
        }
        html = response.text().await.ok()?;
        break;
    }
    let signature = regex_capture(&html, r#"data-n-a-sg="([^"]+)""#)?;
    let timestamp = regex_capture(&html, r#"data-n-a-ts="([^"]+)""#)?;

    let context = json!([
        "X",
        "X",
        ["X", "X"],
        null,
        null,
        1,
        1,
        "US:en",
        null,
        1,
        null,
        null,
        null,
        null,
        null,
        0,
        1
    ]);
    let inner = json!([
        "garturlreq",
        context,
        id,
        timestamp.parse::<i64>().unwrap_or(0),
        signature
    ]);
    let envelope = json!([[["Fbv4je", inner.to_string(), null, "0"]]]);
    let encoded = form_encode(&serde_json::to_string(&envelope).ok()?);
    let body = format!("f.req={encoded}");
    tracing::debug!(len = body.len(), prefix = %body.chars().take(80).collect::<String>(), "google news: corpo");

    let response = match client
        .post("https://news.google.com/_/DotsSplashUi/data/batchexecute")
        .header(
            reqwest::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded;charset=UTF-8",
        )
        .header(reqwest::header::REFERER, "https://news.google.com/")
        .body(body)
        .send()
        .await
    {
        Ok(response) => response,
        Err(error) => {
            tracing::warn!(%error, "google news: falha no POST");
            return None;
        }
    };
    let status = response.status();
    let text = match response.text().await {
        Ok(text) => text,
        Err(error) => {
            tracing::warn!(%error, "google news: falha ao ler resposta");
            return None;
        }
    };
    tracing::debug!(%status, len = text.len(), body = %text.chars().take(500).collect::<String>().replace('\n', "\\n"), "google news: batchexecute");

    if let Some(position) = text.find("garturlres") {
        let rest = &text[position..];
        if let Some(http) = rest.find("http") {
            let candidate = &rest[http..];
            let end = candidate.find(['"', '\\']).unwrap_or(candidate.len());
            let decoded = candidate[..end].to_string();
            if !decoded.contains("news.google.com") && !decoded.contains("google.com/") {
                tracing::debug!(%decoded, "google news: URL resolvida");
                return Some(decoded);
            }
        }
    }

    tracing::warn!(body = %text.chars().take(240).collect::<String>(), "google news: URL não decodificada");
    None
}

fn form_encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len() * 3);
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~' | b'/') {
            out.push(byte as char);
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

fn regex_capture(value: &str, pattern: &str) -> Option<String> {
    regex::Regex::new(pattern)
        .ok()?
        .captures(value)?
        .get(1)
        .map(|capture| capture.as_str().to_string())
}

async fn fetch_one(client: reqwest::Client, url: String) -> Option<Extracted> {
    let url = if url.contains("news.google.com") {
        resolve_google_news_url(&client, &url).await.unwrap_or(url)
    } else {
        url
    };
    let response = client
        .get(&url)
        .header(
            reqwest::header::ACCEPT,
            "text/html,application/xhtml+xml;q=0.9,*/*;q=0.8",
        )
        .send()
        .await
        .ok()?;
    let status = response.status();
    let final_url = response.url().to_string();
    let response = response.error_for_status().ok()?;
    tracing::debug!(%url, %final_url, %status, "fulltext: resposta");
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("")
        .to_lowercase();
    if !content_type.is_empty() && !content_type.contains("text/html") {
        tracing::warn!(%url, %content_type, "fulltext: content-type não HTML");
        return None;
    }

    let bytes = response.bytes().await.ok()?;
    if bytes.len() > 2_500_000 {
        return None;
    }
    let body = decode_html(&bytes);
    let document = Html::parse_document(&body);

    let mut paragraphs: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for selector in [&*ARTICLE_P, &*MAIN_P, &*ROLE_MAIN_P, &*ANY_P] {
        for element in document.select(selector) {
            let text = element
                .text()
                .collect::<Vec<_>>()
                .join(" ")
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            if !is_good_paragraph(&text) {
                continue;
            }
            let key = text.to_lowercase();
            if seen.insert(key) {
                paragraphs.push(text);
            }
            if paragraphs.len() >= 32 {
                break;
            }
        }
        if paragraphs.len() >= 8 {
            break;
        }
    }

    if paragraphs.is_empty() {
        tracing::warn!(%url, "fulltext: nenhum parágrafo extraído");
        return None;
    }

    let joined = paragraphs.join(" ");
    if joined.chars().count() < 260 {
        tracing::warn!(%url, chars = joined.chars().count(), "fulltext: texto curto");
        return None;
    }

    let excerpt: String = joined.chars().take(340).collect();
    let excerpt = excerpt
        .rsplit_once(' ')
        .map(|(head, _)| head.to_string())
        .unwrap_or(excerpt);

    Some(Extracted {
        final_url,
        excerpt: format!("{excerpt}…"),
    })
}

fn is_good_paragraph(text: &str) -> bool {
    if text.len() < 40 || text.len() > 4000 {
        return false;
    }
    let alphabetic = text.chars().filter(|c| c.is_alphabetic()).count() as f64;
    let total = text.chars().count() as f64;
    if total == 0.0 || alphabetic / total < 0.55 {
        return false;
    }
    let lower = text.to_lowercase();
    let boilerplate = [
        "cookies",
        "cookie",
        "assine",
        "newsletter",
        "publicidade",
        "todos os direitos",
        "continuar lendo",
        "compartilhe",
        "siga-nos",
        "receba",
        "opens in new window",
        "adicione",
        "leia também",
        "veja também",
        "siga o",
        "whatsapp",
        "telegram",
        "entre em contato",
        "termos de uso",
        "política de privacidade",
        "aceitar",
    ];
    if boilerplate.iter().any(|needle| lower.contains(needle)) {
        return false;
    }
    true
}

fn decode_html(bytes: &[u8]) -> String {
    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(500)]).to_ascii_lowercase();
    if head.contains("iso-8859-1") || head.contains("windows-1252") {
        let (decoded, _, _) = encoding_rs::WINDOWS_1252.decode(bytes);
        decoded.into_owned()
    } else {
        String::from_utf8_lossy(bytes).into_owned()
    }
}

fn report(progress: Option<&ProgressFn>, update: ProgressUpdate) {
    if let Some(callback) = progress {
        callback(update);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_cookie_banner() {
        assert!(!is_good_paragraph(
            "Este site usa cookies para melhorar sua experiência e personalizar conteúdo."
        ));
    }

    #[test]
    fn accepts_real_paragraph() {
        assert!(is_good_paragraph(
            "O Senado aprovou nesta terça-feira o texto-base da reforma tributária em primeiro turno, após horas de negociação entre os líderes partidários."
        ));
    }
}
