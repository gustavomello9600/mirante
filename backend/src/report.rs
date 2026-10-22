use std::collections::HashMap;

use crate::model::{
    Article, Callout, Cluster, Lean, Report, ReportParagraph, ReportSection, Stats,
};
use crate::text::{display_tokens, human_terms, top_surface_terms};

pub fn generate(
    query: &str,
    articles: &[Article],
    clusters: &[Cluster],
    stats: &Stats,
    mode: &str,
) -> Report {
    let total = articles.len();
    let unknown_share = if total == 0 {
        0.0
    } else {
        stats.unknown_articles as f64 / total as f64 * 100.0
    };

    let all_docs: Vec<Vec<String>> = articles
        .iter()
        .map(|article| display_tokens(&article.title))
        .collect();
    let global_terms = top_surface_terms(&all_docs, 5);

    let mut sections = Vec::new();
    sections.push(what_happened(
        query,
        articles,
        clusters,
        stats,
        &global_terms,
    ));
    sections.push(what_is_missing(articles, clusters, &all_docs));

    let limitations = limitations(stats, unknown_share);
    let mut limitations_body = Vec::new();
    if let Some(first) = limitations.first() {
        limitations_body.push(ReportParagraph {
            text: first.clone(),
            citations: Vec::new(),
            label: None,
        });
    }
    sections.push(ReportSection {
        heading: "Limitações".to_string(),
        body: limitations_body,
        callout: None,
    });

    let dek = format!(
        "{} · {} · últimos {} dias",
        if stats.total_articles == 1 {
            "1 artigo".to_string()
        } else {
            format!("{} artigos", stats.total_articles)
        },
        if stats.total_sources == 1 {
            "1 veículo".to_string()
        } else {
            format!("{} veículos", stats.total_sources)
        },
        stats.window_days
    );

    Report {
        title: "O relance".to_string(),
        dek,
        feature: None,
        sections,
        limitations,
        generated_by: mode.to_string(),
    }
}

fn what_happened(
    query: &str,
    articles: &[Article],
    clusters: &[Cluster],
    stats: &Stats,
    global_terms: &[String],
) -> ReportSection {
    let mut body = Vec::new();
    let terms = if global_terms.is_empty() {
        "os temas principais da amostra".to_string()
    } else {
        human_terms(&global_terms[..global_terms.len().min(4)])
    };

    let lead = format!(
        "A cobertura de “{}” reúne {} artigos de {} veículos nos últimos {} dias. As manchetes giram em torno de {}.",
        query,
        stats.total_articles,
        stats.total_sources,
        stats.window_days,
        terms
    );

    let lead_citations = representative_citations(articles, 3);
    body.push(ReportParagraph {
        text: lead,
        citations: lead_citations,
        label: None,
    });

    if let Some(cluster) = clusters.first() {
        if cluster.size >= 2 {
            let citation = article_index(articles, &cluster.article_ids[0]);
            let text = format!(
                "O maior grupo de manchetes semelhantes tem {} artigos e trata de “{}”. Isso não significa que seja a versão correta: significa que foi o enquadramento mais repetido na amostra.",
                cluster.size, cluster.title
            );
            body.push(ReportParagraph {
                text,
                citations: citation.into_iter().collect(),
                label: None,
            });
        }
    }

    let concentration = concentration_sentence(stats);
    if !concentration.is_empty() {
        body.push(ReportParagraph {
            text: concentration,
            citations: Vec::new(),
            label: None,
        });
    }

    ReportSection {
        heading: "O que aconteceu".to_string(),
        body,
        callout: None,
    }
}

fn what_is_missing(
    articles: &[Article],
    clusters: &[Cluster],
    all_docs: &[Vec<String>],
) -> ReportSection {
    let mut body = Vec::new();

    if let Some((cluster, covered, missing, count)) = asymmetric_cluster(clusters) {
        let citation_indices: Vec<usize> = cluster
            .article_ids
            .iter()
            .filter_map(|id| article_index(articles, id))
            .take(4)
            .collect();
        let title = format!(
            "“{}” apareceu em {} artigos de {}, mas em nenhum de {}.",
            cluster.title,
            count,
            covered.label(),
            missing.label()
        );
        let text = "A ausência é medida só entre veículos com avaliação de viés. Pode ser uma escolha editorial, um limite da amostra ou uma diferença de timing — o relance não decide qual das três.".to_string();
        body.push(ReportParagraph {
            text: text.clone(),
            citations: citation_indices,
            label: Some("Assimetria de cobertura".to_string()),
        });
        return ReportSection {
            heading: "O que ficou de fora".to_string(),
            body,
            callout: Some(Callout {
                label: "Ângulo subrepresentado".to_string(),
                title,
                body: text,
            }),
        };
    }

    if let Some((term, count_a, lean_a, _count_b, lean_b)) = absent_term(articles, all_docs) {
        let text = format!(
            "O termo “{}” aparece em {} artigos de {} e em nenhum de {}. É um indício de enquadramento, não uma prova de intenção editorial.",
            term,
            count_a,
            lean_a.label(),
            lean_b.label()
        );
        body.push(ReportParagraph {
            text: text.clone(),
            citations: term_citations(articles, &term, lean_a),
            label: Some("Termo ausente".to_string()),
        });
        return ReportSection {
            heading: "O que ficou de fora".to_string(),
            body,
            callout: Some(Callout {
                label: "Diferença de vocabulário".to_string(),
                title: format!(
                    "“{}”: {} artigos de {}, nenhum de {}.",
                    term,
                    count_a,
                    lean_a.label(),
                    lean_b.label()
                ),
                body: text,
            }),
        };
    }

    let text = format!(
        "Com {} artigos, não foi possível identificar uma ausência robusta entre os grupos. Isso não significa cobertura equilibrada; significa que a amostra não sustenta a afirmação. Tente ampliar a janela ou o tema.",
        articles.len()
    );
    body.push(ReportParagraph {
        text: text.clone(),
        citations: Vec::new(),
        label: Some("Amostra limitada".to_string()),
    });

    ReportSection {
        heading: "O que ficou de fora".to_string(),
        body,
        callout: Some(Callout {
            label: "Sem ausência robusta".to_string(),
            title: "A amostra não sustenta uma conclusão sobre ausência.".to_string(),
            body: text,
        }),
    }
}

