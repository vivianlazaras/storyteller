use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    id: Uuid,
    name: String,
    description: Option<String>,
}
