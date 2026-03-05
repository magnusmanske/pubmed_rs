use serde::{Deserialize, Serialize};

use crate::helpers::missing_tag_warning;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grant {
    pub grant_id: Option<String>,
    pub agency: Option<String>,
    pub country: Option<String>,
    pub acronym: Option<String>,
}

impl Grant {
    #[must_use] 
    pub fn new_from_xml(node: &roxmltree::Node) -> Self {
        let mut ret = Self {
            grant_id: None,
            agency: None,
            country: None,
            acronym: None,
        };
        for n in node.children().filter(roxmltree::Node::is_element) {
            match n.tag_name().name() {
                "GrantID" => ret.grant_id = n.text().map(std::string::ToString::to_string),
                "Agency" => ret.agency = n.text().map(std::string::ToString::to_string),
                "Country" => ret.country = n.text().map(std::string::ToString::to_string),
                "Acronym" => ret.acronym = n.text().map(std::string::ToString::to_string),
                x => missing_tag_warning(&format!("Not covered in Grant: '{x}'")),
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
    #[must_use] 
    pub fn new_from_xml(node: &roxmltree::Node) -> Self {
        Self {
            complete: node.attribute("CompleteYN") == Some("Y"),
            grants: node
                .descendants()
                .filter(|n| n.is_element() && n.tag_name().name() == "Grant")
                .map(|n| Grant::new_from_xml(&n))
                .collect(),
        }
    }
}
