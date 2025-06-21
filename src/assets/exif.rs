use std::collections::HashMap;

pub enum TagCategory {
    GPS,
    DateTime,
}

pub struct ExifTag {
    human_readable: bool,
    tagName: Option<String>,
}

pub struct ExifProcessor {
    tagmap: HashMap<i32, String>,
}
