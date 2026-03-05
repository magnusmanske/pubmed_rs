use serde::{Deserialize, Serialize};

use crate::helpers::missing_tag_warning;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chemical {
    pub registry_number: Option<String>,
    pub name_of_substance: Option<String>,
    pub name_of_substance_ui: Option<String>,
}

impl Chemical {
    #[must_use] 
    pub fn new_from_xml(node: &roxmltree::Node) -> Self {
        let mut ret = Self {
            registry_number: None,
            name_of_substance: None,
            name_of_substance_ui: None,
        };
        for n in node.children().filter(roxmltree::Node::is_element) {
            match n.tag_name().name() {
                "RegistryNumber" => ret.registry_number = n.text().map(std::string::ToString::to_string),
                "NameOfSubstance" => {
                    ret.name_of_substance = n.text().map(std::string::ToString::to_string);
                    ret.name_of_substance_ui = n.attribute("UI").map(std::string::ToString::to_string);
                }
                x => missing_tag_warning(&format!("Not covered in Chemical: '{x}'")),
            }
        }
        ret
    }
}
