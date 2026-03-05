use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshTermPart {
    pub ui: Option<String>,
    pub major_topic: bool,
    pub name: Option<String>,
}

impl MeshTermPart {
    pub(crate) fn new_from_xml(node: &roxmltree::Node) -> Self {
        Self {
            ui: node.attribute("UI").map(|v| v.to_string()),
            major_topic: node.attribute("MajorTopicYN") == Some("Y"),
            name: node.text().map(|v| v.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshHeading {
    pub descriptor: MeshTermPart,
    pub qualifiers: Vec<MeshTermPart>,
}

impl MeshHeading {
    pub(crate) fn new_from_xml(node: &roxmltree::Node) -> Option<Self> {
        let node_descriptor = node
            .descendants()
            .find(|n| n.is_element() && n.tag_name().name() == "DescriptorName")?;
        let qualifiers = node
            .descendants()
            .filter(|n| n.is_element() && n.tag_name().name() == "QualifierName")
            .map(|n| MeshTermPart::new_from_xml(&n))
            .collect();

        Some(Self {
            descriptor: MeshTermPart::new_from_xml(&node_descriptor),
            qualifiers,
        })
    }
}
