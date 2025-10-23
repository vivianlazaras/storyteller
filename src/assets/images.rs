use crate::errors::ApiError;
use crate::{ApiClient, model::Image};
use crate::{auth::Guard, model::Tag};
use image::{DynamicImage, ImageError, ImageFormat, ImageReader};
use nom_exif::{ExifIter, MediaParser, MediaSource};
use rocket::response::content::RawHtml;
use rocket::{
    FromForm, Route, State,
    form::Form,
    fs::{NamedFile, TempFile},
    get,
    http::ContentType,
    post,
    response::Redirect,
    routes,
    tokio::{
        fs,
        fs::File,
        io::{AsyncReadExt, AsyncWriteExt},
    },
};
use rocket_dyn_templates::{Template, context};
use serde::{Deserialize, Serialize};
use std::{
    io::Cursor,
    path::{Path, PathBuf},
};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExifTagRender {
    tag: String,
    value: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageRender {
    id: Uuid,
    description: Option<String>,
    url: String,
    exif_tags: Vec<ExifTagRender>,
    tags: Vec<String>,
}

#[rocket::async_trait]
pub trait ImageForm<'r>: Sized + Send {
    fn description(&self) -> Option<&str>;
    fn tags(&self) -> &[String];
    fn images(&self) -> Option<&Vec<TempFile<'r>>>;
    fn category(&self) -> &str;
    fn parent(&self) -> Option<Uuid>;
    async fn into_image_builder(
        &self,
        processor: &ImageProcessor,
    ) -> Result<Option<ImageBuilder>, ApiError> {
        let images = match self.images() {
            Some(images) => images,
            None => return Ok(None),
        };
        let data = ImageData {
            images,
            tags: self.tags(),
            description: self.description(),
            category: self.category(),
            parent: self.parent(),
        };
        Ok(processor.process(data).await?)
    }
}

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImageEntry {
    url: String,
    exif_tags: Vec<ExifTag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageBuilder {
    entries: Vec<ImageEntry>,
    description: Option<String>,
    tags: Vec<String>,
    category: String,
    parent: Option<Uuid>,
}

impl ImageBuilder {
    pub fn new(
        entries: Vec<ImageEntry>,
        description: Option<String>,
        tags: Vec<String>,
        category: String,
        parent: Option<Uuid>,
    ) -> Self {
        Self {
            entries,
            description,
            tags,
            category,
            parent,
        }
    }

    pub async fn build(self, api: &ApiClient, access_token: &str) -> Result<Vec<Image>, ApiError> {
        api.post("/assets/images/", access_token, None, &self).await
    }
}

pub struct ImageData<'r> {
    pub images: &'r Vec<TempFile<'r>>,
    pub tags: &'r [String],
    pub description: Option<&'r str>,
    pub category: &'r str,
    pub parent: Option<Uuid>,
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

    pub async fn process<'r>(
        &self,
        image_data: ImageData<'r>,
    ) -> Result<Option<ImageBuilder>, ApiError> {
        let mut entries = Vec::new();
        for image in image_data.images.iter() {
            // if user doesn't select a file the input will exist still, but it won't have a name
            // this checks if the name exists to see if a file was actually selected for upload
            if image.name().is_none() {
                continue;
            }
            let name = Uuid::new_v4().to_string();
            let save_path = self.image_dir.join(&name);

            let mut bytes = Vec::new();
            let mut temp = image.open().await?;
            temp.read_to_end(&mut bytes).await?;

            let mut exif_tags = Vec::new();
            let exif_source = MediaSource::seekable(Cursor::new(&bytes))?;
            if exif_source.has_exif() {
                let mut parser = MediaParser::new();
                if let Ok(tags) = parser.parse(exif_source) {
                    let taglist: ExifIter = tags;
                    for tag in taglist {
                        if let Some(value) = tag.get_value() {
                            exif_tags.push(ExifTag::new(tag.tag_code(), value.to_string()));
                        }
                    }
                }
            }

            let reader = ImageReader::new(Cursor::new(&bytes)).with_guessed_format()?;
            let format = reader.format().unwrap_or(ImageFormat::Png);
            let img = reader.decode()?;

            let mut output = Vec::new();
            img.write_to(&mut Cursor::new(&mut output), format)?;

            let mut file = File::create(&save_path).await?;
            file.write_all(&output).await?;

            let url = format!("/assets/images/{}", name);
            entries.push(ImageEntry { url, exif_tags })
        }
        Ok(Some(ImageBuilder::new(
            entries,
            image_data.description.map(|s| s.to_string()),
            image_data.tags.to_vec(),
            image_data.category.to_string(),
            image_data.parent,
        )))
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
        self.image_dir.join(id.to_string())
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

#[derive(Debug, FromForm)]
pub struct UploadForm<'r> {
    images: Option<Vec<TempFile<'r>>>,
    tags: Option<Vec<String>>,
    description: Option<String>,
    category: String,
    parent: Uuid,
}

impl<'r> ImageForm<'r> for UploadForm<'r> {
    fn images(&self) -> Option<&Vec<TempFile<'r>>> {
        self.images.as_ref()
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn tags(&self) -> &[String] {
        self.tags.as_deref().unwrap_or(&[])
    }

    fn category(&self) -> &str {
        &self.category
    }

    fn parent(&self) -> Option<Uuid> {
        Some(self.parent)
    }
}

#[get("/upload?<parent>&<category>")]
async fn upload_html(parent: Uuid, category: String) -> RawHtml<Template> {
    RawHtml(Template::render(
        "images/upload",
        context! { title: "upload image", parent, category },
    ))
}

#[post("/upload", data = "<form>")]
async fn upload_image<'r>(
    guard: Guard,
    processor: &State<ImageProcessor>,
    form: Form<UploadForm<'r>>,
    api: &State<ApiClient>,
) -> Redirect {
    let form = form.into_inner();
    let parent = form.parent.clone();
    let category = form.category.clone();

    if let Some(builder) = form.into_image_builder(&processor).await.unwrap() {
        builder.build(&api, guard.access_token()).await.unwrap();
    };
    let url = format!("/{}/{}", category, parent);
    Redirect::to(url)
}

#[get("/<id>")]
async fn get_image(id: Uuid, processor: &State<ImageProcessor>) -> Option<NamedFile> {
    let path = processor.get_image_url(id);
    println!("path: {}", path.display());
    NamedFile::open(path).await.ok()
}

#[get("/info/<id>")]
async fn get_info(guard: Guard, id: Uuid, api: &State<ApiClient>) -> RawHtml<Template> {
    let url = format!("/assets/images/{}", id);
    let image: ImageRender = api
        .get_protected(url, guard.access_token(), None)
        .await
        .unwrap();
    RawHtml(Template::render(
        "images/image",
        context!(title: "image info", image ),
    ))
}

#[get("/")]
async fn list_images(guard: Guard, api: &State<ApiClient>) -> RawHtml<Template> {
    let images: Option<Vec<Image>> = api
        .get_protected("/assets/images", guard.access_token(), None)
        .await
        .unwrap();
    RawHtml(Template::render(
        "images/index",
        context!( title: "image list", images ),
    ))
}

pub fn get_routes() -> Vec<Route> {
    routes![get_image, upload_html, upload_image, get_info, list_images]
}
