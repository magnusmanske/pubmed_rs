use roxmltree::ParsingOptions;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;

use crate::types::PubmedArticle;

const DEFAULT_BASE_URL: &str = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    api_key: Option<String>,
    #[serde(skip)]
    http_client: reqwest::Client,
    #[serde(default = "default_base_url")]
    base_url: String,
}

fn default_base_url() -> String {
    DEFAULT_BASE_URL.to_string()
}

impl Client {
    /// Creates a new `Client`, optionally loading an API key from a file
    /// named `ncbi_key` in the current working directory. Whitespace is
    /// trimmed from the key.
    #[must_use]
    pub fn new() -> Self {
        let api_key = fs::read_to_string("ncbi_key")
            .ok()
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty());
        Client {
            api_key,
            http_client: reqwest::Client::new(),
            base_url: default_base_url(),
        }
    }

    /// Creates a new `Client` with an explicit API key.
    pub fn with_api_key(api_key: impl Into<String>) -> Self {
        let key = api_key.into();
        Client {
            api_key: if key.is_empty() { None } else { Some(key) },
            http_client: reqwest::Client::new(),
            base_url: default_base_url(),
        }
    }

    /// Replaces the internal `reqwest::Client`. Useful for sharing a
    /// connection pool, embedding middleware (retry layers, tracing,
    /// custom user-agent), or in tests that want fast-fail timeouts.
    /// All other configuration set so far is preserved.
    pub fn http_client(mut self, client: reqwest::Client) -> Self {
        self.http_client = client;
        self
    }

    /// Overrides the API base URL. Lets tests redirect every request
    /// to a wiremock server: `.with_base_url(mock.uri())`.
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    fn api_key_param(&self) -> String {
        match &self.api_key {
            Some(key) => format!("&api_key={key}"),
            None => String::new(),
        }
    }

    pub async fn article_ids_from_query(
        &self,
        query: &str,
        max: u64,
    ) -> Result<Vec<u64>, Box<dyn Error>> {
        let url = format!(
            "{}/esearch.fcgi?db=pubmed&retmode=json&retmax={}&term={}{}",
            self.base_url,
            max,
            query,
            self.api_key_param()
        );
        let json: serde_json::Value = self.http_client.get(&url).send().await?.json().await?;
        match json["esearchresult"]["idlist"].as_array() {
            Some(idlist) => Ok(idlist
                .iter()
                .filter_map(|id| {
                    id.as_str().and_then(|x| {
                        if let Ok(u) = x.parse::<u64>() {
                            Some(u)
                        } else {
                            eprintln!(
                                "PubMed::article_ids_from_query: '{x}' should be a numeric ID"
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
        let ids: Vec<String> = ids.iter().map(std::string::ToString::to_string).collect();
        let url = format!(
            "{}/efetch.fcgi?db=pubmed&retmode=xml&id={}{}",
            self.base_url,
            ids.join(","),
            self.api_key_param()
        );
        let text = self.http_client.get(&url).send().await?.text().await?;
        let parsing_options = ParsingOptions {
            allow_dtd: true,
            nodes_limit: u32::MAX,
            ..Default::default()
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
        if self.api_key.is_some() {
            std::time::Duration::from_millis(120) // 10/sec with api_key
        } else {
            std::time::Duration::from_millis(400) // 3/sec without api key
        }
    }

    pub async fn article(&self, id: u64) -> Result<PubmedArticle, Box<dyn Error>> {
        match self.articles(&[id]).await?.pop() {
            Some(pubmed_article) => Ok(pubmed_article),
            None => Err(From::from(format!(
                "Can't find PubmedArticle for ID '{id}'"
            ))),
        }
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}
