use rocket::{Route, routes};

use uuid::Uuid;
use wrappedviz::style::{color::Color, shape::NodeShape};

use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncReadExt;
/// for now a structure of having main, desktop, and mobile will work
/// in future versions I will likely want to clean this up to provide default media
/// queries and a way to use said media queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeLoader {
    name: String,
    main: PathBuf,
    desktop: PathBuf,
    mobile: PathBuf,
}

impl ThemeLoader {
    pub fn from_default<P: AsRef<Path>>(name: String, root: P) -> Self {
        let main = root.as_ref().join("/main.css").to_owned();
        let desktop = root.as_ref().join("/desktop.css").to_owned();
        let mobile = root.as_ref().join("/mobile.css").to_owned();
        Self {
            name,
            main,
            desktop,
            mobile,
        }
    }

    pub async fn load(self) -> Result<Theme, std::io::Error> {
        let mut desktop_file = File::open(&self.desktop).await?;
        let mut main_file = File::open(&self.main).await?;
        let mut mobile_file = File::open(&self.mobile).await?;

        let mut mobile_contents = String::new();
        let mut desktop_contents = String::new();
        let main_contents = String::new();

        desktop_file.read_to_string(&mut desktop_contents).await?;
        mobile_file.read_to_string(&mut mobile_contents).await?;
        main_file.read_to_string(&mut mobile_contents).await?;

        Ok(Theme::from_parts(
            self.name,
            main_contents,
            mobile_contents,
            desktop_contents,
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    name: String,
    main: String,
    mobile: String,
    desktop: String,
    graphs: GraphTheme,
}

impl Theme {
    pub fn from_parts(name: String, main: String, mobile: String, desktop: String) -> Self {
        Self {
            name,
            mobile,
            main,
            desktop,
            graphs: GraphTheme::default(),
        }
    }
}

pub fn get_routes() -> Vec<Route> {
    routes![]
}

pub struct EntityTheme {
    shape: NodeShape,
    fill_color: Color,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphTheme {}

impl Default for GraphTheme {
    fn default() -> Self {
        Self {}
    }
}
