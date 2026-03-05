pub mod client;
pub mod helpers;
pub mod types;

// Re-export all public types at the crate root for backwards compatibility
pub use client::Client;
pub use types::*;

#[cfg(test)]
mod tests {
    use crate::types::pubmed_date::PubMedDate;

    #[tokio::test]
    async fn doi() {
        let client = crate::Client::new();
        let ids = client
            .article_ids_from_query("\"10.1038/NATURE11174\"", 1000)
            .await
            .unwrap();
        assert_eq!(ids, vec![22722859])
    }

    #[tokio::test]
    async fn work() {
        let client = crate::Client::new();
        let article = client.article(22722859).await.unwrap();
        let date = article
            .medline_citation
            .unwrap()
            .date_completed
            .unwrap()
            .clone();
        assert_eq!(date.year, 2012);
        assert_eq!(date.month, 8);
        assert_eq!(date.day, 17);
    }

    #[tokio::test]
    async fn date_parsing() {
        let client = crate::Client::new();
        let article = client.article(13777676).await.unwrap();
        let date = article
            .medline_citation
            .unwrap()
            .article
            .unwrap()
            .journal
            .unwrap()
            .journal_issue
            .unwrap()
            .pub_date
            .unwrap();
        assert_eq!(date.year, 1961);
        assert_eq!(date.month, 5);
        assert_eq!(date.day, 0);
    }

    #[test]
    fn test_pubmed_date_precision() {
        let date = PubMedDate {
            year: 0,
            month: 0,
            day: 0,
            hour: -1,
            minute: -1,
            date_type: None,
            pub_status: None,
        };
        assert_eq!(date.precision(), 0);

        let date = PubMedDate {
            year: 2020,
            month: 0,
            day: 0,
            hour: -1,
            minute: -1,
            date_type: None,
            pub_status: None,
        };
        assert_eq!(date.precision(), 9);

        let date = PubMedDate {
            year: 2020,
            month: 6,
            day: 0,
            hour: -1,
            minute: -1,
            date_type: None,
            pub_status: None,
        };
        assert_eq!(date.precision(), 10);

        let date = PubMedDate {
            year: 2020,
            month: 6,
            day: 15,
            hour: -1,
            minute: -1,
            date_type: None,
            pub_status: None,
        };
        assert_eq!(date.precision(), 11);

        let date = PubMedDate {
            year: 2020,
            month: 6,
            day: 15,
            hour: 10,
            minute: -1,
            date_type: None,
            pub_status: None,
        };
        assert_eq!(date.precision(), 12);

        let date = PubMedDate {
            year: 2020,
            month: 6,
            day: 15,
            hour: 10,
            minute: 30,
            date_type: None,
            pub_status: None,
        };
        assert_eq!(date.precision(), 13);
    }

    #[test]
    fn test_default_instances() {
        let article = crate::Article::new();
        assert!(article.pub_model.is_none());
        assert!(article.journal.is_none());
        assert!(article.pagination.is_empty());

        let journal = crate::Journal::new();
        assert!(journal.issn.is_none());
        assert!(journal.title.is_none());

        let journal_issue = crate::JournalIssue::new();
        assert!(journal_issue.cited_medium.is_none());
        assert!(journal_issue.pub_date.is_none());

        let citation = crate::MedlineCitation::new();
        assert_eq!(citation.pmid, 0);
        assert!(citation.article.is_none());
        assert!(citation.mesh_heading_list.is_empty());
    }

    #[test]
    fn test_client_default() {
        let client = crate::Client::default();
        // Just ensure it constructs without panic
        let _ = format!("{:?}", client);
    }
}
