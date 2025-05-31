/// this module provides functions for handling image upload

use rocket::fs::TempFile;
use rocket::form::Form;
use rocket::http::ContentType;
use rocket::response::Redirect;
use rocket_multipart_form_data::{
    MultipartFormDataOptions, MultipartFormData, MultipartFormDataField, RawField,
    mime,
};

use std::path::Path;

#[post("/upload", data = "<data>")]
async fn upload(content_type: &ContentType, data: rocket::Data<'_>) -> &'static str {
    let mut options = MultipartFormDataOptions::new();
    options.allowed_fields.push(
        MultipartFormDataField::raw("image")
            .content_type_by_string(Some(mime::IMAGE_STAR))
            .max_length(5 * 1024 * 1024), // Limit to 5 MB
    );

    let multipart_form_data = match MultipartFormData::parse(content_type, data, options).await {
        Ok(mfd) => mfd,
        Err(_) => return "Failed to parse multipart form data",
    };

    if let Some(RawField::Single(raw)) = multipart_form_data.raw.get("image") {
        let filename = raw.file_name.as_deref().unwrap_or("upload.jpg");
        let path = Path::new("uploads").join(filename);

        // Save the file to disk
        if let Err(e) = tokio::fs::write(&path, &raw.raw).await {
            eprintln!("File write error: {}", e);
            return "Failed to save image";
        }

        return "Image uploaded successfully!";
    }

    "No image uploaded"
}