extern crate roxmltree;

use reqwest;
//use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubMedDate {
    pub year: u32,
    pub month: u8,
    pub day: u8,
}

impl PubMedDate {
    fn new_from_xml(node: &roxmltree::Node) -> Option<PubMedDate> {
        let mut ret = Self {
            year: 0,
            month: 0,
            day: 0,
        };

        for n in node.children().filter(|n| n.is_element()) {
            match n.tag_name().name() {
                "Year" => {
                    ret.year = n
                        .text()
                        .map_or(0, |v| v.to_string().parse::<u32>().unwrap_or(0))
                }
                "Month" => {
                    ret.month = n
                        .text()
                        .map_or(0, |v| v.to_string().parse::<u8>().unwrap_or(0))
                }
                "Day" => {
                    ret.day = n
                        .text()
                        .map_or(0, |v| v.to_string().parse::<u8>().unwrap_or(0))
                }
                _ => {}
            }
        }
        match ret.precision() {
            0 => None,
            _ => Some(ret),
        }
    }

    // 11=day, 10=month, 9=year
    pub fn precision(&self) -> u8 {
        if self.year == 0 {
            0
        } else if self.month == 0 {
            9
        } else if self.day == 0 {
            10
        } else {
            11
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
    fn new_from_xml(node: &roxmltree::Node) -> Self {
        let node_descriptor = node
            .descendants()
            .filter(|n| n.is_element() && n.tag_name().name() == "DescriptorName")
            .next()
            .unwrap();
        let qualifiers = node
            .descendants()
            .filter(|n| n.is_element() && n.tag_name().name() == "QualifierName")
            .map(|n| MeshTermPart::new_from_xml(&n))
            .collect();

        Self {
            descriptor: MeshTermPart::new_from_xml(&node_descriptor),
            qualifiers: qualifiers,
        }
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
                .map(|n| n.text().or(Some("")).unwrap().to_string())
                .next(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffiliationInfo {
    affiliation: Option<String>,
}

impl AffiliationInfo {
    pub fn new_from_xml(node: &roxmltree::Node) -> Self {
        let mut ret = Self { affiliation: None };
        for n in node.children().filter(|n| n.is_element()) {
            match n.tag_name().name() {
                "Affiliation" => ret.affiliation = n.text().map(|v| v.to_string()),
                x => println!("Not covered in AffiliationInfo: '{}'", x),
            }
        }
        ret
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    last_name: Option<String>,
    fore_name: Option<String>,
    initials: Option<String>,
    affiliation_info: Option<AffiliationInfo>,
    valid: bool,
}

impl Author {
    pub fn new_from_xml(node: &roxmltree::Node) -> Self {
        let mut ret = Self {
            last_name: None,
            fore_name: None,
            initials: None,
            affiliation_info: None,
            valid: node.attribute("ValidYN").map_or(false, |v| v == "Y"),
        };
        for n in node.children().filter(|n| n.is_element()) {
            match n.tag_name().name() {
                "LastName" => ret.last_name = n.text().map(|v| v.to_string()),
                "ForeName" => ret.fore_name = n.text().map(|v| v.to_string()),
                "Initials" => ret.initials = n.text().map(|v| v.to_string()),
                "AffiliationInfo" => ret.affiliation_info = Some(AffiliationInfo::new_from_xml(&n)),

                x => println!("Not covered in Author: '{}'", x),
            }
        }
        ret
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorList {
    authors: Vec<Author>,
    complete: bool,
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
    cited_medium: Option<String>,
    volume: Option<String>,
    issue: Option<String>,
    pub_date: Option<PubMedDate>,
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
                "PubDate" => ret.pub_date = PubMedDate::new_from_xml(&n),
                "Volume" => ret.volume = n.text().map(|v| v.to_string()),
                "Issue" => ret.issue = n.text().map(|v| v.to_string()),
                x => println!("Not covered in JournalIssue: '{}'", x),
            }
        }
        ret
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Journal {
    issn: Option<String>,
    issn_type: Option<String>,
    journal_issue: Option<JournalIssue>,
    title: Option<String>,
    iso_abbreviation: Option<String>,
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
                x => println!("Not covered in Journal: '{}'", x),
            }
        }
        ret
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Pagination {
    MedlinePgn(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grant {
    grant_id: Option<String>,
    agency: Option<String>,
    country: Option<String>,
}

impl Grant {
    pub fn new_from_xml(node: &roxmltree::Node) -> Self {
        let mut ret = Self {
            grant_id: None,
            agency: None,
            country: None,
        };
        for n in node.children().filter(|n| n.is_element()) {
            match n.tag_name().name() {
                "GrantID" => ret.grant_id = n.text().map(|v| v.to_string()),
                "Agency" => ret.agency = n.text().map(|v| v.to_string()),
                "Country" => ret.country = n.text().map(|v| v.to_string()),
                x => println!("Not covered in Grant: '{}'", x),
            }
        }
        ret
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrantList {
    grants: Vec<Grant>,
    complete: bool,
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
pub struct Article {
    pub_model: Option<String>,
    journal: Option<Journal>,
    title: Option<String>,
    pagination: Vec<Pagination>,
    e_location_ids: Vec<ELocationID>,
    the_abstract: Option<Abstract>,
    author_list: Option<AuthorList>,
    language: Option<String>,
    grant_list: Option<GrantList>,
    //publication_type_list:PublicationTypeList,
    //article_date:ArticleDate,
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
            grant_list: None,
            //publication_type_list:PublicationTypeList,
            //article_date:ArticleDate,
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
                                n2.text().or(Some("")).unwrap().to_string(),
                            )),
                            x => println!("Not covered in Pagination: '{}'", x),
                        }
                    }
                }
                "ELocationID" => ret.e_location_ids.push(ELocationID::new_from_xml(&n)),
                "Abstract" => ret.the_abstract = Some(Abstract::new_from_xml(&n)),
                "AuthorList" => ret.author_list = Some(AuthorList::new_from_xml(&n)),
                "Language" => ret.language = n.text().map(|v| v.to_string()),
                "GrantList" => ret.grant_list = Some(GrantList::new_from_xml(&n)),
                //"PublicationTypeList" => {}
                //"ArticleDate" => {}
                x => println!("Not covered in Article: '{}'", x),
            }
        }
        ret
    }
}

//____________________________________________________________________________________________________

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Work {
    pmid: u64,
    date_completed: Option<PubMedDate>,
    date_revised: Option<PubMedDate>,
    mesh_heading_list: Vec<MeshHeading>,
    article: Option<Article>,
}

impl Work {
    pub fn new() -> Self {
        Self {
            pmid: 0,
            date_completed: None,
            date_revised: None,
            mesh_heading_list: vec![],
            article: None,
        }
    }

    fn import_medline_citation_from_xml(&mut self, root: &roxmltree::Node) {
        for node in root.children().filter(|n| n.is_element()) {
            match node.tag_name().name() {
                "PMID" => match node.text() {
                    Some(id) => self.pmid = id.parse::<u64>().unwrap(),
                    None => {}
                },
                "DateCompleted" => self.date_completed = PubMedDate::new_from_xml(&node),
                "DateRevised" => self.date_revised = PubMedDate::new_from_xml(&node),
                "Article" => self.article = Some(Article::new_from_xml(&node)),
                "MeshHeadingList" => {
                    self.mesh_heading_list = node
                        .descendants()
                        .filter(|n| n.is_element() && n.tag_name().name() == "MeshHeading")
                        .map(|n| MeshHeading::new_from_xml(&n))
                        .collect()
                }
                x => println!("Not covered in MedlineCitation: '{}'", x),
            }
        }
    }

    fn import_pubmed_data_from_xml(&mut self, root: &roxmltree::Node) {
        for node in root.descendants().filter(|n| n.is_element()) {
            match node.tag_name().name() {
                _x => {} //println!("Not covered in PubmedData: '{}'", x),//TODO
            }
        }
    }

    pub fn new_from_xml(root: &roxmltree::Node) -> Self {
        let mut ret = Work::new();
        for node in root.children().filter(|n| n.is_element()) {
            match node.tag_name().name() {
                "MedlineCitation" => ret.import_medline_citation_from_xml(&node),
                "PubmedData" => ret.import_pubmed_data_from_xml(&node),
                x => println!("Not covered in Work: '{}'", x),
            }
        }
        ret
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {}

impl Client {
    pub fn new() -> Self {
        Client {}
    }

    pub fn work_ids_from_query(
        &self,
        query: &String,
        max: u64,
    ) -> Result<Vec<u64>, Box<::std::error::Error>> {
        let url = "http://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi?db=pubmed&retmode=json"
            .to_string()
            + "&retmax="
            + &max.to_string()
            + "&term=" + query;
        let json: serde_json::Value = reqwest::get(url.as_str())?.json()?;
        match json["esearchresult"]["idlist"].as_array() {
            Some(idlist) => Ok(idlist
                .iter()
                .map(|id| id.as_str().unwrap().parse::<u64>().unwrap())
                .collect()),
            None => Err(From::from("API error/no results")),
        }
    }

    pub fn works(&self, ids: &Vec<u64>) -> Result<Vec<Work>, Box<::std::error::Error>> {
        let ids: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
        let url =
            "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=pubmed&retmode=xml&id="
                .to_string()
                + &ids.join(",");
        let text = reqwest::get(url.as_str())?.text()?;
        let doc = roxmltree::Document::parse(&text)?;
        Ok(doc
            .root()
            .descendants()
            .filter(|n| n.is_element() && n.tag_name().name() == "PubmedArticle")
            .map(|n| Work::new_from_xml(&n))
            .collect())
    }

    pub fn work(&self, id: u64) -> Result<Work, Box<::std::error::Error>> {
        match self.works(&vec![id])?.pop() {
            Some(work) => Ok(work),
            None => Err(From::from(format!("Can't find work for ID '{}'", id))),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_doi() {
        let client = super::Client::new();
        let ids = client
            .work_ids_from_query(&"\"10.1038/NATURE11174\"".to_string(), 1000)
            .unwrap();
        assert_eq!(ids, vec![22722859])
    }

    #[test]
    fn test_work() {
        let client = super::Client::new();
        let work = client.work(22722859).unwrap();
        let date = work.date_completed.unwrap().clone();
        assert_eq!(date.year, 2012);
        assert_eq!(date.month, 8);
        assert_eq!(date.day, 17);
    }
}
