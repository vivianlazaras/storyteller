use uuid::Uuid;

#[derive(  Serialize, Deserialize, Debug, Clone)]
pub struct Users {
    id: Uuid,
    fname: String,
    lname: String,
    subject: Uuid,
    email: String,
}


#[derive(  Serialize, Deserialize, Debug, Clone)]
pub struct Group {
    id: Uuid,
    name: Option<String>,
    description: Option<String>,
}


#[derive(  Serialize, Deserialize, Debug, Clone)]
pub struct License {
    id: Uuid,
    name: String,
    description: Option<String>,
    public: bool,
    content: String,
}

#[derive(  Serialize, Deserialize, Debug, Clone)]
pub struct Timeline {
    id: Uuid,
    created: i64,
    last_edited: i64,
    license: Uuid,
    creator: Uuid,
    shared: Option<Uuid>,
}

#[derive(  Serialize, Deserialize, Debug, Clone)]
pub struct StoryFragment {
    pub created: i64,
    pub last_edited: i64,
    pub license: Uuid,
    pub creator: Uuid,
    pub shared: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub timeline: Uuid,
    pub renderer: Option<String>,
    pub content: Vec<u8>,
}

#[derive(  Serialize, Deserialize, Debug, Clone)]
pub struct Character {
    id: Uuid,
    created: i64,
    last_edited: i64,
    license: Uuid,
    creator: Uuid,
    shared: Option<Uuid>,
    timeline: Uuid,
    name: String,
    description: Option<String>,
}

#[derive(  Serialize, Deserialize, Debug, Clone)]
pub struct Relationship {
    id: Uuid,
    description: Option<String>,
    character: Uuid,
    story: Uuid,
}