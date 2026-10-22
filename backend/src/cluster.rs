use std::collections::HashMap;

#[cfg(test)]
use crate::model::Lean;
use crate::model::{Article, BiasCounts, Cluster, FactCounts};
use crate::text::{build_vectors, cosine, display_tokens, tokens, top_surface_terms, DocVector};

#[derive(Clone, Debug)]
struct ClusterState {
    members: Vec<usize>,
    centroid: DocVector,
}

pub struct ClusterResult {
    pub clusters: Vec<Cluster>,
    pub assignments: Vec<usize>,
}

pub fn cluster_articles(articles: &[Article]) -> ClusterResult {
    if articles.is_empty() {
        return ClusterResult {
            clusters: Vec::new(),
            assignments: Vec::new(),
        };
    }

    let docs: Vec<Vec<String>> = articles
        .iter()
        .map(|article| {
            let mut terms = tokens(&article.title);
            terms.extend(tokens(article.snippet.as_deref().unwrap_or("")));
            terms
        })
        .collect();
    let (vectors, _idf) = build_vectors(&docs);

    let mut states: Vec<Option<ClusterState>> = vectors
        .iter()
        .enumerate()
        .map(|(index, vector)| {
            Some(ClusterState {
                members: vec![index],
                centroid: vector.clone(),
            })
        })
        .collect();

    let threshold = 0.22;

    loop {
        let mut best: Option<(usize, usize, f64)> = None;
        for i in 0..states.len() {
            if states[i].is_none() {
                continue;
            }
            for j in (i + 1)..states.len() {
                if states[j].is_none() {
                    continue;
                }
                let a = states[i].as_ref().unwrap();
                let b = states[j].as_ref().unwrap();
                let score = cosine(&a.centroid, &b.centroid);
                if score >= threshold && best.map(|(_, _, s)| score > s).unwrap_or(true) {
                    best = Some((i, j, score));
                }
            }
        }

        let Some((i, j, _)) = best else {
            break;
        };

        let b = states[j].take().unwrap();
        let a = states[i].as_mut().unwrap();
        let total = (a.members.len() + b.members.len()) as f64;
        let wa = a.members.len() as f64 / total;
        let wb = b.members.len() as f64 / total;

        let mut weights: HashMap<String, f64> = HashMap::new();
        for (term, value) in &a.centroid.weights {
            *weights.entry(term.clone()).or_insert(0.0) += value * wa;
        }
        for (term, value) in &b.centroid.weights {
            *weights.entry(term.clone()).or_insert(0.0) += value * wb;
        }
        let norm = weights
            .values()
            .map(|value| value * value)
            .sum::<f64>()
            .sqrt()
            .max(1e-9);
        for value in weights.values_mut() {
            *value /= norm;
        }

        a.members.extend(b.members);
        a.centroid = DocVector { weights };
    }

    let mut groups: Vec<ClusterState> = states.into_iter().flatten().collect();
    groups.sort_by_key(|group| std::cmp::Reverse(group.members.len()));

    let mut assignments = vec![0usize; articles.len()];
    let mut clusters = Vec::with_capacity(groups.len());

    for (cluster_id, group) in groups.iter().enumerate() {
        let mut lean_counts = BiasCounts::default();
        let mut factuality_counts = FactCounts::default();
        for member in &group.members {
            assignments[*member] = cluster_id;
            lean_counts.add(articles[*member].lean);
            factuality_counts.add(articles[*member].factuality);
        }

        let representative = group
            .members
            .iter()
            .max_by(|a, b| {
                cosine(&vectors[**a], &group.centroid)
                    .partial_cmp(&cosine(&vectors[**b], &group.centroid))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied()
            .unwrap_or(group.members[0]);

        let cluster_docs: Vec<Vec<String>> = group
            .members
            .iter()
            .map(|index| display_tokens(&articles[*index].title))
            .collect();

        clusters.push(Cluster {
            id: cluster_id,
            title: articles[representative].title.clone(),
            size: group.members.len(),
            article_ids: group
                .members
                .iter()
                .map(|index| articles[*index].id.clone())
                .collect(),
            lean_counts,
            factuality_counts,
            top_terms: top_surface_terms(&cluster_docs, 7),
        });
    }

    ClusterResult {
        clusters,
        assignments,
    }
}

pub fn apply_clusters(articles: &mut [Article], result: &ClusterResult) {
    for (index, article) in articles.iter_mut().enumerate() {
        if let Some(cluster_id) = result.assignments.get(index) {
            article.cluster_id = *cluster_id;
            article.cluster_title = result
                .clusters
                .get(*cluster_id)
                .map(|cluster| cluster.title.clone())
                .unwrap_or_default();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Factuality;

    fn article(id: &str, title: &str) -> Article {
        Article {
            id: id.to_string(),
            title: title.to_string(),
            url: format!("https://example.com/{id}"),
            source: "Exemplo".to_string(),
            domain: "example.com".to_string(),
            published_at: None,
            lean: Lean::Center,
            factuality: Factuality::High,
            ownership: None,
            rating_source: "demo".to_string(),
            cluster_id: 0,
            cluster_title: String::new(),
            snippet: None,
            relevance: 0.0,
            deep_read: false,
            excerpt: None,
        }
    }

    #[test]
    fn groups_same_event_titles() {
        let articles = vec![
            article("1", "Senado aprova reforma tributária em primeiro turno"),
            article("2", "Reforma tributária passa no Senado em primeiro turno"),
            article("3", "Senado aprova texto-base da reforma tributária"),
            article("4", "Futebol: time vence clássico no domingo"),
        ];
        let result = cluster_articles(&articles);
        assert!(result.clusters.len() >= 2);
        assert!(result.clusters.iter().any(|cluster| cluster.size >= 3));
    }
}
