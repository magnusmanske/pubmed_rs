use serde::{Deserialize, Serialize};

use crate::helpers::missing_tag_warning;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleId {
    pub id_type: Option<String>,
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleIdList {
    pub ids: Vec<ArticleId>,
}

impl ArticleIdList {
    pub(crate) fn new_from_xml(node: &roxmltree::Node) -> Self {
        let mut ret = Self { ids: vec![] };
        for n in node.children().filter(roxmltree::Node::is_element) {
            match n.tag_name().name() {
                "ArticleId" => ret.ids.push(ArticleId {
                    id_type: n.attribute("IdType").map(std::string::ToString::to_string),
                    id: n.text().map(std::string::ToString::to_string),
                }),
                x => missing_tag_warning(&format!("Not covered in ArticleIdList: '{x}'")),
            }
        }
        ret
    }
}
