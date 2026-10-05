use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DownloadStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "parsing")]
    Parsing,
    #[serde(rename = "downloading")]
    Downloading,
    #[serde(rename = "paused")]
    Paused,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "cancelled")]
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub percentage: f32,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub speed_bytes_per_sec: u64,
    pub eta_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadTask {
    pub id: String,
    pub url: String,
    pub title: String,
    pub up_name: String,
    pub cover_url: String,
    pub duration_text: String,
    pub quality_id: u32,
    pub quality_name: String,
    pub status: DownloadStatus,
    pub progress: Option<DownloadProgress>,
    pub output_path: String,
    pub error_message: Option<String>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub created_at: String,
    pub updated_at: String,
    pub download_description: bool,
    pub download_danmaku: bool,
    pub download_subtitle: bool,
    pub download_cover: bool,
    pub audio_only: bool,
}

impl DownloadTask {
    pub fn new(url: &str, quality_id: u32, title: &str, up_name: &str, cover_url: &str, duration_text: &str, quality_name: &str) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            url: url.to_string(),
            title: title.to_string(),
            up_name: up_name.to_string(),
            cover_url: cover_url.to_string(),
            duration_text: duration_text.to_string(),
            quality_id,
            quality_name: quality_name.to_string(),
            status: DownloadStatus::Pending,
            progress: None,
            output_path: String::new(),
            error_message: None,
            retry_count: 0,
            max_retries: 3,
            created_at: now.clone(),
            updated_at: now,
            download_description: false,
            download_danmaku: false,
            download_subtitle: false,
            download_cover: false,
            audio_only: false,
        }
    }
}