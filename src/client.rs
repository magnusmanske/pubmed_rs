use roxmltree::ParsingOptions;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;

use crate::types::PubmedArticle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    api_key: Option<String>,
}

impl Client {
    /// Creates a new `Client`, optionally loading an API key from a file
    /// named `ncbi_key` in the current working directory. Whitespace is
    /// trimmed from the key.
    pub fn new() -> Self {
        let api_key = fs::read_to_string("ncbi_key")
            .ok()
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty());
        Client { api_key }
    }

    /// Creates a new `Client` with an explicit API key.
    pub fn with_api_key(api_key: impl Into<String>) -> Self {
        let key = api_key.into();
        Client {
            api_key: if key.is_empty() { None } else { Some(key) },
        }
    }

    pub async fn article_ids_from_query(
        &self,
        query: &str,
        max: u64,
    ) -> Result<Vec<u64>, Box<dyn Error>> {
        let url = format!(
            "http://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi?db=pubmed&retmode=json&retmax={}&term={}",
            max, query
        );
        let json: serde_json::Value = reqwest::get(url.as_str()).await?.json().await?;
        match json["esearchresult"]["idlist"].as_array() {
            Some(idlist) => Ok(idlist
                .iter()
                .filter_map(|id| {
                    id.as_str().and_then(|x| match x.parse::<u64>() {
                        Ok(u) => Some(u),
                        Err(_) => {
                            eprintln!(
                                "PubMed::article_ids_from_query: '{}' should be a numeric ID",
                                x
                            );
                            None
                        }
                    })
                })
                .collect()),
            None => Err(From::from("API error/no results")),
        }
    }

    pub async fn articles(&self, ids: &[u64]) -> Result<Vec<PubmedArticle>, Box<dyn Error>> {
        let ids: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
        let url = format!(
            "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=pubmed&retmode=xml&id={}",
            ids.join(",")
        );
        let text = reqwest::get(url.as_str()).await?.text().await?;
        let parsing_options = ParsingOptions {
            allow_dtd: true,
            nodes_limit: u32::MAX,
        };
        let doc = roxmltree::Document::parse_with_options(&text, parsing_options)?;
        tokio::time::sleep(self.get_sleep_time()).await; // To avoid being blocked by PubMed API
        Ok(doc
            .root()
            .descendants()
            .filter(|n| n.is_element() && n.tag_name().name() == "PubmedArticle")
            .map(|n| PubmedArticle::new_from_xml(&n))
            .collect())
    }

    fn get_sleep_time(&self) -> std::time::Duration {
        match self.api_key {
            Some(_) => std::time::Duration::from_millis(120), // 10/sec with api_key
            None => std::time::Duration::from_millis(400),    // 3/sec without api key
        }
    }

    pub async fn article(&self, id: u64) -> Result<PubmedArticle, Box<dyn Error>> {
        match self.articles(&[id]).await?.pop() {
            Some(pubmed_article) => Ok(pubmed_article),
            None => Err(From::from(format!(
                "Can't find PubmedArticle for ID '{}'",
                id
            ))),
        }
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
