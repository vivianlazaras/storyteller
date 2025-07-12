use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Users {
    id: Uuid,
    fname: String,
    lname: String,
    subject: Uuid,
    email: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Group {
    id: Uuid,
    name: Option<String>,
    description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct License {
    id: Uuid,
    name: String,
    description: Option<String>,
    public: bool,
    content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Timeline {
    pub id: Uuid,
    pub created: i64,
    pub last_edited: i64,
    pub license: Uuid,
    pub creator: Uuid,
    pub shared: Option<Uuid>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Story {
    pub id: Uuid,
    pub created: i64,
    pub last_edited: i64,
    pub name: String,
    pub description: Option<String>,
    pub renderer: Option<String>,
    pub image: Option<String>,
}

/// fragments represent chapters within a larger story
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoryFragment {
    pub id: Uuid,
    pub name: String,
    pub metadata: Uuid,
    pub idx: i32,
    pub content: String,
    pub image: Option<String>,
    pub last_edited: i64,
    pub created: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Character {
    pub id: Uuid,
    pub created: i64,
    pub last_edited: i64,
    pub shared: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Relationship {
    id: Uuid,
    description: Option<String>,
    character: Uuid,
    story: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    id: Uuid,
    entity: Uuid,
    value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    id: Uuid,
    name: String,
    description: Option<String>,
    created: Option<i64>,
    completed: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    pub id: Uuid,
    pub url: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created: i64,
}
