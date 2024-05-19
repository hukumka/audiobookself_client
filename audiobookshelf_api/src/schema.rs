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
    pub id: String,
    pub library_item_id: String,
    pub episode_id: Option<String>,
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
    pub id: String,
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
    pub id: String,
    pub name: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    pub id: String,
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
    pub id: String,
    pub full_path: String,
    pub library_id: String,
    #[serde(deserialize_with = "deserialize_timestamp")]
    pub added_at: DateTime<Utc>,
}
