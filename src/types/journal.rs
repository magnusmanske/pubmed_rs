use serde::{Deserialize, Serialize};

use crate::helpers::missing_tag_warning;
use crate::types::journal_issue::JournalIssue;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Journal {
    pub issn: Option<String>,
    pub issn_type: Option<String>,
    pub journal_issue: Option<JournalIssue>,
    pub title: Option<String>,
    pub iso_abbreviation: Option<String>,
}

impl Journal {
    #[must_use] 
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use] 
    pub fn new_from_xml(node: &roxmltree::Node) -> Self {
        let mut ret = Self::default();
        for n in node.children().filter(roxmltree::Node::is_element) {
            match n.tag_name().name() {
                "ISSN" => {
                    ret.issn = n.text().map(std::string::ToString::to_string);
                    ret.issn_type = n.attribute("IssnType").map(std::string::ToString::to_string);
                }
                "JournalIssue" => ret.journal_issue = Some(JournalIssue::new_from_xml(&n)),
                "Title" => ret.title = n.text().map(std::string::ToString::to_string),
                "ISOAbbreviation" => ret.iso_abbreviation = n.text().map(std::string::ToString::to_string),
                x => missing_tag_warning(&format!("Not covered in Journal: '{x}'")),
            }
        }
        ret
    }
}
