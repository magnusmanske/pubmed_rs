use serde::{Deserialize, Serialize};

use crate::helpers::missing_tag_warning;
use crate::types::affiliation_info::AffiliationInfo;
use crate::types::identifier::Identifier;

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
    #[must_use] 
    pub fn new_from_xml(node: &roxmltree::Node) -> Self {
        let mut ret = Self {
            last_name: None,
            fore_name: None,
            initials: None,
            suffix: None,
            collective_name: None,
            affiliation_info: None,
            identifiers: vec![],
            valid: node.attribute("ValidYN") == Some("Y"),
        };
        for n in node.children().filter(roxmltree::Node::is_element) {
            match n.tag_name().name() {
                "LastName" => ret.last_name = n.text().map(std::string::ToString::to_string),
                "ForeName" => ret.fore_name = n.text().map(std::string::ToString::to_string),
                "CollectiveName" => ret.collective_name = n.text().map(std::string::ToString::to_string),
                "Initials" => ret.initials = n.text().map(std::string::ToString::to_string),
                "Suffix" => ret.suffix = n.text().map(std::string::ToString::to_string),
                "Identifier" => ret.identifiers.push(Identifier::new_from_xml(&n)),
                "AffiliationInfo" => ret.affiliation_info = Some(AffiliationInfo::new_from_xml(&n)),
                x => missing_tag_warning(&format!("Not covered in Author: '{x}'")),
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
    #[must_use] 
    pub fn new_from_xml(node: &roxmltree::Node) -> Self {
        Self {
            complete: node.attribute("CompleteYN") == Some("Y"),
            authors: node
                .descendants()
                .filter(|n| n.is_element() && n.tag_name().name() == "Author")
                .map(|n| Author::new_from_xml(&n))
                .collect(),
        }
    }
}
