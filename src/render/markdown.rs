use crate::render::{Renderer};
use std::string::FromUtf8Error;
pub struct MDRenderer;

impl Renderer for MDRenderer {
    type Err = FromUtf8Error;
    fn process(&self, data: &[u8]) -> Result<RawHtml, Self::Err> {
        let content = String::from_utf8(data)?;
        Ok(RawHtml(markdown_to_html(&content)))
    }
}