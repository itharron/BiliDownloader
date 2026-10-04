use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub download_dir: String,
    pub max_concurrent: u32,
    pub max_retries: u32,
    pub theme: ThemeMode,
    pub locale: String,
    pub http_proxy: Option<String>,
    pub cookie_sessdata: Option<String>,
    pub cookie_bili_jct: Option<String>,
    pub cookie_dedeuserid: Option<String>,
    pub login_uname: Option<String>,
    pub login_avatar: Option<String>,
    pub filename_template: String,
    pub auto_start: bool,
    pub minimize_to_tray: bool,
    pub show_notification: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThemeMode {
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "light")]
    Light,
    #[serde(rename = "dark")]
    Dark,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            download_dir: String::new(),
            max_concurrent: 3,
            max_retries: 3,
            theme: ThemeMode::Auto,
            locale: "zh-CN".to_string(),
            http_proxy: None,
            cookie_sessdata: None,
            cookie_bili_jct: None,
            cookie_dedeuserid: None,
            login_uname: None,
            login_avatar: None,
            filename_template: "[%(uploader)s] %(title)s.%(ext)s".to_string(),
            auto_start: false,
            minimize_to_tray: true,
            show_notification: true,
        }
    }
}