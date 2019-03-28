extern crate roxmltree;

use date;
use reqwest;
use serde_json;

type PubMedDate = Option<date::Date>;

#[derive(Debug, Clone)]
pub struct MeshTermPart {
    pub ui: String,
    pub major_topic: bool,
    pub name: String,
}

impl MeshTermPart {
    fn new_from_xml(node: &roxmltree::Node) -> MeshTermPart {
        MeshTermPart {
            ui: node.attribute("UI").or(Some("")).unwrap().to_string(),
            major_topic: node.attribute("MajorTopicYN").map_or(false, |v| v == "Y"),
            name: node.text().unwrap().to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MeshHeading {
    pub descriptor: MeshTermPart,
    pub qualifiers: Vec<MeshTermPart>,
}

impl MeshHeading {
    fn new_from_xml(node: &roxmltree::Node) -> MeshHeading {
        let node_descriptor = node
            .descendants()
            .filter(|n| n.is_element() && n.tag_name().name() == "DescriptorName")
            .next()
            .unwrap();
        let qualifiers = node
            .descendants()
            .filter(|n| n.is_element() && n.tag_name().name() == "QualifierName")
            .map(|n| MeshTermPart::new_from_xml(&n))
            .collect();

        MeshHeading {
            descriptor: MeshTermPart::new_from_xml(&node_descriptor),
            qualifiers: qualifiers,
        }
    }
}

/////////

#[derive(Debug, Clone)]
pub struct ELocationID {
    e_id_type: String,
    valid: bool,
    id: String,
}

impl ELocationID {
    pub fn new_from_xml(node: &roxmltree::Node) -> ELocationID {
        ELocationID {
            e_id_type: node.attribute("EIdType").or(Some("")).unwrap().to_string(),
            valid: node.attribute("ValidYN").map_or(false, |v| v == "Y"),
            id: node.text().unwrap().to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Article {
    pub_model: String,
    //journal:Journal,
    title: String,
    //pagination:Pagination,
    e_location_ids: Vec<ELocationID>,
    //abstract:Abstract,
    //author_list:AuthorList,
    language: String,
    //grant_list:GrantList,
    //publication_type_list:PublicationTypeList,
    //article_date:ArticleDate,
}

impl Article {
    pub fn new() -> Article {
        Article {
            pub_model: "".to_string(),
            //journal:Journal,
            title: "".to_string(),
            //pagination:Pagination,
            e_location_ids: vec![],
            //abstract:Abstract,
            //author_list:AuthorList,
            language: "".to_string(),
            //grant_list:GrantList,
            //publication_type_list:PublicationTypeList,
            //article_date:ArticleDate,
        }
    }

    pub fn new_from_xml(node: &roxmltree::Node) -> Article {
        let mut ret = Article::new();
        ret.pub_model = node.attribute("PubModel").or(Some("")).unwrap().to_string();
        ret
    }
}

#[derive(Debug, Clone)]
pub struct Work {
    pmid: u64,
    date_completed: PubMedDate,
    date_revised: PubMedDate,
    mesh_heading_list: Vec<MeshHeading>,
}

impl Work {
    pub fn new() -> Work {
        Work {
            pmid: 0,
            date_completed: None,
            date_revised: None,
            mesh_heading_list: vec![],
        }
    }

    fn first_node_as_text(node: &roxmltree::Node, tag_name: &str) -> String {
        node.descendants()
            .filter(|n| n.is_element() && n.tag_name().name() == tag_name)
            .next()
            .unwrap()
            .text()
            .unwrap()
            .to_string()
    }

    fn date_from_xml(node: &roxmltree::Node) -> PubMedDate {
        // No year?
        if node
            .descendants()
            .filter(|n| n.is_element() && n.tag_name().name() == "Year")
            .count()
            == 0
        {
            return None;
        }

        // No day?
        if node
            .descendants()
            .filter(|n| n.is_element() && n.tag_name().name() == "Day")
            .count()
            == 0
        {
            return Some(date::Date::new(
                Work::first_node_as_text(&node, "Year")
                    .parse::<u32>()
                    .unwrap(),
                0,
                0,
            ));
        }

        // Year, Month, Day
        Some(date::Date::new(
            Work::first_node_as_text(&node, "Year")
                .parse::<u32>()
                .unwrap(),
            Work::first_node_as_text(&node, "Month")
                .parse::<u8>()
                .unwrap(),
            Work::first_node_as_text(&node, "Day")
                .parse::<u8>()
                .unwrap(),
        ))
    }

    pub fn new_from_xml(root: &roxmltree::Node) -> Work {
        let mut ret = Work::new();
        for node in root.descendants() {
            if node.is_element() {
                match node.tag_name().name() {
                    "PMID" => match node.text() {
                        Some(id) => ret.pmid = id.parse::<u64>().unwrap(),
                        None => {}
                    },
                    "DateCompleted" => ret.date_completed = Self::date_from_xml(&node),
                    "DateRevised" => ret.date_revised = Self::date_from_xml(&node),
                    "MeshHeadingList" => {
                        ret.mesh_heading_list = node
                            .descendants()
                            .filter(|n| n.is_element() && n.tag_name().name() == "MeshHeading")
                            .map(|n| MeshHeading::new_from_xml(&n))
                            .collect()
                    }
                    _ => {}
                }
            }
        }
        ret
    }
}

#[derive(Debug, Clone)]
pub struct Client {}

impl Client {
    pub fn new() -> Client {
        Client {}
    }

    pub fn work_ids_from_query(
        &self,
        query: &String,
        max: u64,
    ) -> Result<Vec<u64>, Box<::std::error::Error>> {
        let url = "http://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi?db=pubmed&retmode=json"
            .to_string()
            + "&retmax="
            + &max.to_string()
            + "&term=" + query;
        let json: serde_json::Value = reqwest::get(url.as_str())?.json()?;
        match json["esearchresult"]["idlist"].as_array() {
            Some(idlist) => Ok(idlist
                .iter()
                .map(|id| id.as_str().unwrap().parse::<u64>().unwrap())
                .collect()),
            None => Err(From::from("API error/no results")),
        }
    }

    pub fn works(&self, ids: &Vec<u64>) -> Result<Vec<Work>, Box<::std::error::Error>> {
        let ids: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
        let url =
            "https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=pubmed&retmode=xml&id="
                .to_string()
                + &ids.join(",");
        let text = reqwest::get(url.as_str())?.text()?;
        let doc = roxmltree::Document::parse(&text)?;
        Ok(doc
            .root()
            .descendants()
            .filter(|n| n.is_element() && n.tag_name().name() == "PubmedArticle")
            .map(|n| Work::new_from_xml(&n))
            .collect())
    }

    pub fn work(&self, id: u64) -> Result<Work, Box<::std::error::Error>> {
        match self.works(&vec![id])?.pop() {
            Some(work) => Ok(work),
            None => Err(From::from(format!("Can't find work for ID '{}'", id))),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_doi() {
        let client = super::Client::new();
        let ids = client
            .work_ids_from_query(&"\"10.1038/NATURE11174\"".to_string(), 1000)
            .unwrap();
        assert_eq!(ids, vec![22722859])
    }

    #[test]
    fn test_work() {
        let client = super::Client::new();
        let work = client.work(22722859).unwrap();
        assert_eq!(work.date_completed.unwrap().year, 2012);
        assert_eq!(work.date_completed.unwrap().month, 8);
        assert_eq!(work.date_completed.unwrap().day, 17);
    }
}
