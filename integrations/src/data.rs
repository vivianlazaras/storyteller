use serde_derive::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSource {
    name: String,
}