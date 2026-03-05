use serde::{Deserialize, Serialize};

use crate::helpers::missing_tag_warning;
use crate::types::article_id::ArticleIdList;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    pub citation: Option<String>,
    pub article_ids: Option<ArticleIdList>,
}

impl Reference {
    pub(crate) fn new_from_xml(node: &roxmltree::Node) -> Self {
        let mut ret = Self {
            citation: None,
            article_ids: None,
        };
        for n in node.children().filter(roxmltree::Node::is_element) {
            match n.tag_name().name() {
                "Citation" => ret.citation = n.text().map(std::string::ToString::to_string),
                "ArticleIdList" => ret.article_ids = Some(ArticleIdList::new_from_xml(&n)),
                x => missing_tag_warning(&format!("Not covered in Reference: '{x}'")),
            }
        }
        ret
    }
}
