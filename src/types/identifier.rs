use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identifier {
    pub id: Option<String>,
    pub source: Option<String>,
}

impl Identifier {
    #[must_use] 
    pub fn new_from_xml(node: &roxmltree::Node) -> Self {
        Self {
            id: node.text().map(std::string::ToString::to_string),
            source: node.attribute("Source").map(std::string::ToString::to_string),
        }
    }
}
