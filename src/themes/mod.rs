use rocket::{routes, Route};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    name: String,
    desktop: PathBuf,
    mobile: PathBuf,
    
}

pub fn get_routes() -> Vec<Route> {

}