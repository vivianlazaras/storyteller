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
use super::frontend::*;
use crate::{
    ApiClient,
    assets::images::{ImageBuilder, ImageData, ImageProcessor},
    model::{Character, Image},
};
use anyhow::Result;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use petgraph::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

impl Character {
    /// # Note
    /// This is an expensive function, the api server has to recursively build the tree
    /// then the rust code has to traverse the tree and create appropriate links
    pub async fn family_tree(
        id: Uuid,
        api: &ApiClient,
        access_token: &str,
    ) -> Result<(DiGraph<String, &'static str>, HashMap<Uuid, NodeIndex>)> {
        let mut index_map = HashMap::new();
        let mut seen = HashSet::new();
        let mut graph = DiGraph::new();
        let mut params = HashMap::new();
        let id_str = id.to_string();
        params.insert("id", id_str.as_str());
        let root: CharacterNode = api
            .get_protected("/characters/tree", access_token, Some(params))
            .await?;
        build_family_tree(&root, &mut graph, &mut index_map, &mut seen);
        Ok((graph, index_map))
    }

    /// Converts a `Character` into a `CharacterRender` with the given image and tags.
    ///
    /// # Arguments
    /// * `image` - Optional image URL.
    /// * `tags` - A list of associated tags.
    ///
    /// # Returns
    /// A `CharacterRender` struct containing character data.
    pub fn render(self, thumbnail: Option<Image>, tags: Vec<String>) -> CharacterRender {
        CharacterRender {
            thumbnail,
            images: Some(Vec::new()),
            tags: Some(tags),
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
    pub thumbnail: Option<ImageBuilder>,
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
    pub fn new(
        name: &str,
        description: Option<&str>,
        tags: Option<Vec<String>>,
        thumbnail: Option<ImageBuilder>,
    ) -> Self {
        Self {
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            tags,
            thumbnail,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterNode {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub children: Vec<CharacterNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterRender {
    pub thumbnail: Option<Image>,
    pub tags: Option<Vec<String>>,
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub images: Option<Vec<Image>>,
}

fn build_family_tree(
    root: &CharacterNode,
    graph: &mut DiGraph<String, &'static str>,
    index_map: &mut HashMap<Uuid, NodeIndex>,
    seen: &mut HashSet<Uuid>,
) -> NodeIndex {
    if let Some(&idx) = index_map.get(&root.id) {
        return idx;
    }

    // Add current node
    let node_index = graph.add_node(root.name.clone());
    index_map.insert(root.id, node_index);
    seen.insert(root.id);

    for child in &root.children {
        let child_index = build_family_tree(child, graph, index_map, seen);

        // Prevent duplicate edges or cycles
        if !graph.edges(node_index).any(|e| e.target() == child_index) {
            graph.add_edge(node_index, child_index, "parent_of");
        }
    }

    node_index
}
