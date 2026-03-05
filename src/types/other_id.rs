use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtherID {
    pub source: Option<String>,
    pub id: Option<String>,
}
