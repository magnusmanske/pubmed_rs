use serde::{Deserialize, Serialize};

use crate::helpers::missing_tag_warning;
use crate::types::medline_citation::MedlineCitation;
use crate::types::pubmed_data::PubmedData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubmedArticle {
    pub medline_citation: Option<MedlineCitation>,
    pub pubmed_data: Option<PubmedData>,
}

impl PubmedArticle {
    pub fn new_from_xml(root: &roxmltree::Node) -> Self {
        let mut ret = Self {
            medline_citation: None,
            pubmed_data: None,
        };
        for node in root.children().filter(|n| n.is_element()) {
            match node.tag_name().name() {
                "MedlineCitation" => {
                    ret.medline_citation = Some(MedlineCitation::new_from_xml(&node))
                }
                "PubmedData" => ret.pubmed_data = Some(PubmedData::new_from_xml(&node)),
                x => missing_tag_warning(&format!("Not covered in PubmedArticle: '{}'", x)),
            }
        }
        ret
    }
}
