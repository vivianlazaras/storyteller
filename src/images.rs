use crate::{ApiClient, model::Image};
use anyhow::Result;
use image::{DynamicImage, ImageError, ImageFormat, io::Reader as ImageReader};
use nom_exif::{ExifIter, MediaParser, MediaSource};
use rocket::{
    Route, State,
    form::{Form, FromForm},
    fs::{NamedFile, TempFile, relative},
    get,
    http::ContentType,
    post, routes,
    tokio::{
        fs,
        fs::File,
        io::{AsyncReadExt, AsyncWriteExt},
    },
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::Cursor,
    path::{Path, PathBuf},
};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExifTag {
    tag: u16,
    value: String,
}

impl ExifTag {
    pub fn new(tag: u16, value: String) -> Self {
        Self { tag, value }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageBuilder {
    url: String,
    description: Option<String>,
    tags: Vec<String>,
    exif_tags: Vec<ExifTag>,
    parent: Uuid,
    category: String,
}

impl ImageBuilder {
    pub fn new(
        url: String,
        description: Option<String>,
        tags: Vec<String>,
        exif_tags: Vec<ExifTag>,
        parent: Uuid,
        category: String,
    ) -> Self {
        Self {
            url,
            description,
            tags,
            exif_tags,
            parent,
            category,
        }
    }

    pub async fn build(self, api: &ApiClient, access_token: &str) -> Result<Image> {
        api.post("/images/", access_token, None, &self).await
    }
}

pub struct ImageData<'r> {
    pub image: TempFile<'r>,
    pub tags: Vec<String>,
    pub description: Option<String>,
    pub parent: Uuid,
    pub category: String,
}

impl<'r> ImageData<'r> {
    pub fn new(
        image: TempFile<'r>,
        tags: Vec<String>,
        description: Option<String>,
        parent: Uuid,
        category: String,
    ) -> Self {
        Self {
            image,
            tags,
            description,
            parent,
            category,
        }
    }
}

pub struct ImageProcessor {
    image_dir: PathBuf,
    hostname: String,
}

impl ImageProcessor {
    pub async fn new(hostname: String, image_dir: PathBuf) -> Self {
        if !image_dir.exists() {
            fs::create_dir_all(&image_dir)
                .await
                .unwrap_or_else(|e| panic!("Failed to create image directory: {}", e));
        }

        if !image_dir.is_dir() {
            panic!(
                "The specified image path is not a directory: {:?}",
                image_dir
            );
        }

        Self {
            image_dir,
            hostname,
        }
    }

    pub async fn process<'r>(&self, mut image_data: ImageData<'r>) -> Result<ImageBuilder> {
        // Generate safe filename
        let name = Uuid::new_v4().to_string();
        let save_path = self.image_dir.join(&name);

        // Read image bytes from TempFile
        let mut bytes = Vec::new();
        let mut temp = image_data.image.open().await?;
        temp.read_to_end(&mut bytes).await?;

        // Extract EXIF data before stripping
        let mut exif_tags = Vec::new();
        let exif_source = MediaSource::seekable(Cursor::new(&bytes))?;
        if exif_source.has_exif() {
            let mut parser = MediaParser::new();
            if let Ok(mut tags) = parser.parse(exif_source) {
                let taglist: ExifIter = tags;
                for tag in taglist {
                    if let Some(value) = tag.get_value() {
                        exif_tags.push(ExifTag::new(tag.tag_code(), value.to_string()));
                    }
                }
            }
        }

        // Decode image and guess format
        let reader = ImageReader::new(Cursor::new(&bytes)).with_guessed_format()?;
        let format = reader.format().unwrap_or(ImageFormat::Png);
        let img = reader.decode()?;

        // Re-encode without EXIF
        let mut output = Vec::new();
        img.write_to(&mut Cursor::new(&mut output), format)?;

        // Save image to disk
        let mut file = File::create(&save_path).await?;
        file.write_all(&output).await?;

        // Generate public URL
        let url = format!("{}/images/{}", self.hostname, name);

        Ok(ImageBuilder::new(
            url,
            image_data.description.clone(),
            image_data.tags.clone(),
            exif_tags,
            image_data.parent,
            image_data.category.clone(),
        ))
    }

    pub async fn get_image(&self, id: Uuid) -> Result<(DynamicImage, ImageFormat), ImageError> {
        let path = self.image_dir.join(id.to_string());
        Self::load_image_from_path(path).await
    }

    async fn load_image_from_path<P: AsRef<Path>>(
        path: P,
    ) -> Result<(DynamicImage, ImageFormat), ImageError> {
        let bytes = fs::read(&path).await.map_err(|e| {
            ImageError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Read error: {e}"),
            ))
        })?;
        let reader = ImageReader::new(Cursor::new(bytes)).with_guessed_format()?;
        let format = reader.format().unwrap_or(ImageFormat::Png);
        let img = reader.decode()?;
        Ok((img, format))
    }
    pub fn get_image_url(&self, id: Uuid) -> PathBuf {
        let relative = self.image_dir.join(id.to_string());
        relative
    }

    fn image_format_to_content_type(format: ImageFormat) -> ContentType {
        match format {
            ImageFormat::Png => ContentType::PNG,
            ImageFormat::Jpeg => ContentType::JPEG,
            ImageFormat::Gif => ContentType::GIF,
            ImageFormat::Bmp => ContentType::new("image", "bmp"),
            ImageFormat::Ico => ContentType::new("image", "x-icon"),
            ImageFormat::Tiff => ContentType::new("image", "tiff"),
            ImageFormat::WebP => ContentType::new("image", "webp"),
            ImageFormat::Avif => ContentType::new("image", "avif"),
            ImageFormat::Pnm => ContentType::new("image", "x-portable-anymap"),
            ImageFormat::Tga => ContentType::new("image", "x-tga"),
            ImageFormat::Dds => ContentType::new("image", "vnd.ms-dds"),
            ImageFormat::Farbfeld => ContentType::new("image", "farbfeld"),
            ImageFormat::Qoi => ContentType::new("image", "qoi"),
            _ => ContentType::Binary,
        }
    }
}

#[get("/<id>")]
async fn get_image(id: Uuid, processor: &State<ImageProcessor>) -> Option<NamedFile> {
    let path = processor.get_image_url(id);
    NamedFile::open(path).await.ok()
}

pub fn get_routes() -> Vec<Route> {
    routes![get_image]
}
