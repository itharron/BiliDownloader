use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoInfo {
    pub bvid: String,
    pub aid: u64,
    pub cid: u64,
    pub title: String,
    pub up_name: String,
    pub up_avatar: String,
    pub cover_url: String,
    pub duration: u64,
    pub duration_text: String,
    pub view_count: u64,
    pub danmaku_count: u64,
    pub qualities: Vec<QualityInfo>,
    pub pages: Vec<PageInfo>,
    pub has_subtitle: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualityInfo {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub width: u32,
    pub height: u32,
    pub codecs: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    pub cid: u64,
    pub page: u32,
    pub title: String,
    pub duration: u64,
}

impl VideoInfo {
    pub fn quality_display_list(&self) -> Vec<String> {
        self.qualities
            .iter()
            .map(|q| format!("{} ({})", q.description, q.name))
            .collect()
    }
}