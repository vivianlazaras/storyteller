//! Character creation module.
//!
//! This module contains logic for constructing characters from form data, processing image uploads,
//! and interacting with a backend API for character persistence.
//!
//! ```rust
//! let api = ApiClient::new("http://localhost:8442").await.unwrap();
//! let character = CharacterBuilder::new(
//!         "character", 
//!         Some("a test character"), 
//!         None
//!     )
//!     .build(&api, "my_jwk_access_token")
//!     .await
//!     .unwrap();
//! ```
use crate::{
    ApiClient,
    assets::images::{ImageBuilder, ImageData, ImageProcessor},
    model::Character,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use super::frontend::*;

impl Character {
    /// Create an image builder using the given form data and image processor.
    /// 
    /// # Arguments
    /// * `form` - The form data containing image and optional tags.
    /// * `processor` - The image processor to handle image processing.
    /// 
    /// # Returns
    /// A `Result` with an optional `ImageBuilder` if an image was provided.
    pub async fn build_image<'r>(
        &self,
        form: CharacterBuilderForm<'r>,
        processor: &ImageProcessor,
    ) -> Result<Option<ImageBuilder>> {
        if let Some(image) = form.image {
            let imagedata = ImageData::new(
                image,
                form.imagetags.clone().unwrap_or_default(),
                None,
                self.id,
                "characters".into(),
            );
            Ok(Some(processor.process(imagedata).await?))
        } else {
            Ok(None)
        }
    }

    /// Converts a `Character` into a `CharacterRender` with the given image and tags.
    /// 
    /// # Arguments
    /// * `image` - Optional image URL.
    /// * `tags` - A list of associated tags.
    /// 
    /// # Returns
    /// A `CharacterRender` struct containing character data.
    pub fn render(self, image: Option<String>, tags: Vec<String>) -> CharacterRender {
        CharacterRender {
            image,
            tags,
            id: self.id,
            name: self.name,
            description: self.description,
        }
    }
}

/// A builder struct for creating new `Character` instances.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterBuilder {
    /// The name of the character.
    pub name: String,
    /// Optional description of the character.
    pub description: Option<String>,
    /// Optional list of tags associated with the character.
    pub tags: Option<Vec<String>>,
}

impl CharacterBuilder {
    /// Sends a POST request to the API to create a new character.
    /// 
    /// # Arguments
    /// * `api` - The API client instance.
    /// * `access_token` - The access token for authentication.
    /// 
    /// # Returns
    /// A `Result` containing the newly created `Character
    pub async fn build(&self, api: &ApiClient, access_token: &str) -> Result<Character> {
        api.post("/characters/", access_token, None, &self).await
    }

    /// Create A CharacterBuilder for creating a character on the server backend.
    ///
    /// # Arguments
    /// * `name` - The name of the new character.
    /// * `description` - An optional description of the character
    /// * `tags` - An optional collection of tags to make finding the character, and analysis easier.
    pub fn new(name: &str, description: Option<&str>, tags: Option<Vec<String>>) -> Self {
        Self {
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            tags
        }
    }
}