use uuid::Uuid;
use rocket::fs::TempFile;
use rocket::{Route, routes, get, post};

pub struct TimelineForm<'r> {
    pub name: String,
    pub description: Option<String>,
    /// This is intended as a means for defining a SVG superset for representing timelines
    /// accross platforms where each node contains either UUID to reference an entity, or this projects
    /// JSON spec to represent first class entities along a timeline.
    /// however processing this entity is currently not implemented, thus does not show up in the UI.
    pub svg: Option<TempFile<'r>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TimelineRender {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub image: Option<String>,
}



pub fn get_routes() -> Vec<Route> {
    routes![]
}