fn limitations(stats: &Stats, unknown_share: f64) -> Vec<String> {
    let mut out = Vec::new();
    out.push(format!(
        "{:.0}% dos artigos vêm de veículos sem avaliação de viés; eles ficam fora das barras coloridas.",
        unknown_share
    ));
    if stats.deep_read > 0 {
        let verb = if stats.deep_read == 1 {
            "matéria foi lida"
        } else {
            "matérias foram lidas"
        };
        out.push(format!(
            "{} {verb} na íntegra para enriquecer o relance; as demais usaram título e metadados.",
            stats.deep_read
        ));
    } else {
        out.push(
            "Nenhuma matéria foi lida na íntegra nesta execução; o relance usa títulos e metadados."
                .to_string(),
        );
    }
    out.push(format!(
        "A amostra tem {} de {}; não representa toda a produção jornalística sobre o tema.",
        if stats.total_articles == 1 {
            "1 artigo".to_string()
        } else {
            format!("{} artigos", stats.total_articles)
        },
        if stats.total_sources == 1 {
            "1 veículo".to_string()
        } else {
            format!("{} veículos", stats.total_sources)
        }
    ));
    if stats.deep_read > 0 {
        out.push(
            "O Mirante usa manchetes, metadados e trechos extraídos; não reproduz o texto completo das matérias."
                .to_string(),
        );
    } else {
        out.push(
            "O Mirante usa manchetes e metadados públicos; não reproduz o texto completo das matérias."
                .to_string(),
        );
    }
    out.push(
        "As avaliações de viés e factualidade são uma curadoria de demonstração; em produção, substitua por uma base licenciada ou por metodologia própria auditável."
            .to_string(),
    );
    out
}

fn concentration_sentence(stats: &Stats) -> String {
    let Some(top) = stats.top_sources.first() else {
        return String::new();
    };
    if stats.total_articles == 0 {
        return String::new();
    }
    let share = top.count as f64 / stats.total_articles as f64 * 100.0;
    format!(
        "O veículo mais frequente é {}, com {} artigos ({:.0}% da amostra).",
        top.name, top.count, share
    )
}

fn representative_citations(articles: &[Article], limit: usize) -> Vec<usize> {
    articles
        .iter()
        .take(limit)
        .filter_map(|article| article_index(articles, &article.id))
        .collect()
}

fn article_index(articles: &[Article], id: &str) -> Option<usize> {
    articles
        .iter()
        .position(|article| article.id == id)
        .map(|index| index + 1)
}

fn asymmetric_cluster(clusters: &[Cluster]) -> Option<(&Cluster, Lean, Lean, usize)> {
    let mut best: Option<(&Cluster, Lean, Lean, usize, usize)> = None;
    for cluster in clusters.iter().filter(|cluster| cluster.size >= 3) {
        for (covered, missing) in [
            (Lean::Left, Lean::Right),
            (Lean::Right, Lean::Left),
            (Lean::Center, Lean::Right),
            (Lean::Center, Lean::Left),
        ] {
            let covered_count = match covered {
                Lean::Left => cluster.lean_counts.left,
                Lean::Center => cluster.lean_counts.center,
                Lean::Right => cluster.lean_counts.right,
                Lean::Unknown => cluster.lean_counts.unknown,
            };
            let missing_count = match missing {
                Lean::Left => cluster.lean_counts.left,
                Lean::Center => cluster.lean_counts.center,
                Lean::Right => cluster.lean_counts.right,
                Lean::Unknown => cluster.lean_counts.unknown,
            };
            if covered_count >= 2 && missing_count == 0 {
                let score = cluster.size;
                if best.map(|(_, _, _, _, s)| score > s).unwrap_or(true) {
                    best = Some((cluster, covered, missing, covered_count, score));
                }
            }
        }
    }
    best.map(|(cluster, covered, missing, count, _)| (cluster, covered, missing, count))
}

