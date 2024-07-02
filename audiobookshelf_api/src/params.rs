use crate::schema::{Author, Id, Progress, Series};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use serde::Serialize;

#[derive(Default, Debug, Clone)]
pub struct LibraryItemParams {
    pub limit: usize,
    pub page: usize,
    pub sort: Option<String>,
    pub desc: bool,
    pub filter: LibraryItemFilter,
}

#[derive(Default, Debug, Clone)]
pub struct LibraryItemFilter {
    pub authors: Vec<Id<Author>>,
    pub series: Vec<Id<Series>>,
    pub tags: Vec<String>,
    pub genres: Vec<String>,
    pub progress: Option<Progress>,
}

impl LibraryItemParams {
    pub fn build_query(self) -> Vec<(&'static str, String)> {
        let mut result = vec![];
        if self.limit != 0 {
            result.push(("limit", self.limit.to_string()));
            result.push(("page", self.page.to_string()));
        }
        if let Some(sort) = self.sort {
            result.push(("sort", sort));
        }
        result.push(("desc", self.desc.to_string()));

        for author in &self.filter.authors {
            Self::add_filter(&mut result, "authors", author.as_str());
        }
        for series in &self.filter.series {
            Self::add_filter(&mut result, "series", series.as_str());
        }
        for tag in &self.filter.tags {
            Self::add_filter(&mut result, "tags", tag.as_str());
        }
        for genre in &self.filter.genres {
            Self::add_filter(&mut result, "genres", genre.as_str());
        }
        if let Some(progress) = self.filter.progress {
            Self::add_filter(&mut result, "progress", progress.as_str());
        }
        result
    }

    fn add_filter(query: &mut Vec<(&'static str, String)>, filter: &str, value: &str) {
        query.push((
            "filter",
            format!("{filter}.{b64value}", b64value = STANDARD.encode(value)),
        ));
    }
}

#[derive(Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct PlayLibraryItemParams {
    pub device_info: DeviceInfoParams,
    pub force_direct_play: bool,
    pub force_transcode: bool,
    pub supported_mime_types: Vec<String>,
    pub media_player: Option<String>,
}

#[derive(Serialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct DeviceInfoParams {
    pub device_id: Option<String>,
    pub client_name: Option<String>,
    pub client_version: Option<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub sdk_version: Option<u64>,
}
