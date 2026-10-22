use crate::model::{Factuality, Lean, SourceInfo};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize)]
struct CatalogFile {
    sources: Vec<SourceSeed>,
}

#[derive(Clone, Debug, Deserialize)]
struct SourceSeed {
    name: String,
    domain: String,
    lean: Lean,
    factuality: Factuality,
    ownership: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Rating {
    pub lean: Lean,
    pub factuality: Factuality,
    pub ownership: Option<String>,
    pub rating_source: String,
}

#[derive(Clone, Debug)]
pub struct Catalog {
    seeds: Vec<SourceSeed>,
    by_name: HashMap<String, usize>,
}

impl Catalog {
    pub fn load() -> Self {
        let file: CatalogFile = serde_json::from_str(include_str!("../data/sources.json"))
            .expect("sources.json válido");
        let mut by_name = HashMap::new();
        for (index, source) in file.sources.iter().enumerate() {
            by_name.insert(source.name.to_lowercase(), index);
        }
        let mut seeds = file.sources;
        seeds.sort_by_key(|seed| std::cmp::Reverse(seed.domain.len()));
        Self { seeds, by_name }
    }

    pub fn len(&self) -> usize {
        self.seeds.len()
    }

    pub fn sources(&self) -> Vec<SourceInfo> {
        let mut out: Vec<SourceInfo> = self
            .seeds
            .iter()
            .map(|seed| SourceInfo {
                name: seed.name.clone(),
                domain: seed.domain.clone(),
                lean: seed.lean,
                factuality: seed.factuality,
                ownership: seed.ownership.clone(),
                rating_source: "Curadoria demo do Mirante".to_string(),
            })
            .collect();
        out.sort_by_key(|source| source.name.to_lowercase());
        out
    }

    pub fn lookup(&self, name: &str, url: &str) -> Rating {
        let domain = domain_from_url(url);
        let normalized_name = name.trim().to_lowercase();
        let normalized_name = normalized_name
            .strip_suffix(" on msn")
            .unwrap_or(&normalized_name)
            .trim()
            .to_string();

        if let Some(seed) = self
            .seeds
            .iter()
            .find(|seed| !domain.is_empty() && domain_matches(&domain, &seed.domain))
        {
            return rating_from_seed(seed);
        }

        if let Some(index) = self.by_name.get(&normalized_name) {
            return rating_from_seed(&self.seeds[*index]);
        }

        Rating {
            lean: Lean::Unknown,
            factuality: Factuality::Unknown,
            ownership: None,
            rating_source: "Sem base de avaliação".to_string(),
        }
    }
}

fn rating_from_seed(seed: &SourceSeed) -> Rating {
    Rating {
        lean: seed.lean,
        factuality: seed.factuality,
        ownership: seed.ownership.clone(),
        rating_source: "Curadoria demo do Mirante".to_string(),
    }
}

pub fn domain_from_url(url: &str) -> String {
    let without_scheme = url.split_once("://").map(|(_, rest)| rest).unwrap_or(url);
    let host = without_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or("")
        .split(':')
        .next()
        .unwrap_or("")
        .trim()
        .to_lowercase();
    host.strip_prefix("www1.")
        .or_else(|| host.strip_prefix("www."))
        .unwrap_or(&host)
        .to_string()
}

fn domain_matches(actual: &str, expected: &str) -> bool {
    actual == expected || actual.ends_with(&format!(".{expected}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_subdomain() {
        assert!(domain_matches("www1.folha.uol.com.br", "folha.uol.com.br"));
        assert!(domain_matches("g1.globo.com", "g1.globo.com"));
        assert!(!domain_matches("oglobo.globo.com", "g1.globo.com"));
    }

    #[test]
    fn matches_syndicated_source_name() {
        let catalog = Catalog::load();
        let rating = catalog.lookup("Estadão on MSN", "https://msn.com/noticia");
        assert_eq!(rating.lean, Lean::Center);
        assert_eq!(rating.factuality, Factuality::High);
    }

    #[test]
    fn catalog_covers_regional_and_specialist_media() {
        let catalog = Catalog::load();
        assert!(catalog.len() >= 150);
        for name in [
            "GZH",
            "Correio do Povo",
            "Brasil 247",
            "Brasil Paralelo",
            "JOTA",
        ] {
            let rating = catalog.lookup(name, "");
            assert_ne!(rating.lean, Lean::Unknown, "{name} sem classificação");
        }
    }

    #[test]
    fn unknown_source_falls_back_to_unknown() {
        let catalog = Catalog::load();
        let rating = catalog.lookup("Um site qualquer", "https://exemplo.invalid/noticia");
        assert_eq!(rating.lean, Lean::Unknown);
        assert_eq!(rating.factuality, Factuality::Unknown);
    }

    #[test]
    fn known_source_is_rated() {
        let catalog = Catalog::load();
        let rating = catalog.lookup("Folha", "https://www1.folha.uol.com.br/mercado");
        assert_eq!(rating.lean, Lean::Center);
        assert_eq!(rating.factuality, Factuality::High);
    }
}
