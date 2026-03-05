use serde::{Deserialize, Serialize};

use crate::helpers::missing_tag_warning;
use crate::types::pubmed_date::PubMedDate;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JournalIssue {
    pub cited_medium: Option<String>,
    pub volume: Option<String>,
    pub issue: Option<String>,
    pub pub_date: Option<PubMedDate>,
}

impl JournalIssue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_from_xml(node: &roxmltree::Node) -> Self {
        let mut ret = Self {
            cited_medium: node.attribute("CitedMedium").map(|v| v.to_string()),
            ..Default::default()
        };
        for n in node.children().filter(|n| n.is_element()) {
            match n.tag_name().name() {
                "PubDate" => {
                    ret.pub_date = PubMedDate::new_from_xml(&n);
                }
                "Volume" => ret.volume = n.text().map(|v| v.to_string()),
                "Issue" => ret.issue = n.text().map(|v| v.to_string()),
                x => missing_tag_warning(&format!("Not covered in JournalIssue: '{}'", x)),
            }
        }
        ret
    }
}
