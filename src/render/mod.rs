use crate::model::StoryFragment;
use rocket::FromFormField;
use rocket::response::content::RawHtml;
use std::fmt;
use std::string::FromUtf8Error;

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
