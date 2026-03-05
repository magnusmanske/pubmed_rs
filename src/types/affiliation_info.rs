use serde::{Deserialize, Serialize};

use crate::helpers::missing_tag_warning;
use crate::types::identifier::Identifier;

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
