use std::collections::{BTreeMap, HashMap, HashSet};

use chrono::{DateTime, NaiveDate, Utc};

use crate::catalog::{domain_from_url, Catalog};
use crate::cluster::{apply_clusters, cluster_articles};
use crate::fetch::RawArticle;
use crate::fulltext;
use crate::llm::{self, LlmConfig};
use crate::model::{
    Analysis, Article, BiasCounts, Cluster, FactCounts, Lean, ProgressFn, ProgressUpdate,
    ScatterPoint, SourceCount, Stats, TimelinePoint,
};
use crate::report;
use crate::text::{hash_id, normalize_query};

pub async fn build(
    query: &str,
    window_days: u32,
    raw: Vec<RawArticle>,
    catalog: &Catalog,
    llm_config: Option<&LlmConfig>,
    progress: Option<ProgressFn>,
) -> Analysis {
    let generated_at = Utc::now().to_rfc3339();
    let mut articles: Vec<Article> = raw
        .into_iter()
        .map(|item| {
            let domain = domain_from_url(&item.source_url);
            let rating = catalog.lookup(&item.source, &item.source_url);
            Article {
                id: hash_id(&format!("{}|{}", item.url, item.title)),
                title: item.title,
                url: item.url,
                source: item.source,
                domain,
                published_at: item.published_at,
                lean: rating.lean,
                factuality: rating.factuality,
                ownership: rating.ownership,
                rating_source: rating.rating_source,
                cluster_id: 0,
                cluster_title: String::new(),
                snippet: item.snippet,
                relevance: 0.0,
                deep_read: false,
                excerpt: None,
            }
        })
        .collect();

    articles.retain(|article| !is_junk_title(&article.title));
    articles.sort_by(|a, b| b.published_at.cmp(&a.published_at));
    articles.dedup_by(|a, b| a.title.eq_ignore_ascii_case(&b.title) && a.domain == b.domain);

    notify(
        &progress,
        ProgressUpdate {
            step: 2,
            label: "Avaliando as fontes".to_string(),
            detail: format!(
                "{} artigos de {} veículos",
                articles.len(),
                unique_sources(&articles)
            ),
            progress: 28.0,
            current: Some(articles.len()),
            total: None,
        },
    );

    let cluster_result = cluster_articles(&articles);
    apply_clusters(&mut articles, &cluster_result);
    score_relevance(&mut articles, &cluster_result.clusters);

    notify(
        &progress,
        ProgressUpdate {
            step: 3,
            label: "Agrupando o mesmo fato".to_string(),
            detail: format!("{} grupos de manchetes", cluster_result.clusters.len()),
            progress: 42.0,
            current: Some(cluster_result.clusters.len()),
            total: None,
        },
    );

    let deep_read =
        fulltext::enrich(&mut articles, fulltext::max_articles(), progress.as_ref()).await;

    let stats = build_stats(
        query,
        window_days,
        &articles,
        cluster_result.clusters.len(),
        deep_read,
    );
    let timeline = build_timeline(window_days, &articles);
    let scatter = build_scatter(&articles);

    let dek = format!(
        "{} artigos · {} veículos · últimos {} dias",
        stats.total_articles, stats.total_sources, window_days
    );

    notify(
        &progress,
        ProgressUpdate {
            step: 5,
            label: "Redigindo o relance".to_string(),
            detail: if llm_config.is_some() {
                format!(
                    "{} matérias lidas; modelo generativo com citações",
                    deep_read
                )
            } else {
                format!("{} matérias lidas; síntese extrativa", deep_read)
            },
            progress: 82.0,
            current: None,
            total: None,
        },
    );

    let mut mode = "extractive".to_string();
    let mut report = report::generate(
        query,
        &articles,
        &cluster_result.clusters,
        &stats,
        "extractive",
    );

    if let Some(config) = llm_config {
        let extractive = report.clone();
        match llm::generate(config, query, &articles, &stats, &dek).await {
            Ok(mut llm_report) => {
                // Se o modelo omitir a seção de ausências ou o callout, completa com a versão
                // extrativa. O usuário nunca deve perder a parte mais importante do produto.
                if let Some(extractive_section) = extractive
                    .sections
                    .iter()
                    .find(|section| section.heading.to_lowercase().contains("ficou de fora"))
                    .cloned()
                {
                    match llm_report
                        .sections
                        .iter_mut()
                        .find(|section| section.heading.to_lowercase().contains("ficou de fora"))
                    {
                        Some(section) => {
                            if section.callout.is_none() {
                                section.callout = extractive_section.callout;
                            }
                        }
                        None => {
                            let insert_at = llm_report
                                .sections
                                .iter()
                                .position(|section| {
                                    section.heading.to_lowercase().contains("limita")
                                })
                                .unwrap_or(llm_report.sections.len());
                            llm_report.sections.insert(insert_at, extractive_section);
                        }
                    }
                }

                for limitation in &extractive.limitations {
                    if !llm_report.limitations.contains(limitation) {
                        llm_report.limitations.push(limitation.clone());
                    }
                }

                mode = "llm".to_string();
                report = llm_report;
            }
            Err(error) => {
                tracing::warn!(%error, "LLM indisponível; usando relance extrativo");
                report.limitations.push(
                    "O modelo generativo não respondeu; o relance exibido é extrativo.".to_string(),
                );
            }
        }
    }

    notify(
        &progress,
        ProgressUpdate {
            step: 6,
            label: "Conferindo citações".to_string(),
            detail: "Validando referências e números do relance".to_string(),
            progress: 94.0,
            current: None,
            total: None,
        },
    );

    let limitations = report.limitations.clone();

    Analysis {
        query: normalize_query(query),
        generated_at,
        mode,
        cached: false,
        stats,
        articles,
        clusters: cluster_result.clusters,
        report,
        timeline,
        scatter,
        limitations,
    }
}

