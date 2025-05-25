use rocket_dyn_templates::{Template};
use rocket::response::content::RawHtml;
use crate::model::StoryFragment;
use std::string::FromUtf8Error;
use rocket::FromFormField;
use std::fmt;

pub trait Renderer {
    fn process(&self, content: &[u8]) -> Result<RawHtml<String>, RenderErr>;
}

pub trait Renderable {
    fn render<R: Renderer>(&self, renderer: &R) -> Result<RawHtml<String>, RenderErr>;
}

impl Renderable for String {
    fn render<R: Renderer>(&self, renderer: &R) -> Result<RawHtml<String>, RenderErr> {
        Ok(renderer.process(&self.as_bytes())?)
    }
}

#[derive(Debug, Error)]
#[error(display = "failed to render: {}", _0)]
pub enum RenderErr {
    #[error(display = "utf8 error: {}", _0)]
    FromUtf8Error(#[error(source)] FromUtf8Error),
}

impl Renderable for StoryFragment {
    fn render<R: Renderer>(&self, renderer: &R) -> Result<RawHtml<String>, RenderErr> {
        Ok(renderer.process(&self.content)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Copy, FromFormField)]
pub enum SupportedRender {
    Markdown,
    Text,
    HTML,
}

impl fmt::Display for SupportedRender {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let strval = match self {
            Self::Markdown => "Markdown",
            Self::Text => "Text",
            Self::HTML => "HTML",
        };
        write!(f, "{}", strval)
    }
}