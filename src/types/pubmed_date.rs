use serde::{Deserialize, Serialize};

use crate::helpers::missing_tag_warning;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubMedDate {
    pub year: u32,
    pub month: u8,
    pub day: u8,
    pub hour: i8,
    pub minute: i8,
    pub date_type: Option<String>,
    pub pub_status: Option<String>,
}

impl PubMedDate {
    pub(crate) fn new_from_xml(node: &roxmltree::Node) -> Option<PubMedDate> {
        let mut ret = Self {
            year: 0,
            month: 0,
            day: 0,
            hour: -1,
            minute: -1,
            date_type: node.attribute("DateType").map(|v| v.to_string()),
            pub_status: node.attribute("PubStatus").map(|v| v.to_string()),
        };

        for n in node.children().filter(|n| n.is_element()) {
            match n.tag_name().name() {
                "MedlineDate" => {} // TODO
                "Year" => ret.year = n.text().map_or(0, |v| v.parse::<u32>().unwrap_or(0)),
                "Month" => ret.month = Self::parse_month_from_xml(&n),
                "Day" => ret.day = n.text().map_or(0, |v| v.parse::<u8>().unwrap_or(0)),
                "Hour" => ret.hour = n.text().map_or(-1, |v| v.parse::<i8>().unwrap_or(-1)),
                "Minute" => ret.minute = n.text().map_or(-1, |v| v.parse::<i8>().unwrap_or(-1)),
                "Season" => {
                    // TODO
                    // Example: https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=pubmed&retmode=xml&id=11364263
                }
                x => missing_tag_warning(&format!("Not covered in PubMedDate: '{}'", x)),
            }
        }
        match ret.precision() {
            0 => None,
            _ => Some(ret),
        }
    }

    fn parse_month_from_xml(node: &roxmltree::Node) -> u8 {
        match node.text() {
            Some(t) => match t.to_lowercase().as_str() {
                "jan" => 1,
                "feb" => 2,
                "mar" => 3,
                "apr" => 4,
                "may" => 5,
                "jun" => 6,
                "jul" => 7,
                "aug" => 8,
                "sep" => 9,
                "oct" => 10,
                "nov" => 11,
                "dec" => 12,
                other => other.parse::<u8>().unwrap_or(0),
            },
            None => 0,
        }
    }

    // 13=minute, 12=hour, 11=day, 10=month, 9=year; same as Wikidata/wikibase
    pub fn precision(&self) -> u8 {
        if self.year == 0 {
            0
        } else if self.month == 0 {
            9
        } else if self.day == 0 {
            10
        } else if self.hour == -1 {
            11
        } else if self.minute == -1 {
            12
        } else {
            13
        }
    }
}