fn build_stats(
    _query: &str,
    window_days: u32,
    articles: &[Article],
    cluster_count: usize,
    deep_read: usize,
) -> Stats {
    let mut bias = BiasCounts::default();
    let mut factuality = FactCounts::default();
    let mut source_counts: HashMap<String, SourceCount> = HashMap::new();
    let mut domains = HashSet::new();

    for article in articles {
        bias.add(article.lean);
        factuality.add(article.factuality);
        domains.insert(article.domain.clone());
        let entry = source_counts
            .entry(article.domain.clone())
            .or_insert_with(|| SourceCount {
                name: article.source.clone(),
                domain: article.domain.clone(),
                count: 0,
                lean: article.lean,
                factuality: article.factuality,
            });
        entry.count += 1;
    }

    let mut top_sources: Vec<SourceCount> = source_counts.into_values().collect();
    top_sources.sort_by_key(|source| std::cmp::Reverse(source.count));
    top_sources.truncate(12);

    let unknown_articles = articles
        .iter()
        .filter(|article| article.lean == Lean::Unknown)
        .count();

    let dates: Vec<&String> = articles
        .iter()
        .filter_map(|article| article.published_at.as_ref())
        .collect();

    Stats {
        total_articles: articles.len(),
        total_sources: domains.len(),
        rated_articles: articles.len().saturating_sub(unknown_articles),
        unknown_articles,
        bias,
        factuality,
        window_days,
        date_min: dates.iter().min().map(|value| (*value).clone()),
        date_max: dates.iter().max().map(|value| (*value).clone()),
        top_sources,
        clusters: cluster_count,
        deep_read,
    }
}

fn notify(progress: &Option<ProgressFn>, update: ProgressUpdate) {
    if let Some(callback) = progress {
        callback(update);
    }
}

fn unique_sources(articles: &[Article]) -> usize {
    articles
        .iter()
        .map(|article| article.domain.as_str())
        .collect::<HashSet<_>>()
        .len()
}

