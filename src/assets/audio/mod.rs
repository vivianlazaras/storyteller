pub mod chapters;
use anyhow::Result;
use chapters::Chapter;
use chrono::{DateTime, TimeZone, Utc};
use lofty::error::{ErrorKind, LoftyError};
use lofty::file::AudioFile;
use lofty::file::TaggedFileExt;
use lofty::prelude::*;
use lofty::tag::{Accessor, Tag};
use rocket::fs::TempFile;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs;
use url::Url;
use uuid::Uuid;

pub struct AudioData<'a> {
    recordings: Option<Vec<TempFile<'a>>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AudioMetadata {
    Audiobook(AudioBookBuilder),
    Podcast(PodcastBuilder),
    Song(SongBuilder),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioBuilder {
    metadata: AudioMetadata,
    url: Url,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AudioBookBuilder {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub author: Option<String>,
    pub narrator: Option<String>,
    pub publisher: Option<String>,
    pub isbn: Option<String>,
    pub language: Option<String>,
    pub genre: Option<String>,
    pub release_date: Option<DateTime<Utc>>,
    pub cover_art_url: Option<String>,
    pub chapters: Vec<Chapter>,
    pub duration_ms: Option<u64>, // Total length in milliseconds
    pub description: Option<String>,
    pub series: Option<String>,     // Series title, if part of one
    pub series_number: Option<u32>, // Position in series
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PodcastBuilder {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub publication_date: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
    pub audio_url: Option<String>,
    pub episode_number: Option<u32>,
    pub season_number: Option<u32>,
    pub explicit: Option<bool>,
    pub cover_art_url: Option<String>,
    pub chapters: Vec<Chapter>,
    pub transcript_url: Option<String>,
    pub transcript_text: Option<String>,
    pub sponsor_info: Option<String>, // Sponsor or ad information
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SongBuilder {
    /// Track title
    pub title: Option<String>,
    /// Performer
    pub artist: Option<String>,
    /// Album name
    pub album: Option<String>,
    /// Music genre
    pub genre: Option<String>,
    /// Release year
    pub year: Option<u32>,
    /// Track number
    pub track_number: Option<u32>,
    /// Disc number
    pub disc_number: Option<u32>,
    /// Album-level artist
    pub album_artist: Option<String>,
    /// Freeform comment
    pub comment: Option<String>,
    /// Composer
    pub composer: Option<String>,
    /// Lyrics
    pub lyrics: Option<String>,
}

pub struct AudioProcessor {
    audio_dir: PathBuf,
    hostname: String,
}

impl AudioProcessor {
    pub async fn new(hostname: String, audio_dir: PathBuf) -> Self {
        if !audio_dir.exists() {
            fs::create_dir_all(&audio_dir)
                .await
                .unwrap_or_else(|e| panic!("Failed to create image directory: {}", e));
        }
        if !audio_dir.is_dir() {
            panic!(
                "The specified image path is not a directory: {:?}",
                audio_dir
            );
        }
        Self {
            audio_dir,
            hostname,
        }
    }

    pub async fn process<'a>(&self, audio_data: AudioData<'a>) -> Result<Option<AudioBuilder>> {
        unimplemented!();
    }
}

// --- Traits for metadata extraction per content type ---
pub trait AudioBookMeta {
    fn parse_audio_book(&self, tag: &Tag) -> AudioBookBuilder;
}

pub trait PodcastMeta {
    fn parse_podcast(&self, tag: &Tag) -> PodcastBuilder;
}

pub trait SongMeta {
    fn parse_song(&self, tag: &Tag) -> SongBuilder;
}

// --- Format-specific implementations ---

pub struct Mp3Meta;
pub struct M4aMeta;

impl AudioBookMeta for Mp3Meta {
    fn parse_audio_book(&self, tag: &Tag) -> AudioBookBuilder {
        AudioBookBuilder {
            title: tag.title().map(|s| s.to_string()),
            author: tag.artist().map(|s| s.to_string()),
            narrator: tag.get_string(&ItemKey::TrackArtist).map(|s| s.to_string()), // example heuristic
            publisher: tag.get_string(&ItemKey::Publisher).map(|s| s.to_string()),
            isbn: tag.get_string(&ItemKey::Isrc).map(|s| s.to_string()),
            chapters: Vec::new(),
            ..Default::default()
        }
    }
}

impl PodcastMeta for Mp3Meta {
    fn parse_podcast(&self, tag: &Tag) -> PodcastBuilder {
        PodcastBuilder {
            title: tag.title().map(|s| s.to_string()),
            episode_number: tag
                .get_string(&ItemKey::Unknown("TXXX:ITUNESEPISODE".into()))
                .and_then(|s| s.parse().ok()),
            season_number: tag
                .get_string(&ItemKey::Unknown("TXXX:ITUNESEASON".into()))
                .and_then(|s| s.parse().ok()),
            explicit: tag
                .get_string(&ItemKey::Unknown("TXXX:ITUNESADVISORY".into()))
                .map(|s| s.to_lowercase().contains("explicit")),
            chapters: parse_chapters(tag),
            ..Default::default()
        }
    }
}

impl SongMeta for Mp3Meta {
    fn parse_song(&self, tag: &Tag) -> SongBuilder {
        SongBuilder {
            title: tag.title().map(|s| s.to_string()),
            artist: tag.artist().map(|s| s.to_string()),
            album: tag.album().map(|s| s.to_string()),
            genre: tag.genre().map(|s| s.to_string()),
            track_number: tag.track().map(|n| n as u32),
            disc_number: tag.disk().map(|n| n as u32),
            album_artist: tag.get_string(&ItemKey::AlbumArtist).map(|s| s.to_string()),
            comment: tag.get_string(&ItemKey::Comment).map(|s| s.to_string()),
            composer: tag.get_string(&ItemKey::Composer).map(|s| s.to_string()),
            lyrics: tag.get_string(&ItemKey::Lyrics).map(|s| s.to_string()),
            ..Default::default()
        }
    }
}

impl AudioBookMeta for M4aMeta {
    fn parse_audio_book(&self, tag: &Tag) -> AudioBookBuilder {
        AudioBookBuilder {
            title: tag.title().map(|s| s.to_string()),
            author: tag
                .get_string(&ItemKey::Unknown("©ART".into()))
                .map(|s| s.to_string()),
            publisher: tag
                .get_string(&ItemKey::Unknown("©pub".into()))
                .map(|s| s.to_string()),
            chapters: parse_chapters(tag),
            ..Default::default()
        }
    }
}

impl PodcastMeta for M4aMeta {
    fn parse_podcast(&self, tag: &Tag) -> PodcastBuilder {
        PodcastBuilder {
            title: tag.title().map(|s| s.to_string()),
            episode_number: tag
                .get_string(&ItemKey::Unknown("tvnn".into()))
                .and_then(|s| s.parse().ok()),
            explicit: tag
                .get_string(&ItemKey::Unknown("rtng".into()))
                .map(|s| s.to_lowercase().contains("explicit")),
            chapters: parse_chapters(tag),
            ..Default::default()
        }
    }
}

impl SongMeta for M4aMeta {
    fn parse_song(&self, tag: &Tag) -> SongBuilder {
        SongBuilder {
            title: tag.title().map(|s| s.to_string()),
            artist: tag
                .get_string(&ItemKey::Unknown("©ART".into()))
                .map(|s| s.to_string()),
            album: tag.album().map(|s| s.to_string()),
            genre: tag.genre().map(|s| s.to_string()),
            ..Default::default()
        }
    }
}

// Helper to parse chapters generically
fn parse_chapters(tag: &Tag) -> Vec<Chapter> {
    /*tag.chapters()
    .iter()
    .map(|c| Chapter {
        id: None,
        title: c.title().map(|t| t.to_string()),
        start_time_ms: c.start_time().as_millis() as u64,
        end_time_ms: c.end_time().map(|et| et.as_millis() as u64),
        href: None,
        image_url: None,
    })
    .collect()*/
    Vec::new()
}

// --- Main dispatch based on file extension ---

pub fn parse_audio_file<P: AsRef<Path>>(path: P) -> Result<AudioMetadata, LoftyError> {
    let path = path.as_ref();

    let tagged_file = lofty::read_from_path(path)?;

    let tag = tagged_file
        .primary_tag()
        .ok_or_else(|| LoftyError::new(ErrorKind::UnknownFormat))?;

    // Detect file format by extension
    let format = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();

    match format.as_str() {
        "mp3" => {
            let parser = Mp3Meta;
            // Use heuristics or priority to pick type:
            // For example: if narrator/isbn present -> audiobook,
            // else if episode number present -> podcast,
            // else song

            if tag.get_string(&ItemKey::Isrc).is_some()
                || tag.get_string(&ItemKey::Publisher).is_some()
            {
                Ok(AudioMetadata::Audiobook(parser.parse_audio_book(&tag)))
            } else if tag
                .get_string(&ItemKey::Unknown("TXXX:ITUNESEPISODE".into()))
                .is_some()
            {
                Ok(AudioMetadata::Podcast(parser.parse_podcast(&tag)))
            } else {
                Ok(AudioMetadata::Song(parser.parse_song(&tag)))
            }
        }
        "m4a" => {
            let parser = M4aMeta;

            if tag.get_string(&ItemKey::Unknown("©isb".into())).is_some()
                || tag.get_string(&ItemKey::Unknown("©pub".into())).is_some()
            {
                Ok(AudioMetadata::Audiobook(parser.parse_audio_book(&tag)))
            } else if tag.get_string(&ItemKey::Unknown("tvnn".into())).is_some() {
                Ok(AudioMetadata::Podcast(parser.parse_podcast(&tag)))
            } else {
                Ok(AudioMetadata::Song(parser.parse_song(&tag)))
            }
        }
        _ => {
            // Default fallback: treat as Song
            let parser = Mp3Meta; // or create a default parser
            Ok(AudioMetadata::Song(parser.parse_song(&tag)))
        }
    }
}
