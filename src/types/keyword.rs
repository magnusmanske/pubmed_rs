use serde::{Deserialize, Serialize};

use crate::helpers::missing_tag_warning;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyword {
    pub keyword: String,
    pub major_topic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeywordList {
    pub owner: Option<String>,
    pub keywords: Vec<Keyword>,
}

impl KeywordList {
    #[must_use] 
    pub fn new_from_xml(node: &roxmltree::Node) -> Self {
        let mut ret = Self {
            owner: node.attribute("Owner").map(std::string::ToString::to_string),
            keywords: vec![],
        };
        for n in node.children().filter(roxmltree::Node::is_element) {
            match n.tag_name().name() {
                "Keyword" => {
                    ret.keywords.push(Keyword {
                        major_topic: n.attribute("MajorTopicYN") == Some("Y"),
                        keyword: n.text().unwrap_or("").to_string(),
                    });
                }
                x => missing_tag_warning(&format!("Not covered in KeywordList: '{x}'")),
            }
        }
        ret
    }
}
