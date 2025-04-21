use uuid::Uuid;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Community {
    id: Uuid,
    name: String,
    description: Option<String>,
    
}

