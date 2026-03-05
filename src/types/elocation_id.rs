use serde::{Deserialize, Serialize};

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
            valid: node.attribute("ValidYN") == Some("Y"),
            id: node.text().map(|v| v.to_string()),
        }
    }
}
