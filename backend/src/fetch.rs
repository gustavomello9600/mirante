use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashSet;

use crate::catalog::domain_from_url;
use crate::text::{clean_text, normalize_query, strip_source_suffix, surface_tokens};

static ITEM_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?s)<item>(.*?)</item>").unwrap());
static TITLE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?s)<title>(.*?)</title>").unwrap());
static LINK_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?s)<link>(.*?)</link>").unwrap());
static PUB_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?s)<pubDate>(.*?)</pubDate>").unwrap());
static SOURCE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?s)<source\s+url="([^"]*)"\s*>(.*?)</source>"#).unwrap());
static BING_SOURCE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?s)<News:Source>(.*?)</News:Source>").unwrap());
static DESC_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?s)<description>(.*?)</description>").unwrap());
static ENTITY_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"&#(?:x([0-9a-fA-F]+)|([0-9]+));").unwrap());

const SITE_DOMAINS: &[&str] = &[
    "brasildefato.com.br",
    "cartacapital.com.br",
    "revistaforum.com.br",
    "apublica.org",
    "sul21.com.br",
    "g1.globo.com",
    "folha.uol.com.br",
    "estadao.com.br",
    "uol.com.br",
    "cnnbrasil.com.br",
    "poder360.com.br",
    "agenciabrasil.ebc.com.br",
    "valor.globo.com",
    "veja.abril.com.br",
    "gazetadopovo.com.br",
    "jovempan.com.br",
    "r7.com",
];

#[derive(Clone, Debug)]
pub struct RawArticle {
    pub title: String,
    pub url: String,
    pub source: String,
    pub source_url: String,
    pub published_at: Option<String>,
    pub snippet: Option<String>,
}

#[derive(Clone)]
pub struct Fetcher {
    client: reqwest::Client,
    google_base: String,
    gdelt_base: String,
}

/// User-Agent enviado aos veículos e feeds.
///
/// O valor padrão aponta para o repositório do projeto, como pede a etiqueta
/// de identificação de leitores; quem opera uma instância pode sobrescrever
/// com `MIRANTE_USER_AGENT`.
pub fn user_agent() -> String {
    std::env::var("MIRANTE_USER_AGENT").unwrap_or_else(|_| {
        format!(
            "Mirante/{} (+{}; leitor de cobertura)",
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_REPOSITORY")
        )
    })
}