fn absent_term(
    articles: &[Article],
    all_docs: &[Vec<String>],
) -> Option<(String, usize, Lean, usize, Lean)> {
    let mut best: Option<(String, usize, Lean, usize, Lean, usize)> = None;
    for (lean_a, lean_b) in [
        (Lean::Left, Lean::Right),
        (Lean::Right, Lean::Left),
        (Lean::Center, Lean::Right),
        (Lean::Center, Lean::Left),
    ] {
        let docs_a: Vec<Vec<String>> = articles
            .iter()
            .filter(|article| article.lean == lean_a)
            .map(|article| display_tokens(&article.title))
            .collect();
        if docs_a.len() < 3 {
            continue;
        }
        let docs_b: Vec<Vec<String>> = articles
            .iter()
            .filter(|article| article.lean == lean_b)
            .map(|article| display_tokens(&article.title))
            .collect();
        if docs_b.is_empty() {
            continue;
        }
        let mut counts_a: HashMap<String, usize> = HashMap::new();
        for doc in &docs_a {
            for term in doc {
                *counts_a.entry(term.clone()).or_insert(0) += 1;
            }
        }
        let mut counts_b: HashMap<String, usize> = HashMap::new();
        for doc in &docs_b {
            for term in doc {
                *counts_b.entry(term.clone()).or_insert(0) += 1;
            }
        }
        for (term, count_a) in counts_a {
            if count_a >= 2
                && !counts_b.contains_key(&term)
                && best
                    .as_ref()
                    .map(|(_, c, _, _, _, _)| count_a > *c)
                    .unwrap_or(true)
            {
                best = Some((term, count_a, lean_a, 0, lean_b, count_a));
            }
        }
    }
    let _ = all_docs;
    best.map(|(term, count_a, lean_a, count_b, lean_b, _)| (term, count_a, lean_a, count_b, lean_b))
}

fn term_citations(articles: &[Article], term: &str, lean: Lean) -> Vec<usize> {
    articles
        .iter()
        .enumerate()
        .filter(|(_, article)| article.lean == lean)
        .filter(|(_, article)| {
            display_tokens(&article.title)
                .iter()
                .any(|token| token == term)
        })
        .map(|(index, _)| index + 1)
        .take(3)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{BiasCounts, FactCounts, Factuality};

    fn sample_articles() -> Vec<Article> {
        let mut out = Vec::new();
        for (index, (lean, title)) in [
            (
                Lean::Left,
                "Trabalhadores criticam impacto da reforma tributária",
            ),
            (
                Lean::Left,
                "Reforma tributária pode aumentar custo para famílias",
            ),
            (
                Lean::Right,
                "Reforma tributária aumenta carga sobre empresas",
            ),
            (
                Lean::Right,
                "Empresas alertam para custo da reforma tributária",
            ),
            (
                Lean::Center,
                "Senado aprova reforma tributária em primeiro turno",
            ),
        ]
        .into_iter()
        .enumerate()
        {
            out.push(Article {
                id: format!("a{}", index),
                title: title.to_string(),
                url: format!("https://example.com/{index}"),
                source: "Fonte".to_string(),
                domain: "example.com".to_string(),
                published_at: None,
                lean,
                factuality: Factuality::High,
                ownership: None,
                rating_source: "demo".to_string(),
                cluster_id: 0,
                cluster_title: String::new(),
                snippet: None,
                relevance: 0.0,
                deep_read: false,
                excerpt: None,
            });
        }
        out
    }

    #[test]
    fn report_has_citations_and_limitations() {
        let articles = sample_articles();
        let stats = Stats {
            total_articles: articles.len(),
            total_sources: 1,
            rated_articles: articles.len(),
            unknown_articles: 0,
            bias: BiasCounts {
                left: 2,
                center: 1,
                right: 2,
                unknown: 0,
            },
            factuality: FactCounts {
                high: 5,
                mixed: 0,
                low: 0,
                unknown: 0,
            },
            window_days: 7,
            date_min: None,
            date_max: None,
            top_sources: vec![],
            clusters: 1,
            deep_read: 0,
        };
        let clusters = vec![Cluster {
            id: 0,
            title: "Senado aprova reforma tributária".to_string(),
            size: 5,
            article_ids: articles.iter().map(|a| a.id.clone()).collect(),
            lean_counts: stats.bias.clone(),
            factuality_counts: stats.factuality.clone(),
            top_terms: vec!["reforma".to_string()],
        }];
        let report = generate(
            "reforma tributária",
            &articles,
            &clusters,
            &stats,
            "extractive",
        );
        assert!(report
            .sections
            .iter()
            .any(|s| s.heading == "O que aconteceu"));
        assert!(report.limitations.len() >= 3);
        assert!(report
            .sections
            .iter()
            .flat_map(|s| &s.body)
            .any(|p| !p.citations.is_empty()));
    }
}
