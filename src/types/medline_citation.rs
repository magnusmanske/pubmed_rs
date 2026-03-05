use serde::{Deserialize, Serialize};

use crate::helpers::missing_tag_warning;
use crate::types::article::Article;
use crate::types::author::Author;
use crate::types::chemical::Chemical;
use crate::types::keyword::KeywordList;
use crate::types::medline_journal_info::MedlineJournalInfo;
use crate::types::mesh::MeshHeading;
use crate::types::other_id::OtherID;
use crate::types::pubmed_date::PubMedDate;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MedlineCitation {
    pub pmid: u64,
    pub date_completed: Option<PubMedDate>,
    pub date_revised: Option<PubMedDate>,
    pub mesh_heading_list: Vec<MeshHeading>,
    pub medline_journal_info: Option<MedlineJournalInfo>,
    pub article: Option<Article>,
    pub other_ids: Vec<OtherID>,
    pub citation_subsets: Vec<String>,
    pub gene_symbol_list: Vec<String>,
    pub keyword_lists: Vec<KeywordList>,
    pub chemical_list: Vec<Chemical>,
    pub investigator_list: Vec<Author>,
    pub coi_statement: Option<String>,
    pub number_of_references: Option<String>,
}

impl MedlineCitation {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn parse_chemical_list(&mut self, node: &roxmltree::Node) {
        for n in node.children().filter(roxmltree::Node::is_element) {
            match n.tag_name().name() {
                "Chemical" => self.chemical_list.push(Chemical::new_from_xml(&n)),
                x => missing_tag_warning(&format!(
                    "Not covered in MedlineCitation::ChemicalList: '{x}'"
                )),
            }
        }
    }

    fn parse_investigator_list(&mut self, node: &roxmltree::Node) {
        for n in node.children().filter(roxmltree::Node::is_element) {
            match n.tag_name().name() {
                "Investigator" => self.investigator_list.push(Author::new_from_xml(&n)),
                x => missing_tag_warning(&format!(
                    "Not covered in MedlineCitation::InvestigatorList: '{x}'"
                )),
            }
        }
    }

    fn parse_gene_symbol_list(&mut self, node: &roxmltree::Node) {
        for n in node.children().filter(roxmltree::Node::is_element) {
            match n.tag_name().name() {
                "GeneSymbol" => self
                    .gene_symbol_list
                    .push(n.text().unwrap_or("").to_string()),
                x => missing_tag_warning(&format!(
                    "Not covered in MedlineCitation::GeneSymbolList: '{x}'"
                )),
            }
        }
    }

    pub(crate) fn new_from_xml(node: &roxmltree::Node) -> Self {
        let mut ret = Self::default();
        for n in node.children().filter(roxmltree::Node::is_element) {
            match n.tag_name().name() {
                "PMID" => {
                    if let Some(id) = n.text() {
                        ret.pmid = id.parse::<u64>().unwrap_or(0);
                    }
                }
                "CoiStatement" => {
                    ret.coi_statement = n.text().map(std::string::ToString::to_string)
                }
                "NumberOfReferences" => {
                    ret.number_of_references = n.text().map(std::string::ToString::to_string)
                }
                "KeywordList" => ret.keyword_lists.push(KeywordList::new_from_xml(&n)),
                "ChemicalList" => ret.parse_chemical_list(&n),
                "GeneSymbolList" => ret.parse_gene_symbol_list(&n),
                "InvestigatorList" => ret.parse_investigator_list(&n),
                "OtherID" => ret.other_ids.push(OtherID {
                    source: n.attribute("Source").map(std::string::ToString::to_string),
                    id: n.text().map(std::string::ToString::to_string),
                }),
                "CitationSubset" => {
                    if let Some(subset) = n.text().map(std::string::ToString::to_string) {
                        ret.citation_subsets.push(subset);
                    }
                }
                "DateCompleted" => ret.date_completed = PubMedDate::new_from_xml(&n),
                "DateRevised" => ret.date_revised = PubMedDate::new_from_xml(&n),
                "Article" => ret.article = Some(Article::new_from_xml(&n)),
                "MedlineJournalInfo" => {
                    ret.medline_journal_info = Some(MedlineJournalInfo::new_from_xml(&n));
                }
                "MeshHeadingList" => {
                    ret.mesh_heading_list = n
                        .descendants()
                        .filter(|n| n.is_element() && n.tag_name().name() == "MeshHeading")
                        .filter_map(|n| MeshHeading::new_from_xml(&n))
                        .collect();
                }
                "PersonalNameSubjectList"
                | "GeneralNote"
                | "OtherAbstract"
                | "SupplMeshList"
                | "CommentsCorrectionsList" => {
                    // TODO
                }
                x => missing_tag_warning(&format!("Not covered in MedlineCitation: '{x}'")),
            }
        }
        ret
    }
}
