use serde::{Deserialize, Serialize};

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
