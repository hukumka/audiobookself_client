use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};

fn deserialize_timestamp<'d, D: Deserializer<'d>>(
    deserializer: D,
) -> Result<DateTime<Utc>, D::Error> {
    let timestamp = i64::deserialize(deserializer)?;
    DateTime::from_timestamp_millis(timestamp)
        .ok_or(serde::de::Error::custom("DateTime out of range"))
}

fn deserialize_timestamp_option<'d, D: Deserializer<'d>>(
    deserializer: D,
) -> Result<Option<DateTime<Utc>>, D::Error> {
    let timestamp = Option::<i64>::deserialize(deserializer)?;
    if let Some(timestamp) = timestamp {
        let datetime = DateTime::from_timestamp_millis(timestamp)
            .ok_or(serde::de::Error::custom("DateTime out of range"))?;
        Ok(Some(datetime))
    } else {
        Ok(None)
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}

/// Response to `AuthRequest`
#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    pub user: UserData,
    pub user_default_library_id: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UserData {
    pub id: String,
    pub username: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub token: String,
    pub media_progress: Vec<MediaProgress>,
    pub permissions: UserPermissions,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UserPermissions {
    pub download: bool,
    pub update: bool,
    pub delete: bool,
    pub upload: bool,
    pub access_all_libraries: bool,
    pub access_all_tags: bool,
    pub access_explicit_content: bool,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MediaProgress {
    pub id: Id<MediaProgress>,
    pub library_item_id: Id<LibraryItem>,
    pub episode_id: Option<Id<Episode>>,
    pub duration: f64,
    pub progress: f64,
    pub current_time: f64,
    pub is_finished: bool,
    pub hide_from_continue_listening: bool,
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub last_update: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub started_at: DateTime<Utc>,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_timestamp_option")]
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(transparent)]
pub struct Id<T> {
    pub id: String,
    #[serde(skip)]
    pub marker: std::marker::PhantomData<T>,
}

impl<T> Id<T> {
    pub fn as_str(&self) -> &str {
        self.id.as_str()
    }
}

/// Response to `GET /api/libraries`
#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Libraries {
    pub libraries: Vec<Library>,
}

/// Response to `GET /api/libraries/<ID>`
#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Library {
    pub id: Id<Library>,
    pub name: String,
    pub folders: Vec<Folder>,
    pub display_order: u64,
    pub icon: String,
    pub media_type: MediaType,
    pub provider: String,
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub created_at: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub last_update: DateTime<Utc>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LibraryWithFilters {
    pub library: Library,
    pub filterdata: LibraryFilterData,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LibraryFilterData {
    pub authors: Vec<Author>,
    pub genres: Vec<String>,
    pub series: Vec<Series>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Series {
    pub id: Id<Series>,
    pub name: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    pub id: Id<Author>,
    pub name: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum MediaType {
    Book,
    Podcast,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Folder {
    pub id: Id<Folder>,
    pub full_path: String,
    pub library_id: String,
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub added_at: DateTime<Utc>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedResponse<T> {
    pub results: Vec<T>,
    pub total: usize,
    pub limit: usize,
    pub page: usize,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LibraryItem {
    pub id: Id<LibraryItem>,
    pub library_id: Id<Library>,
    pub folder_id: Id<Folder>,
    pub path: String,
    pub rel_path: String,
    pub is_file: bool,
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub mtime_ms: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub ctime_ms: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub birthtime_ms: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub added_at: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub updated_at: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_timestamp_option")]
    pub last_scan: Option<DateTime<Utc>>,
    pub scan_version: Option<String>,
    pub is_missing: bool,
    pub is_invalid: bool,
    #[serde(flatten)]
    pub media: LibraryMedia,
    pub library_files: Vec<LibraryFile>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LibraryItemMinified {
    pub id: Id<LibraryItem>,
    pub library_id: Id<Library>,
    pub folder_id: Id<Folder>,
    pub path: String,
    pub rel_path: String,
    pub is_file: bool,
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub mtime_ms: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub ctime_ms: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub birthtime_ms: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub added_at: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub updated_at: DateTime<Utc>,
    pub is_missing: bool,
    pub is_invalid: bool,
    #[serde(flatten)]
    pub media: LibraryMediaMinified,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "mediaType", content = "media")]
#[serde(rename_all = "camelCase")]
pub enum LibraryMedia {
    #[serde(rename_all = "camelCase")]
    Book {
        library_item_id: Id<LibraryItem>,
        metadata: BookMetadata,
        cover_path: Option<String>,
        tags: Vec<String>,
        audio_files: Vec<AudioFile>,
        chapters: Vec<Chapter>,
    },
    #[serde(rename_all = "camelCase")]
    Podcast {
        library_item_id: Id<LibraryItem>,
        metadata: PodcastMetadata,
        cover_path: Option<String>,
        tags: Vec<String>,
        episodes: Vec<PodcastEpisode>,
        auto_download_episodes: bool,
        auto_download_schedule: String,
        last_episode_check: bool,
        max_episodes_to_keep: usize,
        max_new_episodes_to_download: usize,
    },
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "mediaType", content = "media")]
#[serde(rename_all = "camelCase")]
pub enum LibraryMediaMinified {
    Book {
        metadata: BookMetadataMinified,
        cover_path: Option<String>,
        tags: Vec<String>,
    },
    Podcast {
        metadata: PodcastMetadataMinified,
        cover_path: Option<String>,
        tags: Vec<String>,
        auto_download_episodes: bool,
        auto_download_schedule: String,
        last_episode_check: bool,
        max_episodes_to_keep: usize,
        max_new_episodes_to_download: usize,
    },
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PodcastEpisode {
    pub library_item_id: Id<LibraryItem>,
    pub id: Id<PodcastEpisode>,
    pub index: usize,
    pub season: String,
    pub episode: String,
    pub episode_type: String,
    pub title: String,
    pub subtitle: String,
    pub description: String,
    pub pub_date: String,
    pub audio_file: AudioFile,
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub published_at: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub added_at: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PodcastMetadata {
    pub title: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub release_date: Option<String>,
    pub genres: Vec<String>,
    pub feed_url: Option<String>,
    pub image_url: Option<String>,
    pub itunes_page_url: Option<String>,
    pub itunes_id: Option<i64>,
    pub itunes_artist_id: Option<i64>,
    pub explicit: bool,
    pub language: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<String>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PodcastMetadataMinified {
    pub title_ignore_prefix: String,
    pub title: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub release_date: Option<String>,
    pub genres: Vec<String>,
    pub feed_url: Option<String>,
    pub image_url: Option<String>,
    pub itunes_page_url: Option<String>,
    pub itunes_id: Option<i64>,
    pub itunes_artist_id: Option<i64>,
    pub explicit: bool,
    pub language: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<String>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AudioFile {
    pub index: usize,
    pub ino: String,
    pub metadata: FileMetadata,
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub added_at: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub updated_at: DateTime<Utc>,
    pub track_num_from_meta: Option<u32>,
    pub disc_num_from_meta: Option<u32>,
    pub track_num_from_filename: Option<u32>,
    pub disc_num_from_filename: Option<u32>,
    pub manually_verified: bool,
    pub exclude: bool,
    pub error: Option<String>,
    pub format: String,
    pub duration: f64,
    pub bit_rate: u32,
    pub language: Option<String>,
    pub codec: String,
    pub time_base: String,
    pub channels: u32,
    pub channel_layout: String,
    pub chapters: Vec<Chapter>,
    pub embedded_cover_art: Option<String>,
    pub mime_type: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Chapter {
    pub id: usize,
    pub start: f64,
    pub end: f64,
    pub title: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BookMetadata {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub authors: Vec<Author>,
    pub narrators: Vec<String>,
    pub series: Vec<Series>,
    pub genres: Vec<String>,
    pub published_year: Option<String>,
    pub published_data: Option<String>,
    pub publisher: Option<String>,
    pub description: Option<String>,
    pub isbn: Option<String>,
    pub asin: Option<String>,
    pub language: Option<String>,
    pub explicit: bool,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BookMetadataMinified {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub genres: Vec<String>,
    pub title_ignore_prefix: String,
    pub author_name: String,
    #[serde(rename = "authorNameLF")]
    pub author_name_lf: String,
    pub narrator_name: String,
    pub series_name: String,
    pub published_year: Option<String>,
    pub published_data: Option<String>,
    pub publisher: Option<String>,
    pub description: Option<String>,
    pub isbn: Option<String>,
    pub asin: Option<String>,
    pub language: Option<String>,
    pub explicit: bool,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LibraryFile {
    pub ino: String,
    pub metadata: FileMetadata,
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub added_at: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub updated_at: DateTime<Utc>,
    pub file_type: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FileMetadata {
    pub filename: String,
    pub ext: String,
    pub path: String,
    pub rel_path: String,
    pub size: usize,
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub mtime_ms: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub ctime_ms: DateTime<Utc>,
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub birthtime_ms: DateTime<Utc>,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Episode {}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Progress {
    Finished,
    NotStarted,
    NotFinished,
    InProgress,
}

impl Progress {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Finished => "finished",
            Self::NotStarted => "not-started",
            Self::NotFinished => "not-finished",
            Self::InProgress => "in-progress",
        }
    }
}
