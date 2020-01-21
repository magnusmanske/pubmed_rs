extern crate roxmltree;

use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::{thread, time};

#[cfg(debug_assertions)]
fn missing_tag_warning(_s: &str) {
    panic!("{}", _s);
}

#[cfg(not(debug_assertions))]
fn missing_tag_warning(_s: &str) {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubMedDate {
    pub year: u32,
    pub month: u8,
    pub day: u8,
    pub hour: i8,
    pub minute: i8,
    pub date_type: Option<String>,
    pub pub_status: Option<String>,
}

impl PubMedDate {
    fn new_from_xml(node: &roxmltree::Node) -> Option<PubMedDate> {
        let mut ret = Self {
            year: 0,
            month: 0,
            day: 0,
            hour: -1,
            minute: -1,
            date_type: node.attribute("DateType").map(|v| v.to_string()),
            pub_status: node.attribute("PubStatus").map(|v| v.to_string()),
        };

        for n in node.children().filter(|n| n.is_element()) {
            match n.tag_name().name() {
                "MedlineDate" => {} // TODO
                "Year" => {
                    ret.year = n
                        .text()
                        .map_or(0, |v| v.to_string().parse::<u32>().unwrap_or(0))
                }
                "Month" => ret.month = Self::parse_month_from_xml(&n),
                "Day" => {
                    ret.day = n
                        .text()
                        .map_or(0, |v| v.to_string().parse::<u8>().unwrap_or(0))
                }
                "Hour" => {
                    ret.hour = n
                        .text()
                        .map_or(-1, |v| v.to_string().parse::<i8>().unwrap_or(-1))
                }
                "Minute" => {
                    ret.minute = n
                        .text()
                        .map_or(-1, |v| v.to_string().parse::<i8>().unwrap_or(-1))
                }
                "Season" => {
                    // TODO
                    // Example: https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=pubmed&retmode=xml&id=11364263
                }
                x => missing_tag_warning(&format!("Not covered in PubMedDate: '{}'", x)),
            }
        }
        match ret.precision() {
            0 => None,
            _ => Some(ret),
        }
    }

    fn parse_month_from_xml(node: &roxmltree::Node) -> u8 {
        match node.text() {
            Some(t) => match t.to_lowercase().as_str() {
                "jan" => 1,
                "feb" => 2,
                "mar" => 3,
                "apr" => 4,
                "may" => 5,
                "jun" => 6,
                "jul" => 7,
                "aug" => 8,
                "sep" => 9,
                "oct" => 10,
                "nov" => 11,
                "dec" => 12,
                other => other.to_string().parse::<u8>().unwrap_or(0),
            },
            None => 0,
        }
    }

    // 13=minute, 12,hour, 11=day, 10=month, 9=year; same as Wikidata/wikibase
    pub fn precision(&self) -> u8 {
        if self.year == 0 {
            0
        } else if self.month == 0 {
            9
        } else if self.day == 0 {
            10
        } else if self.hour == -1 {
            11
        } else if self.minute == -1 {
            12
        } else {
            13
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshTermPart {
    pub ui: Option<String>,
    pub major_topic: bool,
    pub name: Option<String>,
}

impl MeshTermPart {
    fn new_from_xml(node: &roxmltree::Node) -> Self {
        Self {
            ui: node.attribute("UI").map(|v| v.to_string()),
            major_topic: node.attribute("MajorTopicYN").map_or(false, |v| v == "Y"),
            name: node.text().map(|v| v.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshHeading {
    pub descriptor: MeshTermPart,
    pub qualifiers: Vec<MeshTermPart>,
}

impl MeshHeading {
    fn new_from_xml(node: &roxmltree::Node) -> Option<Self> {
        let node_descriptor = node
            .descendants()
            .filter(|n| n.is_element() && n.tag_name().name() == "DescriptorName")
            .next()?;
        let qualifiers = node
            .descendants()
            .filter(|n| n.is_element() && n.tag_name().name() == "QualifierName")
            .map(|n| MeshTermPart::new_from_xml(&n))
            .collect();

        Some(Self {
            descriptor: MeshTermPart::new_from_xml(&node_descriptor),
            qualifiers: qualifiers,
        })
    }
}

//____________________________________________________________________________________________________
// Article

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ELocationID {
    pub e_id_type: Option<String>,
    pub valid: bool,
    pub id: Option<String>,
}

impl ELocationID {
    pub fn new_from_xml(node: &roxmltree::Node) -> Self {
        Self {
            e_id_type: node.attribute("EIdType").map(|v| v.to_string()),
            valid: node.attribute("ValidYN").map_or(false, |v| v == "Y"),
            id: node.text().map(|v| v.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Abstract {
    pub text: Option<String>,
}

impl Abstract {
    pub fn new_from_xml(node: &roxmltree::Node) -> Self {
        Self {
            text: node
                .descendants()
                .filter(|n| n.is_element() && n.tag_name().name() == "AbstractText")
                .map(|n| n.text().or(Some("")).unwrap_or("").to_string())
                .next(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffiliationInfo {
    pub affiliation: Option<String>,
    pub identifiers: Vec<Identifier>,
}

impl AffiliationInfo {
    pub fn new_from_xml(node: &roxmltree::Node) -> Self {
        let mut ret = Self {
            affiliation: None,
            identifiers: vec![],
        };
        for n in node.children().filter(|n| n.is_element()) {
            match n.tag_name().name() {
                "Affiliation" => ret.affiliation = n.text().map(|v| v.to_string()),
                "Identifier" => ret.identifiers.push(Identifier::new_from_xml(&n)),
                x => missing_tag_warning(&format!("Not covered in AffiliationInfo: '{}'", x)),
            }
        }
        ret
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identifier {
    pub id: Option<String>,
    pub source: Option<String>,
}

impl Identifier {
    pub fn new_from_xml(node: &roxmltree::Node) -> Self {
        Self {
            id: node.text().map(|v| v.to_string()),
            source: node.attribute("Source").map(|v| v.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub last_name: Option<String>,
    pub fore_name: Option<String>,
    pub initials: Option<String>,
    pub suffix: Option<String>,
    pub collective_name: Option<String>,
    pub affiliation_info: Option<AffiliationInfo>,
    pub identifiers: Vec<Identifier>,
    pub valid: bool,
}

impl Author {
    pub fn new_from_xml(node: &roxmltree::Node) -> Self {
        let mut ret = Self {
            last_name: None,
            fore_name: None,
            initials: None,
            suffix: None,
            collective_name: None,
            affiliation_info: None,
            identifiers: vec![],
            valid: node.attribute("ValidYN").map_or(false, |v| v == "Y"),
        };
        for n in node.children().filter(|n| n.is_element()) {
            match n.tag_name().name() {
                "LastName" => ret.last_name = n.text().map(|v| v.to_string()),
                "ForeName" => ret.fore_name = n.text().map(|v| v.to_string()),
                "CollectiveName" => ret.collective_name = n.text().map(|v| v.to_string()),
                "Initials" => ret.initials = n.text().map(|v| v.to_string()),
                "Suffix" => ret.suffix = n.text().map(|v| v.to_string()),
                "Identifier" => ret.identifiers.push(Identifier::new_from_xml(&n)),
                "AffiliationInfo" => ret.affiliation_info = Some(AffiliationInfo::new_from_xml(&n)),
                x => missing_tag_warning(&format!("Not covered in Author: '{}'", x)),
            }
        }
        ret
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorList {
    pub authors: Vec<Author>,
    pub complete: bool,
}

impl AuthorList {
    pub fn new_from_xml(node: &roxmltree::Node) -> Self {
        Self {
            complete: node.attribute("CompleteYN").map_or(false, |v| v == "Y"),
            authors: node
                .descendants()
                .filter(|n| n.is_element() && n.tag_name().name() == "Author")
                .map(|n| Author::new_from_xml(&n))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalIssue {
    pub cited_medium: Option<String>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub pub_date: Option<PubMedDate>,
}

impl JournalIssue {
    pub fn new() -> Self {
        Self {
            cited_medium: None,
            volume: None,
            issue: None,
            pub_date: None,
        }
    }

    pub fn new_from_xml(node: &roxmltree::Node) -> Self {
        let mut ret = Self::new();
        ret.cited_medium = node.attribute("CitedMedium").map(|v| v.to_string());
        for n in node.children().filter(|n| n.is_element()) {
            match n.tag_name().name() {
                "PubDate" => {
                    ret.pub_date = PubMedDate::new_from_xml(&n);
                }
                "Volume" => ret.volume = n.text().map(|v| v.to_string()),
                "Issue" => ret.issue = n.text().map(|v| v.to_string()),
                x => missing_tag_warning(&format!("Not covered in JournalIssue: '{}'", x)),
            }
        }
        ret
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Journal {
    pub issn: Option<String>,
    pub issn_type: Option<String>,
    pub journal_issue: Option<JournalIssue>,
    pub title: Option<String>,
    pub iso_abbreviation: Option<String>,
}

impl Journal {
    pub fn new() -> Self {
        Self {
            issn: None,
            issn_type: None,
            journal_issue: None,
            title: None,
            iso_abbreviation: None,
        }
    }

    pub fn new_from_xml(node: &roxmltree::Node) -> Self {
        let mut ret = Self::new();
        for n in node.children().filter(|n| n.is_element()) {
            match n.tag_name().name() {
                "ISSN" => {
                    ret.issn = n.text().map(|v| v.to_string());
                    ret.issn_type = n.attribute("IssnType").map(|v| v.to_string());
                }
                "JournalIssue" => ret.journal_issue = Some(JournalIssue::new_from_xml(&n)),
                "Title" => ret.title = n.text().map(|v| v.to_string()),
                "ISOAbbreviation" => ret.iso_abbreviation = n.text().map(|v| v.to_string()),
                x => missing_tag_warning(&format!("Not covered in Journal: '{}'", x)),
            }
        }
        ret
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Pagination {
    MedlinePgn(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grant {
    pub grant_id: Option<String>,
    pub agency: Option<String>,
    pub country: Option<String>,
    pub acronym: Option<String>,
}

impl Grant {
    pub fn new_from_xml(node: &roxmltree::Node) -> Self {
        let mut ret = Self {
            grant_id: None,
            agency: None,
            country: None,
            acronym: None,
        };
        for n in node.children().filter(|n| n.is_element()) {
            match n.tag_name().name() {
                "GrantID" => ret.grant_id = n.text().map(|v| v.to_string()),
                "Agency" => ret.agency = n.text().map(|v| v.to_string()),
                "Country" => ret.country = n.text().map(|v| v.to_string()),
                "Acronym" => ret.acronym = n.text().map(|v| v.to_string()),
                x => missing_tag_warning(&format!("Not covered in Grant: '{}'", x)),
            }
        }
        ret
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrantList {
    pub grants: Vec<Grant>,
    pub complete: bool,
}

impl GrantList {
    pub fn new_from_xml(node: &roxmltree::Node) -> Self {
        Self {
            complete: node.attribute("CompleteYN").map_or(false, |v| v == "Y"),
            grants: node
                .descendants()
                .filter(|n| n.is_element() && n.tag_name().name() == "Grant")
                .map(|n| Grant::new_from_xml(&n))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicationType {
    pub ui: Option<String>,
    pub name: Option<String>,
}

impl PublicationType {
    pub fn new_from_xml(node: &roxmltree::Node) -> Self {
        Self {
            ui: node.attribute("UI").map(|v| v.to_string()),
            name: node.text().map(|v| v.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub pub_model: Option<String>,
    pub journal: Option<Journal>,
    pub title: Option<String>,
    pub pagination: Vec<Pagination>,
    pub e_location_ids: Vec<ELocationID>,
    pub the_abstract: Option<Abstract>,
    pub author_list: Option<AuthorList>,
    pub language: Option<String>,
    pub vernacular_title: Option<String>,
    pub grant_list: Option<GrantList>,
    pub publication_type_list: Vec<PublicationType>,
    pub article_date: Vec<PubMedDate>,
}

impl Article {
    pub fn new() -> Self {
        Self {
            pub_model: None,
            journal: None,
            title: None,
            pagination: vec![],
            e_location_ids: vec![],
            the_abstract: None,
            author_list: None,
            language: None,
            vernacular_title: None,
            grant_list: None,
            publication_type_list: vec![],
            article_date: vec![],
        }
    }

    pub fn new_from_xml(node: &roxmltree::Node) -> Self {
        let mut ret = Article::new();
        ret.pub_model = node.attribute("PubModel").map(|v| v.to_string());
        for n in node.children().filter(|n| n.is_element()) {
            match n.tag_name().name() {
                "ArticleTitle" => ret.title = n.text().map(|v| v.to_string()),
                "Journal" => ret.journal = Some(Journal::new_from_xml(&n)),
                "Pagination" => {
                    for n2 in n.children().filter(|n| n.is_element()) {
                        match n2.tag_name().name() {
                            "MedlinePgn" => ret.pagination.push(Pagination::MedlinePgn(
                                n2.text().or(Some("")).unwrap_or("").to_string(),
                            )),
                            x => {
                                missing_tag_warning(&format!("Not covered in Pagination: '{}'", x))
                            }
                        }
                    }
                }
                "ELocationID" => ret.e_location_ids.push(ELocationID::new_from_xml(&n)),
                "Abstract" => ret.the_abstract = Some(Abstract::new_from_xml(&n)),
                "AuthorList" => ret.author_list = Some(AuthorList::new_from_xml(&n)),
                "Language" => ret.language = n.text().map(|v| v.to_string()),
                "VernacularTitle" => ret.vernacular_title = n.text().map(|v| v.to_string()),
                "GrantList" => ret.grant_list = Some(GrantList::new_from_xml(&n)),
                "ArticleDate" => match PubMedDate::new_from_xml(&n) {
                    Some(date) => ret.article_date.push(date),
                    None => {}
                },
                "PublicationTypeList" => {
                    ret.publication_type_list = n
                        .children()
                        .filter(|n| n.is_element() && n.tag_name().name() == "PublicationType")
                        .map(|n| PublicationType::new_from_xml(&n))
                        .collect()
                }
                "DataBankList" => {
                    // TODO
                    // Example: https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=pubmed&retmode=xml&id=2567002
                }
                x => missing_tag_warning(&format!("Not covered in Article: '{}'", x)),
            }
        }
        ret
    }
}

//____________________________________________________________________________________________________

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedlineJournalInfo {
    pub country: Option<String>,
    pub medline_ta: Option<String>,
    pub nlm_unique_id: Option<String>,
    pub issn_linking: Option<String>,
}

impl MedlineJournalInfo {
    pub fn new_from_xml(node: &roxmltree::Node) -> Self {
        let mut ret = Self {
            country: None,
            medline_ta: None,
            nlm_unique_id: None,
            issn_linking: None,
        };
        for n in node.children().filter(|n| n.is_element()) {
            match n.tag_name().name() {
                "Country" => ret.country = n.text().map(|v| v.to_string()),
                "MedlineTA" => ret.medline_ta = n.text().map(|v| v.to_string()),
                "NlmUniqueID" => ret.nlm_unique_id = n.text().map(|v| v.to_string()),
                "ISSNLinking" => ret.issn_linking = n.text().map(|v| v.to_string()),
                x => missing_tag_warning(&format!("Not covered in MedlineJournalInfo: '{}'", x)),
            }
        }
        ret
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtherID {
    pub source: Option<String>,
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyword {
    pub keyword: String,
    pub major_topic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordList {
    pub owner: Option<String>,
    pub keywords: Vec<Keyword>,
}

impl KeywordList {
    pub fn new_from_xml(node: &roxmltree::Node) -> Self {
        let mut ret = Self {
            owner: node.attribute("Owner").map(|v| v.to_string()),
            keywords: vec![],
        };
        for n in node.children().filter(|n| n.is_element()) {
            match n.tag_name().name() {
                "Keyword" => {
                    ret.keywords.push(Keyword {
                        major_topic: n.attribute("MajorTopicYN").map_or(false, |v| v == "Y"),
                        keyword: n.text().map_or("".to_string(), |v| v.to_string()),
                    });
                }
                x => missing_tag_warning(&format!("Not covered in KeywordList: '{}'", x)),
            }
        }
        ret
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chemical {
    registry_number: Option<String>,
    name_of_substance: Option<String>,
    name_of_substance_ui: Option<String>,
}

impl Chemical {
    /*
        pub fn new () -> Self {
            Self {
                registry_number:None,
                name_of_substance:None,
                name_of_substance_id:None,
            }
        }
    */
    pub fn new_from_xml(node: &roxmltree::Node) -> Self {
        let mut ret = Self {
            registry_number: None,
            name_of_substance: None,
            name_of_substance_ui: None,
        };
        for n in node.children().filter(|n| n.is_element()) {
            match n.tag_name().name() {
                "RegistryNumber" => ret.registry_number = n.text().map(|v| v.to_string()),
                "NameOfSubstance" => {
                    ret.name_of_substance = n.text().map(|v| v.to_string());
                    ret.name_of_substance_ui = n.attribute("UI").map(|v| v.to_string())
                }
                x => missing_tag_warning(&format!("Not covered in Chemical: '{}'", x)),
            }
        }
        ret
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MedlineCitation {
    pub pmid: u64,
    pub date_completed: Option<PubMedDate>,
    pub date_revised: Option<PubMedDate>,
    pub mesh_heading_list: Vec<MeshHeading>,
    pub medline_journal_info: Option<MedlineJournalInfo>,
    pub article: Option<Article>,
    pub other_ids: Vec<OtherID>,
    pub citation_subsets: Vec<String>,
    pub gene_symbol_list: Vec<String>,
    pub keyword_lists: Vec<KeywordList>,
    pub chemical_list: Vec<Chemical>,
    pub investigator_list: Vec<Author>,
    pub coi_statement: Option<String>,
    pub number_of_references: Option<String>,
}

impl MedlineCitation {
    pub fn new() -> Self {
        Self {
            pmid: 0,
            date_completed: None,
            date_revised: None,
            mesh_heading_list: vec![],
            medline_journal_info: None,
            article: None,
            other_ids: vec![],
            citation_subsets: vec![],
            gene_symbol_list: vec![],
            keyword_lists: vec![],
            chemical_list: vec![],
            investigator_list: vec![],
            coi_statement: None,
            number_of_references: None,
        }
    }

    fn chemical_list(&mut self, node: &roxmltree::Node) {
        for n in node.children().filter(|n| n.is_element()) {
            match n.tag_name().name() {
                "Chemical" => self.chemical_list.push(Chemical::new_from_xml(&n)),
                x => missing_tag_warning(&format!(
                    "Not covered in MedlineCitation::ChemicalList: '{}'",
                    x
                )),
            }
        }
    }

    fn investigator_list(&mut self, node: &roxmltree::Node) {
        for n in node.children().filter(|n| n.is_element()) {
            match n.tag_name().name() {
                "Investigator" => self.investigator_list.push(Author::new_from_xml(&n)),
                x => missing_tag_warning(&format!(
                    "Not covered in MedlineCitation::InvestigatorList: '{}'",
                    x
                )),
            }
        }
    }

    fn gene_symbol_list(&mut self, node: &roxmltree::Node) {
        for n in node.children().filter(|n| n.is_element()) {
            match n.tag_name().name() {
                "GeneSymbol" => self
                    .gene_symbol_list
                    .push(n.text().map(|v| v.to_string()).unwrap_or("".to_string())),
                x => missing_tag_warning(&format!(
                    "Not covered in MedlineCitation::GeneSymbolList: '{}'",
                    x
                )),
            }
        }
    }

    fn new_from_xml(node: &roxmltree::Node) -> Self {
        let mut ret = Self::new();
        for n in node.children().filter(|n| n.is_element()) {
            match n.tag_name().name() {
                "PMID" => match n.text() {
                    Some(id) => ret.pmid = id.parse::<u64>().unwrap_or(0),
                    None => {}
                },
                "CoiStatement" => ret.coi_statement = n.text().map(|v| v.to_string()),
                "NumberOfReferences" => ret.number_of_references = n.text().map(|v| v.to_string()),
                "KeywordList" => ret.keyword_lists.push(KeywordList::new_from_xml(&n)),
                "ChemicalList" => ret.chemical_list(&n),
                "GeneSymbolList" => ret.gene_symbol_list(&n),
                "InvestigatorList" => ret.investigator_list(&n),
                "OtherID" => ret.other_ids.push(OtherID {
                    source: n.attribute("Source").map(|v| v.to_string()),
                    id: n.text().map(|v| v.to_string()),
                }),
                "CitationSubset" => match n.text().map(|v| v.to_string()) {
                    Some(subset) => ret.citation_subsets.push(subset),
                    None => {}
                },
                "DateCompleted" => ret.date_completed = PubMedDate::new_from_xml(&n),
                "DateRevised" => ret.date_revised = PubMedDate::new_from_xml(&n),
                "Article" => ret.article = Some(Article::new_from_xml(&n)),
                "MedlineJournalInfo" => {
                    ret.medline_journal_info = Some(MedlineJournalInfo::new_from_xml(&n))
                }
                "MeshHeadingList" => {
                    ret.mesh_heading_list = n
                        .descendants()
                        .filter(|n| n.is_element() && n.tag_name().name() == "MeshHeading")
                        .filter_map(|n| MeshHeading::new_from_xml(&n))
                        .collect()
                }
                "PersonalNameSubjectList" => {
                    // TODO
                    // Example: https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=pubmed&retmode=xml&id=24332228
                }
                "GeneralNote" => {
                    // TODO
                    // Example: https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=pubmed&retmode=xml&id=12233518
                }
                "OtherAbstract" => {
                    // TODO
                    // Example: https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=pubmed&retmode=xml&id=11364263
                }
                "SupplMeshList" => {
                    // TODO
                    // Example: https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=pubmed&retmode=xml&id=14300027
                }
                "CommentsCorrectionsList" => {
                    // TODO
                    // Example: https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=pubmed&retmode=xml&id=21392701
                }
                x => missing_tag_warning(&format!("Not covered in MedlineCitation: '{}'", x)),
            }
        }
        ret
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleId {
    pub id_type: Option<String>,
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleIdList {
    pub ids: Vec<ArticleId>,
}

impl ArticleIdList {
    fn new_from_xml(node: &roxmltree::Node) -> Self {
        let mut ret = Self { ids: vec![] };
        for n in node.children().filter(|v| v.is_element()) {
            match n.tag_name().name() {
                "ArticleId" => ret.ids.push(ArticleId {
                    id_type: n.attribute("IdType").map(|v| v.to_string()),
                    id: n.text().map(|v| v.to_string()),
                }),
                x => missing_tag_warning(&format!("Not covered in ArticleIdList: '{}'", x)),
            }
        }
        ret
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    pub citation: Option<String>,
    pub article_ids: Option<ArticleIdList>,
}

impl Reference {
    fn new_from_xml(node: &roxmltree::Node) -> Self {
        let mut ret = Self {
            citation: None,
            article_ids: None,
        };
        for n in node.children().filter(|v| v.is_element()) {
            match n.tag_name().name() {
                "Citation" => ret.citation = n.text().map(|v| v.to_string()),
                "ArticleIdList" => ret.article_ids = Some(ArticleIdList::new_from_xml(&n)),
                x => missing_tag_warning(&format!("Not covered in Reference: '{}'", x)),
            }
        }
        ret
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubmedData {
    pub article_ids: Option<ArticleIdList>,
    pub history: Vec<PubMedDate>,
    pub references: Vec<Reference>,
    pub publication_status: Option<String>,
}

impl PubmedData {
    pub fn new_from_xml(node: &roxmltree::Node) -> Self {
        let mut ret = Self {
            article_ids: None,
            history: vec![],
            references: vec![],
            publication_status: None,
        };
        for n in node.children().filter(|n| n.is_element()) {
            match n.tag_name().name() {
                "ReferenceList" => ret.add_references_from_xml(&n),
                "ArticleIdList" => ret.article_ids = Some(ArticleIdList::new_from_xml(&n)),
                "PublicationStatus" => ret.publication_status = n.text().map(|v| v.to_string()),
                "History" => ret.add_history_from_xml(&n),
                x => missing_tag_warning(&format!("Not covered in PubmedData: '{}'", x)), //TODO
            }
        }
        ret
    }

    fn add_history_from_xml(&mut self, node: &roxmltree::Node) {
        for n in node.children().filter(|v| v.is_element()) {
            match n.tag_name().name() {
                "PubMedPubDate" => match PubMedDate::new_from_xml(&n) {
                    Some(date) => self.history.push(date),
                    None => {}
                },
                x => missing_tag_warning(&format!(
                    "Not covered in PubmedData::add_history_from_xml: '{}'",
                    x
                )),
            }
        }
    }

    fn add_references_from_xml(&mut self, node: &roxmltree::Node) {
        for n in node.children().filter(|v| v.is_element()) {
            match n.tag_name().name() {
                "Reference" => self.references.push(Reference::new_from_xml(&n)),
                x => println!(
                    "Not covered in PubmedData::add_references_from_xml: '{}'",
                    x
                ),
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubmedArticle {
    pub medline_citation: Option<MedlineCitation>,
    pub pubmed_data: Option<PubmedData>,
}

impl PubmedArticle {
    pub fn new_from_xml(root: &roxmltree::Node) -> Self {
        let mut ret = Self {
            medline_citation: None,
            pubmed_data: None,
        };
        for node in root.children().filter(|n| n.is_element()) {
            match node.tag_name().name() {
                "MedlineCitation" => {
                    ret.medline_citation = Some(MedlineCitation::new_from_xml(&node))
                }
                "PubmedData" => ret.pubmed_data = Some(PubmedData::new_from_xml(&node)),
                x => missing_tag_warning(&format!("Not covered in PubmedArticle: '{}'", x)),
            }
        }
        ret
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    api_key: Option<String>,
}

impl Client {
    pub fn new() -> Self {
        let mut ret = Client { api_key: None };
        match File::open("ncbi_key") {
            Ok(mut f) => {
                let mut buffer = String::new();
                match f.read_to_string(&mut buffer) {
                    Ok(_) => {
                        ret.api_key = Some(buffer);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        ret
    }

    pub fn article_ids_from_query(
        &self,
        query: &String,
        max: u64,
    ) -> Result<Vec<u64>, Box<dyn Error>> {
        let url = format!("http://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi?db=pubmed&retmode=json&retmax={}&term={}",max,query);
        //println!("PubMed::article_ids_from_query: {}", &url);
        let json: serde_json::Value = reqwest::blocking::get(url.as_str())?.json()?;
        match json["esearchresult"]["idlist"].as_array() {
            Some(idlist) => Ok(idlist
                .iter()
                .map(|id| match id.as_str() {
                    Some(x) => match x.parse::<u64>() {
                        Ok(u) => u,
                        Err(_) => {
                            println!(
                                "PubMed::article_ids_from_query: '{}' should be a numeric ID",
                                &x
                            );
                            0
                        }
                    },
                    None => 0,
                })
                .filter(|id| *id != 0)
                .collect()),
            None => Err(From::from("API error/no results")),
        }
    }

    pub fn articles(&self, ids: &Vec<u64>) -> Result<Vec<PubmedArticle>, Box<dyn Error>> {
        let ids: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
        let url = format!(
            "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=pubmed&retmode=xml&id={}",
            ids.join(",")
        );
        let text = reqwest::blocking::get(url.as_str())?.text()?;
        let doc = roxmltree::Document::parse(&text)?;
        thread::sleep(self.get_sleep_time()); // To avoid being blocked by PubMed API
        Ok(doc
            .root()
            .descendants()
            .filter(|n| n.is_element() && n.tag_name().name() == "PubmedArticle")
            .map(|n| PubmedArticle::new_from_xml(&n))
            .collect())
    }

    fn get_sleep_time(&self) -> time::Duration {
        /*
        match self.api_key {
            Some(_) => time::Duration::from_millis(120), // 10/sec with api_key
            None => time::Duration::from_millis(400),    // 3/sec without api key
        }
        */
        time::Duration::from_millis(500) // Blanket default
    }

    pub fn article(&self, id: u64) -> Result<PubmedArticle, Box<dyn Error>> {
        match self.articles(&vec![id])?.pop() {
            Some(pubmed_article) => Ok(pubmed_article),
            None => Err(From::from(format!(
                "Can't find PubmedArticle for ID '{}'",
                id
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn doi() {
        let client = super::Client::new();
        let ids = client
            .article_ids_from_query(&"\"10.1038/NATURE11174\"".to_string(), 1000)
            .unwrap();
        assert_eq!(ids, vec![22722859])
    }

    #[test]
    fn work() {
        let client = super::Client::new();
        let article = client.article(22722859).unwrap();
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

    #[test]
    fn date_parsing() {
        let client = super::Client::new();
        let article = client.article(13777676).unwrap();
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
}
