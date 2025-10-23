use uuid::Uuid;

/// please note this is a super stripped down minimum metadata set
/// in future more metadata fields will be added to this type
/// the goal of datasets is to provide pattern structures for fragments
/// as well as to provide inspiration for a more holistic, detailed view of a story
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSetInfo {
    id: Uuid,
    name: String,
    description: Option<String>,
    disaggregated: bool,
    human_subject: bool,
}