impl Fetcher {
    pub fn new() -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent(user_agent())
            .timeout(std::time::Duration::from_secs(18))
            .build()
            .context("cliente HTTP")?;
        Ok(Self {
            client,
            google_base: std::env::var("MIRANTE_GOOGLE_NEWS_BASE")
                .unwrap_or_else(|_| "https://news.google.com".to_string()),
            gdelt_base: std::env::var("MIRANTE_GDELT_BASE")
                .unwrap_or_else(|_| "https://api.gdeltproject.org".to_string()),
        })
    }

    pub async fn fetch(&self, query: &str, window_days: u32) -> Result<Vec<RawArticle>> {
        let query = normalize_query(query);
        if query.is_empty() {
            anyhow::bail!("tema vazio");
        }
        let query_terms = surface_tokens(&query);

        let broad = self.fetch_google_variants(&query, &query_terms, window_days);
        let bing = self.fetch_bing(&query, &query_terms, window_days);
        let (mut all, mut bing_items) = tokio::join!(broad, bing);
        all.append(&mut bing_items);

        // Se a busca ampla não encontrou nada, não vale gastar 17 buscas por site.
        if all.len() >= 3 {
            let mut site_items = self
                .fetch_google_sites(&query, &query_terms, window_days)
                .await;
            all.append(&mut site_items);
        }

        if !all.is_empty() && all.len() < 12 {
            match tokio::time::timeout(
                std::time::Duration::from_secs(6),
                self.fetch_gdelt(&query, window_days),
            )
            .await
            {
                Ok(Ok(mut items)) => all.append(&mut items),
                Ok(Err(error)) => tracing::warn!(%error, "falha ao buscar GDELT"),
                Err(_) => tracing::warn!("timeout ao buscar GDELT"),
            }
        }

        // Publisher URLs first: they are better citations than Google News redirects.
        all.sort_by_key(|article| article.url.contains("news.google.com"));
        let mut seen = HashSet::new();
        all.retain(|article| {
            let key = format!(
                "{}|{}",
                normalize_query(&strip_source_suffix(&article.title, &article.source)),
                domain_from_url(&article.source_url)
            );
            seen.insert(key)
        });

        all.sort_by(|a, b| b.published_at.cmp(&a.published_at));
        all.truncate(280);
        Ok(all)
    }

    async fn fetch_bing(
        &self,
        query: &str,
        query_terms: &[String],
        window_days: u32,
    ) -> Vec<RawArticle> {
        let mut items = Vec::new();
        for (index, term) in [format!("\"{query}\""), query.to_string()]
            .iter()
            .enumerate()
        {
            let url = format!(
                "https://www.bing.com/news/search?format=RSS&q={}&setlang=pt-BR&cc=BR",
                urlencoding::encode(term)
            );
            let response = match self.client.get(&url).send().await {
                Ok(response) => match response.error_for_status() {
                    Ok(response) => response,
                    Err(_) => continue,
                },
                Err(_) => continue,
            };
            let body = match response.text().await {
                Ok(body) => body,
                Err(_) => continue,
            };
            let mut parsed = parse_bing_rss(&body, window_days);
            parsed.retain(|article| matches_query(&article.title, query, query_terms));
            tracing::debug!(parsed = parsed.len(), quoted = index == 0, "bing news");
            items.append(&mut parsed);
            if !items.is_empty() {
                break;
            }
        }
        items.truncate(30);
        items
    }

    async fn fetch_google_variants(
        &self,
        query: &str,
        query_terms: &[String],
        window_days: u32,
    ) -> Vec<RawArticle> {
        let variants = [
            format!("{query} when:{window_days}d"),
            format!("{query} análise when:{window_days}d"),
            format!("{query} governo when:{window_days}d"),
            format!("{query} empresas when:{window_days}d"),
        ];
        let fetcher = self.clone();
        let futures = variants.iter().cloned().map(move |variant| {
            let fetcher = fetcher.clone();
            async move {
                tokio::time::timeout(
                    std::time::Duration::from_secs(6),
                    fetcher.fetch_google(&variant, window_days),
                )
                .await
            }
        });
        let results = futures::future::join_all(futures).await;
        let mut all = Vec::new();
        for result in results {
            match result {
                Ok(Ok(mut items)) => {
                    items.retain(|article| matches_query(&article.title, query, query_terms));
                    all.append(&mut items);
                }
                Ok(Err(error)) => tracing::warn!(%error, "falha ao buscar Google News"),
                Err(_) => tracing::warn!("timeout ao buscar Google News"),
            }
        }
        all
    }

    async fn fetch_google_sites(
        &self,
        query: &str,
        query_terms: &[String],
        window_days: u32,
    ) -> Vec<RawArticle> {
        let fetcher = self.clone();
        let futures = SITE_DOMAINS.iter().map(move |domain| {
            let fetcher = fetcher.clone();
            let query = query.to_string();
            let terms = query_terms.to_vec();
            let domain = domain.to_string();
            async move {
                let site_query = format!("{} site:{} when:{}d", query, domain, window_days);
                match tokio::time::timeout(
                    std::time::Duration::from_secs(6),
                    fetcher.fetch_google(&site_query, window_days),
                )
                .await
                {
                    Ok(Ok(mut items)) => {
                        items.retain(|article| {
                            let actual = domain_from_url(&article.source_url);
                            (actual == domain || actual.ends_with(&format!(".{domain}")))
                                && matches_query(&article.title, &query, &terms)
                        });
                        items.truncate(12);
                        items
                    }
                    Ok(Err(error)) => {
                        tracing::warn!(%error, domain, "falha em busca por site");
                        Vec::new()
                    }
                    Err(_) => {
                        tracing::warn!(domain, "timeout em busca por site");
                        Vec::new()
                    }
                }
            }
        });

        let results = futures::future::join_all(futures).await;
        results.into_iter().flatten().collect()
    }

    async fn fetch_google(&self, query: &str, window_days: u32) -> Result<Vec<RawArticle>> {
        let url = format!(
            "{}/rss/search?q={}&hl=pt-BR&gl=BR&ceid=BR:pt-419",
            self.google_base,
            urlencoding::encode(query)
        );
        let response = self.client.get(&url).send().await?.error_for_status()?;
        let body = response.text().await?;
        Ok(parse_rss(&body, window_days, None, None))
    }

    async fn fetch_gdelt(&self, query: &str, window_days: u32) -> Result<Vec<RawArticle>> {
        let timespan = format!("{window_days}d");
        let quoted = format!("\"{}\"", query.replace('"', ""));
        let url = format!(
            "{}/api/v2/doc/doc?query={}&mode=artlist&format=json&maxrecords=100&timespan={}&sort=datedesc",
            self.gdelt_base,
            urlencoding::encode(&quoted),
            timespan
        );
        let response = self.client.get(&url).send().await?.error_for_status()?;
        let payload: GdeltResponse = response
            .json()
            .await
            .unwrap_or(GdeltResponse { articles: vec![] });

        Ok(payload
            .articles
            .into_iter()
            .map(|item| {
                let domain = domain_from_url(&item.url);
                let source = source_name_from_domain(&domain);
                RawArticle {
                    title: clean_text(&item.title),
                    url: item.url,
                    source,
                    source_url: if domain.is_empty() {
                        String::new()
                    } else {
                        format!("https://{domain}")
                    },
                    published_at: parse_gdelt_date(&item.seendate),
                    snippet: None,
                }
            })
            .collect())
    }
}

