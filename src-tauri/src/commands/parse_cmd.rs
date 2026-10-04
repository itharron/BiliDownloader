use crate::models::video_info::VideoInfo;
use crate::parser::{bili_parser::parse_bilibili_video, url_validator::validate_bili_url};
use anyhow::Result;

#[tauri::command]
pub async fn parse_video_url(url: String, cookie_sessdata: Option<String>) -> Result<VideoInfo, String> {
    validate_bili_url(&url).map_err(|e| e.to_string())?;

    parse_bilibili_video(&url, cookie_sessdata.as_deref())
        .await
        .map_err(|e| format!("解析视频信息失败: {}", e))
}