use serde::{Deserialize, Serialize};

use crate::helpers::missing_tag_warning;
use crate::types::article_abstract::Abstract;
use crate::types::author::AuthorList;
use crate::types::elocation_id::ELocationID;
use crate::types::grant::GrantList;
use crate::types::journal::Journal;
use crate::types::pagination::Pagination;
use crate::types::publication_type::PublicationType;
use crate::types::pubmed_date::PubMedDate;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
        Self::default()
    }

    pub fn new_from_xml(node: &roxmltree::Node) -> Self {
        let mut ret = Article {
            pub_model: node.attribute("PubModel").map(|v| v.to_string()),
            ..Default::default()
        };
        for n in node.children().filter(|n| n.is_element()) {
            match n.tag_name().name() {
                "ArticleTitle" => ret.title = n.text().map(|v| v.to_string()),
                "Journal" => ret.journal = Some(Journal::new_from_xml(&n)),
                "Pagination" => {
                    for n2 in n.children().filter(|n| n.is_element()) {
                        match n2.tag_name().name() {
                            "MedlinePgn" => ret
                                .pagination
                                .push(Pagination::MedlinePgn(n2.text().unwrap_or("").to_string())),
                            "StartPage" => {} // TODO
                            "EndPage" => {}   // TODO
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
                "ArticleDate" => {
                    if let Some(date) = PubMedDate::new_from_xml(&n) {
                        ret.article_date.push(date)
                    }
                }
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
