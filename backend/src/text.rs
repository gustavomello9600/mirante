use once_cell::sync::Lazy;
use rust_stemmers::{Algorithm, Stemmer};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};

static STEMMER: Lazy<Stemmer> = Lazy::new(|| Stemmer::create(Algorithm::Portuguese));

static STOPWORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "a", "o", "as", "os", "um", "uma", "uns", "umas", "de", "do", "da", "dos", "das", "em",
        "no", "na", "nos", "nas", "por", "pelo", "pela", "pelos", "pelas", "para", "pra", "com",
        "sem", "sob", "sobre", "entre", "e", "ou", "mas", "que", "se", "como", "quando", "onde",
        "porque", "porquê", "ao", "aos", "à", "às", "é", "são", "foi", "foram", "ser", "estar",
        "está", "estão", "já", "não", "sim", "mais", "menos", "muito", "muita", "muitos", "muitas",
        "seu", "sua", "seus", "suas", "este", "esta", "estes", "estas", "esse", "essa", "esses",
        "essas", "isso", "isto", "aquele", "aquela", "aqueles", "aquelas", "ele", "ela", "eles",
        "elas", "eu", "nós", "você", "vocês", "lhe", "lhes", "me", "te", "nos", "vos", "também",
        "após", "antes", "até", "desde", "contra", "entre", "sem", "durante", "pode", "podem",
        "pode", "será", "serão", "tem", "têm", "ter", "havia", "há", "ano", "anos", "dia", "dias",
        "semana", "semanas", "mês", "meses", "hoje", "ontem", "amanhã", "agora", "ainda", "sobre",
        "the", "and", "for", "with", "from", "this", "that", "are", "was", "were", "has", "have",
        "had", "not", "but", "you", "your", "its", "it", "as", "at", "by", "on", "in", "of", "to",
        "is", "be", "or", "an", "a", "new", "said", "says", "after", "before", "over", "under",
        "will", "would", "could", "should", "can", "may", "more", "most", "than", "then", "they",
        "their", "them", "we", "our", "us", "he", "she", "his", "her", "who", "what", "why", "how",
    ]
    .into_iter()
    .collect()
});

