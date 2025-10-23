//! This module provides support for parsing Id3v2 "CHAP" frames however lofty currently
//! does not support this functionality and will need to be forked so currently this is not implemented.
//!
use lofty::{id3::v2::Id3v2Tag, tag::Tag};
use std::str;
use tokio::io::AsyncReadExt;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Chapter {
    pub id: Option<Uuid>,          // Optional unique ID for chapter
    pub title: Option<String>,     // Chapter title
    pub start_time_ms: u64,        // Chapter start time in milliseconds
    pub end_time_ms: Option<u64>,  // Optional chapter end time in milliseconds
    pub href: Option<String>,      // Optional link (e.g., chapter URL)
    pub image_url: Option<String>, // Optional chapter-specific image
}
/*
pub fn parse_id3v2_chap_frames(tag: &Tag) -> Vec<Chapter> {
    let mut chapters = Vec::new();

    // Only ID3v2 tags support CHAP frames
    // Try to downcast to ID3v2Tag if possible
    if tag.tag_type() == TagType::ID3v2 {
        // safe to convert to Id3v2Tag
        let id3v2_tag: Id3v2Tag = tag.clone().into();
        for frame in id3v2_tag.frames() {
            if frame.id() == "CHAP" {
                if let Some(chapter) = parse_chap_frame(frame) {
                    chapters.push(chapter);
                }
            }
        }
    }

    chapters
}

fn parse_chap_frame(frame: &ID3v2Frame) -> Option<Chapter> {
    let data = frame.value().as_bytes()?;

    let mut cursor = std::io::Cursor::new(data);

    // Read element ID (null-terminated UTF-8 string)
    let element_id = {
        let mut bytes = Vec::new();
        loop {
            let mut buf = [0u8];
            if cursor.read_exact(&mut buf).is_err() {
                return None;
            }
            if buf[0] == 0 {
                break;
            }
            bytes.push(buf[0]);
        }
        String::from_utf8(bytes).ok()?
    };

    // Helper to read u32 BE
    fn read_u32_be(cursor: &mut std::io::Cursor<&[u8]>) -> Option<u32> {
        let mut buf = [0u8; 4];
        cursor.read_exact(&mut buf).ok()?;
        Some(u32::from_be_bytes(buf))
    }

    let start_time = read_u32_be(&mut cursor)? as u64;
    let end_time = read_u32_be(&mut cursor)? as u64;
    let _start_offset = read_u32_be(&mut cursor)?;
    let _end_offset = read_u32_be(&mut cursor)?;

    // Now parse subframes until end of data or error
    // We look mainly for TIT2 subframe for title
    let mut title: Option<String> = None;

    while (cursor.position() as usize) < data.len() {
        // Frame ID is 4 bytes
        let mut id_bytes = [0u8; 4];
        if cursor.read_exact(&mut id_bytes).is_err() {
            break;
        }
        let subframe_id = String::from_utf8_lossy(&id_bytes).to_string();

        // Frame size (4 bytes BE)
        let size = read_u32_be(&mut cursor)? as usize;

        // Skip 2 bytes flags
        cursor.set_position(cursor.position() + 2);

        // Read frame data
        let pos = cursor.position() as usize;
        if pos + size > data.len() {
            break;
        }
        let frame_data = &data[pos..pos + size];
        cursor.set_position((pos + size) as u64);

        if subframe_id == "TIT2" {
            // Text frame: encoding byte + UTF-8 or UTF-16 string
            if frame_data.len() < 1 {
                continue;
            }
            let encoding = frame_data[0];
            let text_bytes = &frame_data[1..];
            let text = match encoding {
                0 => std::str::from_utf8(text_bytes).ok()?.to_string(),       // ISO-8859-1, treat as UTF-8
                1 => String::from_utf16(
                    &text_bytes
                        .chunks(2)
                        .map(|b| u16::from_be_bytes([b[0], b[1]]))
                        .collect::<Vec<_>>(),
                )
                .ok()?,
                _ => continue,
            };
            title = Some(text);
        }
    }

    Some(Chapter {
        id: None,
        title,
        start_time_ms: start_time,
        end_time_ms: Some(end_time),
        href: None,
        image_url: None,
    })
}*/
