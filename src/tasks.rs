use rocket::{State, get, post, put, delete, form::Form, FromForm, routes, Route};
use rocket_dyn_templates::{Template, context};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    id: Uuid,
    name: String,
    description: Option<String>,
    created: i64,
    completed: Option<i64>,
    deadline: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromForm)]
pub struct CreateTask {
    name: String,
    description: Option<String>,
    deadline: Option<i64>,
}

#[post("/", data = "<task>")]
async fn create_task(task: Form<CreateTask>) {}

pub fn get_routes() -> Vec<Route> {
    routes![]
}