pub fn clean_text(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut in_tag = false;
    for ch in value.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out.replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn strip_source_suffix(title: &str, source: &str) -> String {
    let title = clean_text(title);
    let source = clean_text(source);
    if let Some((head, tail)) = title.rsplit_once(" - ") {
        if !source.is_empty() && tail.trim().eq_ignore_ascii_case(&source) {
            return head.trim().to_string();
        }
    }
    if let Some((head, tail)) = title.rsplit_once(" | ") {
        if !source.is_empty() && tail.trim().eq_ignore_ascii_case(&source) {
            return head.trim().to_string();
        }
    }
    title
}

fn fold(value: &str) -> String {
    deunicode::deunicode(&value.to_lowercase())
}

pub fn tokens(value: &str) -> Vec<String> {
    let folded = fold(value);
    let mut out = Vec::new();
    for raw in folded.split(|c: char| !c.is_alphanumeric()) {
        if raw.len() < 3 || raw.len() > 40 {
            continue;
        }
        if STOPWORDS.contains(raw) {
            continue;
        }
        if raw.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        let stemmed = STEMMER.stem(raw).to_string();
        if stemmed.len() >= 3 && !STOPWORDS.contains(stemmed.as_str()) {
            out.push(stemmed);
        } else {
            out.push(raw.to_string());
        }
    }
    out
}

pub fn normalize_query(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_lowercase()
}

pub fn hash_id(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    hex::encode(&digest[..8])
}

#[derive(Clone, Debug, Default)]
pub struct DocVector {
    pub weights: HashMap<String, f64>,
}

pub fn build_vectors(docs: &[Vec<String>]) -> (Vec<DocVector>, HashMap<String, f64>) {
    let n = docs.len() as f64;
    let mut df: HashMap<String, usize> = HashMap::new();
    for doc in docs {
        let mut seen = HashSet::new();
        for term in doc {
            if seen.insert(term.clone()) {
                *df.entry(term.clone()).or_insert(0) += 1;
            }
        }
    }

    let idf: HashMap<String, f64> = df
        .iter()
        .map(|(term, count)| {
            let value = ((1.0 + n) / (1.0 + *count as f64)).ln() + 1.0;
            (term.clone(), value)
        })
        .collect();

    let vectors = docs
        .iter()
        .map(|doc| {
            let mut tf: HashMap<String, f64> = HashMap::new();
            let total = doc.len().max(1) as f64;
            for term in doc {
                *tf.entry(term.clone()).or_insert(0.0) += 1.0 / total;
            }
            let mut weights = HashMap::new();
            let mut norm_sq = 0.0;
            for (term, value) in tf {
                let weight = value * idf.get(&term).copied().unwrap_or(1.0);
                norm_sq += weight * weight;
                weights.insert(term, weight);
            }
            let norm = norm_sq.sqrt().max(1e-9);
            for value in weights.values_mut() {
                *value /= norm;
            }
            DocVector { weights }
        })
        .collect();

    (vectors, idf)
}

pub fn cosine(a: &DocVector, b: &DocVector) -> f64 {
    if a.weights.len() > b.weights.len() {
        return cosine(b, a);
    }
    a.weights
        .iter()
        .map(|(term, value)| value * b.weights.get(term).copied().unwrap_or(0.0))
        .sum()
}

pub fn display_tokens(value: &str) -> Vec<String> {
    let lower = value.to_lowercase();
    let mut out = Vec::new();
    for raw in lower.split(|c: char| !c.is_alphanumeric()) {
        if raw.len() < 3 || raw.len() > 40 {
            continue;
        }
        if STOPWORDS.contains(raw) || raw.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        out.push(raw.to_string());
    }
    out
}

pub fn surface_tokens(value: &str) -> Vec<String> {
    let folded = fold(value);
    let mut out = Vec::new();
    for raw in folded.split(|c: char| !c.is_alphanumeric()) {
        if raw.len() < 3 || raw.len() > 40 {
            continue;
        }
        if STOPWORDS.contains(raw) || raw.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        out.push(raw.to_string());
    }
    out
}

pub fn top_surface_terms(docs: &[Vec<String>], limit: usize) -> Vec<String> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for doc in docs {
        for term in doc {
            *counts.entry(term.clone()).or_insert(0) += 1;
        }
    }
    let mut ranked: Vec<(String, usize)> = counts.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    ranked
        .into_iter()
        .take(limit)
        .map(|(term, _)| term)
        .collect()
}

pub fn human_terms(terms: &[String]) -> String {
    match terms.len() {
        0 => String::new(),
        1 => terms[0].clone(),
        2 => format!("{} e {}", terms[0], terms[1]),
        _ => format!(
            "{}, {} e {}",
            terms[0],
            terms[1..terms.len() - 1].join(", "),
            terms[terms.len() - 1]
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_google_source_suffix() {
        assert_eq!(
            strip_source_suffix("Nova regra avança no Senado - Folha", "Folha"),
            "Nova regra avança no Senado"
        );
    }

    #[test]
    fn tokenizes_portuguese_without_stopwords() {
        let tokens = tokens("A regulação da inteligência artificial no Brasil");
        assert!(tokens.iter().any(|t| t.starts_with("regul")));
        assert!(!tokens.iter().any(|t| t == "a" || t == "da" || t == "no"));
    }

    #[test]
    fn identical_vectors_have_similarity_one() {
        let docs = vec![
            tokens("reforma tributária avança no senado"),
            tokens("reforma tributária avança no senado"),
        ];
        let (vectors, _) = build_vectors(&docs);
        assert!((cosine(&vectors[0], &vectors[1]) - 1.0).abs() < 1e-6);
    }
}
