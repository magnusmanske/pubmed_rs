use serde::{Deserialize, Serialize};

use crate::helpers::missing_tag_warning;

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
