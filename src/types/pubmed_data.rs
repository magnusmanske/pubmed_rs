use serde::{Deserialize, Serialize};

use crate::helpers::missing_tag_warning;
use crate::types::article_id::ArticleIdList;
use crate::types::pubmed_date::PubMedDate;
use crate::types::reference::Reference;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubmedData {
    pub article_ids: Option<ArticleIdList>,
    pub history: Vec<PubMedDate>,
    pub references: Vec<Reference>,
    pub publication_status: Option<String>,
}

impl PubmedData {
    #[must_use] 
    pub fn new_from_xml(node: &roxmltree::Node) -> Self {
        let mut ret = Self {
            article_ids: None,
            history: vec![],
            references: vec![],
            publication_status: None,
        };
        for n in node.children().filter(roxmltree::Node::is_element) {
            match n.tag_name().name() {
                "ReferenceList" => ret.add_references_from_xml(&n),
                "ArticleIdList" => ret.article_ids = Some(ArticleIdList::new_from_xml(&n)),
                "PublicationStatus" => ret.publication_status = n.text().map(std::string::ToString::to_string),
                "History" => ret.add_history_from_xml(&n),
                x => missing_tag_warning(&format!("Not covered in PubmedData: '{x}'")),
            }
        }
        ret
    }

    fn add_history_from_xml(&mut self, node: &roxmltree::Node) {
        for n in node.children().filter(roxmltree::Node::is_element) {
            match n.tag_name().name() {
                "PubMedPubDate" => {
                    if let Some(date) = PubMedDate::new_from_xml(&n) {
                        self.history.push(date);
                    }
                }
                x => missing_tag_warning(&format!(
                    "Not covered in PubmedData::add_history_from_xml: '{x}'"
                )),
            }
        }
    }

    fn add_references_from_xml(&mut self, node: &roxmltree::Node) {
        for n in node.children().filter(roxmltree::Node::is_element) {
            match n.tag_name().name() {
                "Reference" => self.references.push(Reference::new_from_xml(&n)),
                "Title" => {} // ReferenceList can contain a Title element; ignored
                x => missing_tag_warning(&format!(
                    "Not covered in PubmedData::add_references_from_xml: '{x}'"
                )),
            }
        }
    }
}
