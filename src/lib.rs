pub mod client;
pub mod helpers;
pub mod types;

// Re-export all public types at the crate root for backwards compatibility
pub use client::Client;
pub use types::*;

#[cfg(test)]
mod tests {
    use crate::types::pubmed_date::PubMedDate;

    /// Helper: parse an XML string and return the root element node.
    fn root_element(xml: &str) -> roxmltree::Document<'_> {
        roxmltree::Document::parse(xml).unwrap()
    }

    // ── Network-dependent integration tests ──────────────────────────

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

    // ── Offline unit tests ───────────────────────────────────────────

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
    fn test_pubmed_date_from_xml_full() {
        let xml = r#"<PubMedPubDate PubStatus="received"><Year>2020</Year><Month>03</Month><Day>15</Day><Hour>10</Hour><Minute>30</Minute></PubMedPubDate>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        let date = PubMedDate::new_from_xml(&node).unwrap();
        assert_eq!(date.year, 2020);
        assert_eq!(date.month, 3);
        assert_eq!(date.day, 15);
        assert_eq!(date.hour, 10);
        assert_eq!(date.minute, 30);
        assert_eq!(date.pub_status.as_deref(), Some("received"));
        assert_eq!(date.precision(), 13);
    }

    #[test]
    fn test_pubmed_date_from_xml_month_name() {
        let xml = r#"<Date><Year>1999</Year><Month>Jan</Month></Date>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        let date = PubMedDate::new_from_xml(&node).unwrap();
        assert_eq!(date.year, 1999);
        assert_eq!(date.month, 1);
        assert_eq!(date.day, 0);
    }

    #[test]
    fn test_pubmed_date_from_xml_month_names_all() {
        let months = [
            ("Jan", 1),
            ("Feb", 2),
            ("Mar", 3),
            ("Apr", 4),
            ("May", 5),
            ("Jun", 6),
            ("Jul", 7),
            ("Aug", 8),
            ("Sep", 9),
            ("Oct", 10),
            ("Nov", 11),
            ("Dec", 12),
        ];
        for (name, expected) in &months {
            let xml = format!(r#"<Date><Year>2000</Year><Month>{}</Month></Date>"#, name);
            let doc = root_element(&xml);
            let node = doc.root_element();
            let date = PubMedDate::new_from_xml(&node).unwrap();
            assert_eq!(date.month, *expected, "Failed for month name {}", name);
        }
    }

    #[test]
    fn test_pubmed_date_from_xml_year_only_returns_some() {
        let xml = r#"<Date><Year>2020</Year></Date>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        let date = PubMedDate::new_from_xml(&node);
        assert!(date.is_some());
        assert_eq!(date.unwrap().precision(), 9);
    }

    #[test]
    fn test_pubmed_date_from_xml_empty_returns_none() {
        let xml = r#"<Date></Date>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        assert!(PubMedDate::new_from_xml(&node).is_none());
    }

    #[test]
    fn test_pubmed_date_from_xml_invalid_year() {
        let xml = r#"<Date><Year>notanumber</Year></Date>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        // Invalid year parses to 0, which gives precision 0 → None
        assert!(PubMedDate::new_from_xml(&node).is_none());
    }

    #[test]
    fn test_mesh_term_part_from_xml() {
        let xml = r#"<DescriptorName UI="D001234" MajorTopicYN="Y">Some Term</DescriptorName>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        let part = crate::MeshTermPart::new_from_xml(&node);
        assert_eq!(part.ui.as_deref(), Some("D001234"));
        assert!(part.major_topic);
        assert_eq!(part.name.as_deref(), Some("Some Term"));
    }

    #[test]
    fn test_mesh_term_part_not_major() {
        let xml = r#"<DescriptorName UI="D005678" MajorTopicYN="N">Another Term</DescriptorName>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        let part = crate::MeshTermPart::new_from_xml(&node);
        assert!(!part.major_topic);
    }

    #[test]
    fn test_mesh_heading_from_xml() {
        let xml = r#"<MeshHeading><DescriptorName UI="D001" MajorTopicYN="N">Descriptor</DescriptorName><QualifierName UI="Q001" MajorTopicYN="Y">Qualifier1</QualifierName><QualifierName UI="Q002" MajorTopicYN="N">Qualifier2</QualifierName></MeshHeading>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        let heading = crate::MeshHeading::new_from_xml(&node).unwrap();
        assert_eq!(heading.descriptor.ui.as_deref(), Some("D001"));
        assert_eq!(heading.qualifiers.len(), 2);
        assert!(heading.qualifiers[0].major_topic);
        assert!(!heading.qualifiers[1].major_topic);
    }

    #[test]
    fn test_mesh_heading_without_descriptor_returns_none() {
        let xml = r#"<MeshHeading><QualifierName UI="Q001" MajorTopicYN="Y">Qualifier</QualifierName></MeshHeading>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        assert!(crate::MeshHeading::new_from_xml(&node).is_none());
    }

    #[test]
    fn test_elocation_id_from_xml() {
        let xml = r#"<ELocationID EIdType="doi" ValidYN="Y">10.1234/test</ELocationID>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        let eloc = crate::ELocationID::new_from_xml(&node);
        assert_eq!(eloc.e_id_type.as_deref(), Some("doi"));
        assert!(eloc.valid);
        assert_eq!(eloc.id.as_deref(), Some("10.1234/test"));
    }

    #[test]
    fn test_elocation_id_invalid() {
        let xml = r#"<ELocationID EIdType="pii" ValidYN="N">S1234</ELocationID>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        let eloc = crate::ELocationID::new_from_xml(&node);
        assert!(!eloc.valid);
    }

    #[test]
    fn test_abstract_from_xml() {
        let xml = r#"<Abstract><AbstractText>This is the abstract text.</AbstractText></Abstract>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        let abs = crate::Abstract::new_from_xml(&node);
        assert_eq!(abs.text.as_deref(), Some("This is the abstract text."));
    }

    #[test]
    fn test_abstract_empty() {
        let xml = r#"<Abstract></Abstract>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        let abs = crate::Abstract::new_from_xml(&node);
        assert!(abs.text.is_none());
    }

    #[test]
    fn test_identifier_from_xml() {
        let xml = r#"<Identifier Source="ORCID">0000-0001-2345-6789</Identifier>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        let id = crate::Identifier::new_from_xml(&node);
        assert_eq!(id.source.as_deref(), Some("ORCID"));
        assert_eq!(id.id.as_deref(), Some("0000-0001-2345-6789"));
    }

    #[test]
    fn test_affiliation_info_from_xml() {
        let xml = r#"<AffiliationInfo><Affiliation>Some University</Affiliation><Identifier Source="GRID">grid.12345</Identifier></AffiliationInfo>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        let aff = crate::AffiliationInfo::new_from_xml(&node);
        assert_eq!(aff.affiliation.as_deref(), Some("Some University"));
        assert_eq!(aff.identifiers.len(), 1);
        assert_eq!(aff.identifiers[0].source.as_deref(), Some("GRID"));
    }

    #[test]
    fn test_author_from_xml() {
        let xml = r#"<Author ValidYN="Y"><LastName>Smith</LastName><ForeName>John</ForeName><Initials>J</Initials><Suffix>Jr</Suffix></Author>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        let author = crate::Author::new_from_xml(&node);
        assert!(author.valid);
        assert_eq!(author.last_name.as_deref(), Some("Smith"));
        assert_eq!(author.fore_name.as_deref(), Some("John"));
        assert_eq!(author.initials.as_deref(), Some("J"));
        assert_eq!(author.suffix.as_deref(), Some("Jr"));
        assert!(author.collective_name.is_none());
    }

    #[test]
    fn test_author_collective_name() {
        let xml = r#"<Author ValidYN="Y"><CollectiveName>WHO Group</CollectiveName></Author>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        let author = crate::Author::new_from_xml(&node);
        assert_eq!(author.collective_name.as_deref(), Some("WHO Group"));
        assert!(author.last_name.is_none());
    }

    #[test]
    fn test_author_list_from_xml() {
        let xml = r#"<AuthorList CompleteYN="Y"><Author ValidYN="Y"><LastName>A</LastName></Author><Author ValidYN="Y"><LastName>B</LastName></Author></AuthorList>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        let list = crate::AuthorList::new_from_xml(&node);
        assert!(list.complete);
        assert_eq!(list.authors.len(), 2);
        assert_eq!(list.authors[0].last_name.as_deref(), Some("A"));
        assert_eq!(list.authors[1].last_name.as_deref(), Some("B"));
    }

    #[test]
    fn test_author_list_incomplete() {
        let xml = r#"<AuthorList CompleteYN="N"><Author ValidYN="Y"><LastName>A</LastName></Author></AuthorList>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        let list = crate::AuthorList::new_from_xml(&node);
        assert!(!list.complete);
    }

    #[test]
    fn test_journal_issue_from_xml() {
        let xml = r#"<JournalIssue CitedMedium="Internet"><Volume>42</Volume><Issue>3</Issue><PubDate><Year>2020</Year><Month>Mar</Month></PubDate></JournalIssue>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        let ji = crate::JournalIssue::new_from_xml(&node);
        assert_eq!(ji.cited_medium.as_deref(), Some("Internet"));
        assert_eq!(ji.volume.as_deref(), Some("42"));
        assert_eq!(ji.issue.as_deref(), Some("3"));
        let date = ji.pub_date.unwrap();
        assert_eq!(date.year, 2020);
        assert_eq!(date.month, 3);
    }

    #[test]
    fn test_journal_from_xml() {
        let xml = r#"<Journal><ISSN IssnType="Electronic">1234-5678</ISSN><JournalIssue CitedMedium="Internet"><Volume>1</Volume></JournalIssue><Title>Test Journal</Title><ISOAbbreviation>Test J.</ISOAbbreviation></Journal>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        let j = crate::Journal::new_from_xml(&node);
        assert_eq!(j.issn.as_deref(), Some("1234-5678"));
        assert_eq!(j.issn_type.as_deref(), Some("Electronic"));
        assert_eq!(j.title.as_deref(), Some("Test Journal"));
        assert_eq!(j.iso_abbreviation.as_deref(), Some("Test J."));
        assert!(j.journal_issue.is_some());
    }

    #[test]
    fn test_grant_from_xml() {
        let xml = r#"<Grant><GrantID>R01-123</GrantID><Agency>NIH</Agency><Country>US</Country><Acronym>NH</Acronym></Grant>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        let g = crate::Grant::new_from_xml(&node);
        assert_eq!(g.grant_id.as_deref(), Some("R01-123"));
        assert_eq!(g.agency.as_deref(), Some("NIH"));
        assert_eq!(g.country.as_deref(), Some("US"));
        assert_eq!(g.acronym.as_deref(), Some("NH"));
    }

    #[test]
    fn test_grant_list_from_xml() {
        let xml = r#"<GrantList CompleteYN="Y"><Grant><GrantID>G1</GrantID></Grant><Grant><GrantID>G2</GrantID></Grant></GrantList>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        let gl = crate::GrantList::new_from_xml(&node);
        assert!(gl.complete);
        assert_eq!(gl.grants.len(), 2);
    }

    #[test]
    fn test_publication_type_from_xml() {
        let xml = r#"<PublicationType UI="D016428">Journal Article</PublicationType>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        let pt = crate::PublicationType::new_from_xml(&node);
        assert_eq!(pt.ui.as_deref(), Some("D016428"));
        assert_eq!(pt.name.as_deref(), Some("Journal Article"));
    }

    #[test]
    fn test_chemical_from_xml() {
        let xml = r#"<Chemical><RegistryNumber>0</RegistryNumber><NameOfSubstance UI="D014867">Water</NameOfSubstance></Chemical>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        let c = crate::Chemical::new_from_xml(&node);
        assert_eq!(c.registry_number.as_deref(), Some("0"));
        assert_eq!(c.name_of_substance.as_deref(), Some("Water"));
        assert_eq!(c.name_of_substance_ui.as_deref(), Some("D014867"));
    }

    #[test]
    fn test_keyword_list_from_xml() {
        let xml = r#"<KeywordList Owner="NOTNLM"><Keyword MajorTopicYN="Y">Cancer</Keyword><Keyword MajorTopicYN="N">Therapy</Keyword></KeywordList>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        let kl = crate::KeywordList::new_from_xml(&node);
        assert_eq!(kl.owner.as_deref(), Some("NOTNLM"));
        assert_eq!(kl.keywords.len(), 2);
        assert!(kl.keywords[0].major_topic);
        assert_eq!(kl.keywords[0].keyword, "Cancer");
        assert!(!kl.keywords[1].major_topic);
        assert_eq!(kl.keywords[1].keyword, "Therapy");
    }

    #[test]
    fn test_medline_journal_info_from_xml() {
        let xml = r#"<MedlineJournalInfo><Country>England</Country><MedlineTA>Nature</MedlineTA><NlmUniqueID>0410462</NlmUniqueID><ISSNLinking>0028-0836</ISSNLinking></MedlineJournalInfo>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        let mji = crate::MedlineJournalInfo::new_from_xml(&node);
        assert_eq!(mji.country.as_deref(), Some("England"));
        assert_eq!(mji.medline_ta.as_deref(), Some("Nature"));
        assert_eq!(mji.nlm_unique_id.as_deref(), Some("0410462"));
        assert_eq!(mji.issn_linking.as_deref(), Some("0028-0836"));
    }

    #[test]
    fn test_article_from_xml() {
        let xml = r#"<Article PubModel="Print">
            <Journal><Title>Test</Title></Journal>
            <ArticleTitle>My Article</ArticleTitle>
            <Pagination><MedlinePgn>1-10</MedlinePgn></Pagination>
            <ELocationID EIdType="doi" ValidYN="Y">10.1/test</ELocationID>
            <Abstract><AbstractText>Abstract here.</AbstractText></Abstract>
            <AuthorList CompleteYN="Y"><Author ValidYN="Y"><LastName>Doe</LastName></Author></AuthorList>
            <Language>eng</Language>
            <VernacularTitle>Mon Article</VernacularTitle>
            <PublicationTypeList><PublicationType UI="D016428">Journal Article</PublicationType></PublicationTypeList>
            <ArticleDate DateType="Electronic"><Year>2020</Year><Month>01</Month><Day>01</Day></ArticleDate>
        </Article>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        let a = crate::Article::new_from_xml(&node);
        assert_eq!(a.pub_model.as_deref(), Some("Print"));
        assert_eq!(a.title.as_deref(), Some("My Article"));
        assert_eq!(a.language.as_deref(), Some("eng"));
        assert_eq!(a.vernacular_title.as_deref(), Some("Mon Article"));
        assert_eq!(a.pagination.len(), 1);
        assert_eq!(a.e_location_ids.len(), 1);
        assert_eq!(a.e_location_ids[0].id.as_deref(), Some("10.1/test"));
        assert!(a.the_abstract.is_some());
        assert!(a.author_list.is_some());
        assert_eq!(a.publication_type_list.len(), 1);
        assert_eq!(a.article_date.len(), 1);
        assert_eq!(a.article_date[0].year, 2020);
        assert!(a.journal.is_some());
    }

    #[test]
    fn test_medline_citation_from_xml() {
        let xml = r#"<MedlineCitation>
            <PMID>12345678</PMID>
            <DateCompleted><Year>2020</Year><Month>06</Month><Day>15</Day></DateCompleted>
            <DateRevised><Year>2021</Year><Month>01</Month><Day>01</Day></DateRevised>
            <Article PubModel="Print"><ArticleTitle>Test</ArticleTitle><Journal><Title>J</Title></Journal><PublicationTypeList><PublicationType UI="D016428">Journal Article</PublicationType></PublicationTypeList></Article>
            <MedlineJournalInfo><MedlineTA>Test J</MedlineTA></MedlineJournalInfo>
            <CitationSubset>IM</CitationSubset>
            <KeywordList Owner="NLM"><Keyword MajorTopicYN="N">kw1</Keyword></KeywordList>
            <ChemicalList><Chemical><RegistryNumber>0</RegistryNumber><NameOfSubstance UI="D000">Aspirin</NameOfSubstance></Chemical></ChemicalList>
            <CoiStatement>None declared.</CoiStatement>
            <MeshHeadingList><MeshHeading><DescriptorName UI="D001" MajorTopicYN="N">Term</DescriptorName></MeshHeading></MeshHeadingList>
        </MedlineCitation>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        let mc = crate::types::medline_citation::MedlineCitation::new_from_xml(&node);
        assert_eq!(mc.pmid, 12345678);
        assert!(mc.date_completed.is_some());
        assert_eq!(mc.date_completed.as_ref().unwrap().year, 2020);
        assert!(mc.date_revised.is_some());
        assert!(mc.article.is_some());
        assert!(mc.medline_journal_info.is_some());
        assert_eq!(mc.citation_subsets, vec!["IM"]);
        assert_eq!(mc.keyword_lists.len(), 1);
        assert_eq!(mc.chemical_list.len(), 1);
        assert_eq!(mc.coi_statement.as_deref(), Some("None declared."));
        assert_eq!(mc.mesh_heading_list.len(), 1);
    }

    #[test]
    fn test_pubmed_article_from_xml() {
        let xml = r#"<PubmedArticle>
            <MedlineCitation>
                <PMID>99999</PMID>
                <Article PubModel="Print"><ArticleTitle>T</ArticleTitle><Journal><Title>J</Title></Journal><PublicationTypeList></PublicationTypeList></Article>
            </MedlineCitation>
            <PubmedData>
                <PublicationStatus>ppublish</PublicationStatus>
                <ArticleIdList><ArticleId IdType="pubmed">99999</ArticleId><ArticleId IdType="doi">10.1/x</ArticleId></ArticleIdList>
                <History><PubMedPubDate PubStatus="pubmed"><Year>2020</Year><Month>01</Month><Day>01</Day></PubMedPubDate></History>
                <ReferenceList><Title>References</Title><Reference><Citation>Ref 1</Citation></Reference></ReferenceList>
            </PubmedData>
        </PubmedArticle>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        let pa = crate::PubmedArticle::new_from_xml(&node);
        assert!(pa.medline_citation.is_some());
        let mc = pa.medline_citation.unwrap();
        assert_eq!(mc.pmid, 99999);

        assert!(pa.pubmed_data.is_some());
        let pd = pa.pubmed_data.unwrap();
        assert_eq!(pd.publication_status.as_deref(), Some("ppublish"));
        assert!(pd.article_ids.is_some());
        let ids = pd.article_ids.unwrap();
        assert_eq!(ids.ids.len(), 2);
        assert_eq!(ids.ids[0].id_type.as_deref(), Some("pubmed"));
        assert_eq!(ids.ids[1].id_type.as_deref(), Some("doi"));
        assert_eq!(pd.history.len(), 1);
        assert_eq!(pd.references.len(), 1);
        assert_eq!(pd.references[0].citation.as_deref(), Some("Ref 1"));
    }

    #[test]
    fn test_reference_with_article_ids() {
        let xml = r#"<Reference><Citation>Some paper</Citation><ArticleIdList><ArticleId IdType="doi">10.1/ref</ArticleId></ArticleIdList></Reference>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        let r = crate::types::reference::Reference::new_from_xml(&node);
        assert_eq!(r.citation.as_deref(), Some("Some paper"));
        assert!(r.article_ids.is_some());
        assert_eq!(r.article_ids.unwrap().ids.len(), 1);
    }

    #[test]
    fn test_pagination_medline_pgn() {
        let xml = r#"<Article PubModel="Print"><ArticleTitle>T</ArticleTitle><Journal><Title>J</Title></Journal><Pagination><MedlinePgn>123-456</MedlinePgn></Pagination><PublicationTypeList></PublicationTypeList></Article>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        let a = crate::Article::new_from_xml(&node);
        assert_eq!(a.pagination.len(), 1);
        match &a.pagination[0] {
            crate::Pagination::MedlinePgn(s) => assert_eq!(s, "123-456"),
        }
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

    #[test]
    fn test_serde_roundtrip_pubmed_date() {
        let date = PubMedDate {
            year: 2020,
            month: 6,
            day: 15,
            hour: 10,
            minute: 30,
            date_type: Some("Electronic".to_string()),
            pub_status: Some("received".to_string()),
        };
        let json = serde_json::to_string(&date).unwrap();
        let date2: PubMedDate = serde_json::from_str(&json).unwrap();
        assert_eq!(date.year, date2.year);
        assert_eq!(date.month, date2.month);
        assert_eq!(date.day, date2.day);
        assert_eq!(date.hour, date2.hour);
        assert_eq!(date.minute, date2.minute);
        assert_eq!(date.date_type, date2.date_type);
        assert_eq!(date.pub_status, date2.pub_status);
    }

    #[test]
    fn test_serde_roundtrip_article() {
        let article = crate::Article::new();
        let json = serde_json::to_string(&article).unwrap();
        let article2: crate::Article = serde_json::from_str(&json).unwrap();
        assert!(article2.pub_model.is_none());
        assert!(article2.pagination.is_empty());
    }

    #[test]
    fn test_medline_citation_gene_symbol_list() {
        let xml = r#"<MedlineCitation>
            <PMID>1</PMID>
            <Article PubModel="Print"><ArticleTitle>T</ArticleTitle><Journal><Title>J</Title></Journal><PublicationTypeList></PublicationTypeList></Article>
            <GeneSymbolList><GeneSymbol>BRCA1</GeneSymbol><GeneSymbol>TP53</GeneSymbol></GeneSymbolList>
        </MedlineCitation>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        let mc = crate::types::medline_citation::MedlineCitation::new_from_xml(&node);
        assert_eq!(mc.gene_symbol_list, vec!["BRCA1", "TP53"]);
    }

    #[test]
    fn test_medline_citation_investigator_list() {
        let xml = r#"<MedlineCitation>
            <PMID>1</PMID>
            <Article PubModel="Print"><ArticleTitle>T</ArticleTitle><Journal><Title>J</Title></Journal><PublicationTypeList></PublicationTypeList></Article>
            <InvestigatorList><Investigator ValidYN="Y"><LastName>Jones</LastName><ForeName>Alice</ForeName></Investigator></InvestigatorList>
        </MedlineCitation>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        let mc = crate::types::medline_citation::MedlineCitation::new_from_xml(&node);
        assert_eq!(mc.investigator_list.len(), 1);
        assert_eq!(mc.investigator_list[0].last_name.as_deref(), Some("Jones"));
    }

    #[test]
    fn test_medline_citation_other_id() {
        let xml = r#"<MedlineCitation>
            <PMID>1</PMID>
            <Article PubModel="Print"><ArticleTitle>T</ArticleTitle><Journal><Title>J</Title></Journal><PublicationTypeList></PublicationTypeList></Article>
            <OtherID Source="NLM">PMC12345</OtherID>
        </MedlineCitation>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        let mc = crate::types::medline_citation::MedlineCitation::new_from_xml(&node);
        assert_eq!(mc.other_ids.len(), 1);
        assert_eq!(mc.other_ids[0].source.as_deref(), Some("NLM"));
        assert_eq!(mc.other_ids[0].id.as_deref(), Some("PMC12345"));
    }

    #[test]
    fn test_medline_citation_number_of_references() {
        let xml = r#"<MedlineCitation>
            <PMID>1</PMID>
            <Article PubModel="Print"><ArticleTitle>T</ArticleTitle><Journal><Title>J</Title></Journal><PublicationTypeList></PublicationTypeList></Article>
            <NumberOfReferences>42</NumberOfReferences>
        </MedlineCitation>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        let mc = crate::types::medline_citation::MedlineCitation::new_from_xml(&node);
        assert_eq!(mc.number_of_references.as_deref(), Some("42"));
    }

    #[test]
    fn test_pubmed_data_history() {
        let xml = r#"<PubmedData>
            <History>
                <PubMedPubDate PubStatus="received"><Year>2020</Year><Month>01</Month><Day>01</Day></PubMedPubDate>
                <PubMedPubDate PubStatus="accepted"><Year>2020</Year><Month>03</Month><Day>15</Day></PubMedPubDate>
            </History>
        </PubmedData>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        let pd = crate::PubmedData::new_from_xml(&node);
        assert_eq!(pd.history.len(), 2);
        assert_eq!(pd.history[0].pub_status.as_deref(), Some("received"));
        assert_eq!(pd.history[1].pub_status.as_deref(), Some("accepted"));
    }

    #[test]
    fn test_author_with_affiliation_and_identifiers() {
        let xml = r#"<Author ValidYN="Y">
            <LastName>Doe</LastName>
            <ForeName>Jane</ForeName>
            <Initials>JD</Initials>
            <Identifier Source="ORCID">0000-0000-0000-0001</Identifier>
            <AffiliationInfo><Affiliation>MIT</Affiliation></AffiliationInfo>
        </Author>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        let author = crate::Author::new_from_xml(&node);
        assert_eq!(author.identifiers.len(), 1);
        assert_eq!(author.identifiers[0].source.as_deref(), Some("ORCID"));
        assert!(author.affiliation_info.is_some());
        assert_eq!(
            author.affiliation_info.unwrap().affiliation.as_deref(),
            Some("MIT")
        );
    }

    #[test]
    fn test_article_with_grant_list() {
        let xml = r#"<Article PubModel="Electronic">
            <ArticleTitle>T</ArticleTitle>
            <Journal><Title>J</Title></Journal>
            <GrantList CompleteYN="Y">
                <Grant><GrantID>ABC</GrantID><Agency>NSF</Agency><Country>US</Country></Grant>
            </GrantList>
            <PublicationTypeList></PublicationTypeList>
        </Article>"#;
        let doc = root_element(xml);
        let node = doc.root_element();
        let a = crate::Article::new_from_xml(&node);
        assert!(a.grant_list.is_some());
        let gl = a.grant_list.unwrap();
        assert!(gl.complete);
        assert_eq!(gl.grants.len(), 1);
        assert_eq!(gl.grants[0].grant_id.as_deref(), Some("ABC"));
    }
}