fn score_relevance(articles: &mut [Article], clusters: &[Cluster]) {
    let sizes: HashMap<usize, usize> = clusters
        .iter()
        .map(|cluster| (cluster.id, cluster.size))
        .collect();
    let now = Utc::now();

    for article in articles {
        let cluster_size = sizes.get(&article.cluster_id).copied().unwrap_or(1) as f64;
        let cluster_score = ((cluster_size + 1.0).ln() / 4.6).min(1.0);
        let recency = article
            .published_at
            .as_deref()
            .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
            .map(|date| {
                let hours = (now - date.with_timezone(&Utc)).num_hours().max(0) as f64;
                (1.0 - hours / 168.0).clamp(0.0, 1.0)
            })
            .unwrap_or(0.3);
        let lean_bonus = if article.lean == Lean::Unknown {
            0.0
        } else {
            0.15
        };
        let fact_bonus = match article.factuality {
            crate::model::Factuality::High => 0.12,
            crate::model::Factuality::Mixed => 0.06,
            _ => 0.0,
        };
        let title_bonus =
            ((article.title.chars().count() as f64 - 20.0) / 100.0).clamp(0.0, 1.0) * 0.08;
        let direct_bonus = if article.url.contains("news.google.com") {
            0.0
        } else {
            0.06
        };
        article.relevance = (0.42 * cluster_score
            + 0.25 * recency
            + lean_bonus
            + fact_bonus
            + title_bonus
            + direct_bonus)
            .clamp(0.0, 1.0);
    }
}

fn build_timeline(window_days: u32, articles: &[Article]) -> Vec<TimelinePoint> {
    let today = Utc::now().date_naive();
    let start = today - chrono::Duration::days(window_days.saturating_sub(1) as i64);
    let mut buckets: BTreeMap<NaiveDate, TimelinePoint> = BTreeMap::new();

    let mut day = start;
    while day <= today {
        buckets.insert(
            day,
            TimelinePoint {
                date: day.format("%d/%m").to_string(),
                left: 0,
                center: 0,
                right: 0,
                unknown: 0,
            },
        );
        day += chrono::Duration::days(1);
    }

    for article in articles {
        let Some(value) = article.published_at.as_deref() else {
            continue;
        };
        let Ok(parsed) = DateTime::parse_from_rfc3339(value) else {
            continue;
        };
        let date = parsed.date_naive();
        if let Some(point) = buckets.get_mut(&date) {
            match article.lean {
                Lean::Left => point.left += 1,
                Lean::Center => point.center += 1,
                Lean::Right => point.right += 1,
                Lean::Unknown => point.unknown += 1,
            }
        }
    }

    buckets.into_values().collect()
}

fn build_scatter(articles: &[Article]) -> Vec<ScatterPoint> {
    let mut by_source: HashMap<String, ScatterPoint> = HashMap::new();
    for article in articles {
        if article.lean == Lean::Unknown {
            continue;
        }
        let entry = by_source
            .entry(article.domain.clone())
            .or_insert_with(|| ScatterPoint {
                name: article.source.clone(),
                domain: article.domain.clone(),
                lean: article.lean,
                factuality: article.factuality,
                count: 0,
            });
        entry.count += 1;
    }
    let mut points: Vec<ScatterPoint> = by_source.into_values().collect();
    points.sort_by_key(|point| std::cmp::Reverse(point.count));
    points.truncate(40);
    points
}

fn is_junk_title(title: &str) -> bool {
    let lower = title.to_lowercase();
    let junk_fragments = [
        "whatsappimage",
        "whatsapp image",
        ".jpeg",
        ".jpg",
        ".png",
        "notícias sobre",
        "noticias sobre",
        "archives",
        "arquivos",
        " - página ",
        " - pagina ",
        "últimas notícias",
        "ultimas noticias",
    ];
    junk_fragments
        .iter()
        .any(|fragment| lower.contains(fragment))
        || lower.matches(" | ").count() > 3
}
