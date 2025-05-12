
use crate::stories::Story;
use crate::{Owner, Ownership, Record};
use rmp_serde::encode;
use sled::Tree;
use std::path::Path;
use std::path::PathBuf;
use tokio::io::AsyncReadExt;
use uuid::Uuid;
use walkdir::WalkDir;

/// This module provides utilities for loading site content from ron, md, and pdf documents

/// default database backend
pub struct Sled {
    pub stories: Tree,
}

impl Sled {
    async fn load_stories<P: AsRef<Path>>(
        path: P,
        tree: &mut Tree,
        owner: Ownership,
    ) -> Result<(), sled::Error> {
        println!("opened sled database");
        for file in Self::list_files(path, "md") {
            println!("opening file: {}", file.display());
            let mut content = String::new();
            let mut mdfile = tokio::fs::File::open(&file).await.unwrap();
            mdfile.read_to_string(&mut content).await.unwrap();
            let record = Record {
                name: file.file_name().unwrap().display().to_string(),
                description: None,
                tags: Vec::new(),
                owner,
            };
            let story = Story {
                id: Uuid::new_v4(),
                record,
                text: content,
            };
            tree.insert(story.id, encode::to_vec(&story).unwrap())?;
        }

        Ok(())
    }

    fn list_files<P: AsRef<Path>>(path: P, extension: &str) -> Vec<PathBuf> {
        WalkDir::new(path)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.path().is_file()
                    && entry
                        .path()
                        .extension()
                        .map_or(false, |ext| ext == extension)
            })
            .map(|entry| entry.path().to_path_buf())
            .collect::<Vec<PathBuf>>()
    }

    pub async fn load<P: AsRef<Path>>(
        path: P,
        db_path: P,
        user_id: Uuid,
    ) -> Result<Self, sled::Error> {
        let ownership = Ownership::public(user_id);
        let db: sled::Db = sled::open(db_path)?;
        let mut stories = db.open_tree("stories")?;
        if stories.len() == 0 {
            Self::load_stories(
                path.as_ref().join("writings"),
                &mut stories,
                ownership.clone(),
            )
            .await?;
        }
        Ok(Self {
            projects,
            stories,
            resources,
        })
    }

    pub async fn load_readings<P: AsRef<Path>>(path: P, db_path: P) -> Result<Tree, sled::Error> {
        let db: sled::Db = sled::open(db_path)?;
        let tree = db.open_tree("stories")?;
        for file in Self::list_files(path, "ron") {
            let mut content = String::new();
            let mut ronfile = tokio::fs::File::open(&file).await.unwrap();
            ronfile.read_to_string(&mut content).await.unwrap();
            let resource: Resource = ron::from_str(&content).unwrap();
            tree.insert(resource.id, encode::to_vec(&resource).unwrap())?;
        }
        Ok(tree)
    }
}