#[derive(Debug, Deserialize)]
struct GdeltResponse {
    #[serde(default)]
    articles: Vec<GdeltArticle>,
}

#[derive(Debug, Deserialize)]
struct GdeltArticle {
    url: String,
    title: String,
    seendate: String,
}

fn matches_query(title: &str, query: &str, query_terms: &[String]) -> bool {
    if query_terms.is_empty() {
        return title.to_lowercase().contains(&query.to_lowercase());
    }
    let title_terms: HashSet<String> = surface_tokens(title).into_iter().collect();
    let matched = query_terms
        .iter()
        .filter(|term| title_terms.contains(*term))
        .count();
    let required = if query_terms.len() >= 2 { 2 } else { 1 };
    matched >= required
}

fn parse_bing_rss(body: &str, window_days: u32) -> Vec<RawArticle> {
    let cutoff = Utc::now() - chrono::Duration::days(window_days as i64 + 1);
    let mut out = Vec::new();

    for item in ITEM_RE.captures_iter(body) {
        let block = item.get(1).map(|m| m.as_str()).unwrap_or("");
        let title = clean_text(&first_capture(&TITLE_RE, block));
        let link = first_capture(&LINK_RE, block);
        let pub_date = first_capture(&PUB_RE, block);
        let source_name = first_capture(&BING_SOURCE_RE, block);
        let description = clean_text(&first_capture(&DESC_RE, block));

        if title.is_empty() || link.is_empty() {
            continue;
        }

        let publisher = link
            .split("url=")
            .nth(1)
            .and_then(|value| value.split('&').next())
            .and_then(|value| urlencoding::decode(value).ok())
            .map(|value| value.into_owned())
            .unwrap_or_else(|| link.clone());
        let domain = domain_from_url(&publisher);
        let source = sanitize_source_name(&source_name, &domain);

        let published = parse_rfc2822(&pub_date);
        if let Some(value) = &published {
            if let Ok(parsed) = DateTime::parse_from_rfc3339(value) {
                if parsed.with_timezone(&Utc) < cutoff {
                    continue;
                }
            }
        }

        let cleaned_title = strip_source_suffix(&title, &source);
        if cleaned_title.len() < 12 {
            continue;
        }

        out.push(RawArticle {
            title: cleaned_title,
            url: publisher,
            source,
            source_url: if domain.is_empty() {
                String::new()
            } else {
                format!("https://{domain}")
            },
            published_at: published,
            snippet: if description.is_empty() {
                None
            } else {
                Some(description.chars().take(360).collect())
            },
        });
    }

    out
}

fn parse_rss(
    body: &str,
    window_days: u32,
    source_hint: Option<&str>,
    source_url_hint: Option<&str>,
) -> Vec<RawArticle> {
    let cutoff = Utc::now() - chrono::Duration::days(window_days as i64 + 1);
    let mut out = Vec::new();

    for item in ITEM_RE.captures_iter(body) {
        let block = item.get(1).map(|m| m.as_str()).unwrap_or("");
        let title_raw = first_capture(&TITLE_RE, block);
        let link = first_capture(&LINK_RE, block);
        let pub_date = first_capture(&PUB_RE, block);
        let (parsed_source_url, parsed_source_name) = SOURCE_RE
            .captures(block)
            .map(|capture| {
                (
                    decode_xml(capture.get(1).map(|m| m.as_str()).unwrap_or("")),
                    decode_xml(capture.get(2).map(|m| m.as_str()).unwrap_or("")),
                )
            })
            .unwrap_or_default();

        let title = clean_text(&title_raw);
        if title.is_empty() || link.is_empty() {
            continue;
        }

        let source_url = source_url_hint
            .map(str::to_string)
            .unwrap_or(parsed_source_url);
        let source = source_hint
            .map(str::to_string)
            .or_else(|| {
                if parsed_source_name.trim().is_empty() {
                    None
                } else {
                    Some(clean_text(&parsed_source_name))
                }
            })
            .unwrap_or_else(|| source_name_from_domain(&domain_from_url(&source_url)));

        let published = parse_rfc2822(&pub_date);
        if let Some(value) = &published {
            if let Ok(parsed) = DateTime::parse_from_rfc3339(value) {
                if parsed.with_timezone(&Utc) < cutoff {
                    continue;
                }
            }
        }

        let cleaned_title = strip_source_suffix(&title, &source);
        if cleaned_title.len() < 12 {
            continue;
        }

        out.push(RawArticle {
            title: cleaned_title,
            url: decode_xml(&link),
            source,
            source_url: source_url.clone(),
            published_at: published,
            snippet: None,
        });
    }

    out
}

