use thiserror::Error;

/// Errors returned by this crate.
#[derive(Debug, Error)]
pub enum PubMedError {
    /// The HTTP request to the PubMed API failed.
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// The XML response could not be parsed.
    #[error("failed to parse XML response: {0}")]
    Xml(#[from] roxmltree::Error),

    /// PubMed returned a response without the expected result payload.
    #[error("PubMed API returned no results")]
    NoResults,

    /// The requested article was not present in the response.
    #[error("no PubMed article found for ID '{0}'")]
    ArticleNotFound(u64),
}

/// Convenience alias for results returned by this crate.
pub type Result<T> = std::result::Result<T, PubMedError>;
