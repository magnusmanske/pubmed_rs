use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Abstract {
    pub text: Option<String>,
}

impl Abstract {
    pub fn new_from_xml(node: &roxmltree::Node) -> Self {
        Self {
            text: node
                .descendants()
                .filter(|n| n.is_element() && n.tag_name().name() == "AbstractText")
                .map(|n| n.text().unwrap_or("").to_string())
                .next(),
        }
    }
}