fn first_capture(regex: &Regex, block: &str) -> String {
    regex
        .captures(block)
        .and_then(|capture| capture.get(1))
        .map(|m| decode_xml(m.as_str()))
        .unwrap_or_default()
}

fn decode_xml(value: &str) -> String {
    let without_cdata = value
        .trim()
        .trim_start_matches("<![CDATA[")
        .trim_end_matches("]]>")
        .to_string();
    let decoded = ENTITY_RE.replace_all(&without_cdata, |captures: &regex::Captures| {
        let number = captures
            .get(1)
            .and_then(|value| u32::from_str_radix(value.as_str(), 16).ok())
            .or_else(|| {
                captures
                    .get(2)
                    .and_then(|value| value.as_str().parse::<u32>().ok())
            });
        number
            .and_then(char::from_u32)
            .map(|character| character.to_string())
            .unwrap_or_default()
    });
    decoded
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
}

fn parse_rfc2822(value: &str) -> Option<String> {
    DateTime::parse_from_rfc2822(value.trim())
        .ok()
        .map(|date| date.with_timezone(&Utc).to_rfc3339())
}

fn parse_gdelt_date(value: &str) -> Option<String> {
    NaiveDateTime::parse_from_str(value, "%Y%m%dT%H%M%SZ")
        .ok()
        .map(|date| Utc.from_utc_datetime(&date).to_rfc3339())
}

fn sanitize_source_name(name: &str, domain: &str) -> String {
    let cleaned = name.trim().trim_end_matches(';').trim();
    let lower = cleaned.to_lowercase();
    let junk = cleaned.is_empty()
        || cleaned.len() > 80
        || lower.contains("sans-serif")
        || lower.contains("font-family")
        || lower.contains("open sans")
        || cleaned.contains('{')
        || cleaned.contains("<!doctype")
        || cleaned.contains("<html");
    if junk {
        source_name_from_domain(domain)
    } else {
        cleaned.to_string()
    }
}

fn source_name_from_domain(domain: &str) -> String {
    if domain.is_empty() {
        return "Fonte não identificada".to_string();
    }
    let label = domain.split('.').next().unwrap_or(domain);
    let mut chars = label.chars();
    match chars.next() {
        Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
        None => domain.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bing_direct_url_and_entities() {
        let date = Utc::now().to_rfc2822();
        let xml = format!(
            r#"<rss><channel><item><title>Tarifa de energia sobe &amp; pressiona</title><link>http://www.bing.com/news/apiclick.aspx?url=https%3a%2f%2fg1.globo.com%2feconomia%2ftarifa.ghtml&amp;c=1</link><description>Consumidores sentem o reajuste.</description><pubDate>{date}</pubDate><News:Source>g1</News:Source></item></channel></rss>"#
        );
        let items = parse_bing_rss(&xml, 7);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].url, "https://g1.globo.com/economia/tarifa.ghtml");
        assert_eq!(items[0].source, "g1");
        assert!(items[0].title.contains("Tarifa de energia"));
    }

    #[test]
    fn parses_google_news_fixture() {
        let xml = r#"<?xml version="1.0"?><rss><channel>
        <item>
          <title>Nova regra avança no Senado - Folha</title>
          <link>https://news.google.com/rss/articles/abc</link>
          <pubDate>Wed, 21 Oct 2026 10:00:00 GMT</pubDate>
          <source url="https://www1.folha.uol.com.br">Folha</source>
        </item>
        </channel></rss>"#;
        let items = parse_rss(xml, 7, None, None);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Nova regra avança no Senado");
        assert_eq!(items[0].source, "Folha");
        assert_eq!(items[0].source_url, "https://www1.folha.uol.com.br");
        assert!(items[0].published_at.is_some());
    }

    #[test]
    fn parses_regular_feed_with_hint() {
        let xml = r#"<rss><channel><item>
        <title>Reforma tributária avança no Senado</title>
        <link>https://exemplo.com/materia</link>
        <pubDate>Wed, 21 Oct 2026 10:00:00 GMT</pubDate>
        </item></channel></rss>"#;
        let items = parse_rss(xml, 7, Some("Exemplo"), Some("https://exemplo.com"));
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].source, "Exemplo");
        assert_eq!(items[0].source_url, "https://exemplo.com");
    }

    #[test]
    fn matches_two_term_query() {
        let terms = surface_tokens("reforma tributária");
        assert!(matches_query(
            "Senado aprova reforma tributária em primeiro turno",
            "reforma tributária",
            &terms
        ));
        assert!(!matches_query(
            "Senado aprova reforma política em primeiro turno",
            "reforma tributária",
            &terms
        ));
    }
}
