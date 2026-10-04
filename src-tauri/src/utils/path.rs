use crate::utils::error::{AppError, AppResult};
use std::path::{Path, PathBuf};

/// 获取应用数据目录
pub fn app_data_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("BiliDownloader")
    }
    #[cfg(target_os = "macos")]
    {
        dirs_fallback()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Library")
            .join("Application Support")
            .join("BiliDownloader")
    }
    #[cfg(target_os = "linux")]
    {
        dirs_fallback()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config")
            .join("bili-downloader")
    }
}

/// 获取侧载二进制文件目录
pub fn sidecar_dir() -> PathBuf {
    app_data_dir().join("binaries")
}

/// 获取下载输出目录
pub fn download_output_dir(custom_dir: Option<&str>) -> PathBuf {
    if let Some(dir) = custom_dir {
        let path = PathBuf::from(dir);
        if path.exists() {
            return path;
        }
    }
    default_download_dir()
}

fn default_download_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        std::env::var("USERPROFILE")
            .map(|p| PathBuf::from(p).join("Downloads"))
            .unwrap_or_else(|_| PathBuf::from("./downloads"))
    }
    #[cfg(not(target_os = "windows"))]
    {
        dirs_fallback()
            .map(|d| d.join("Downloads"))
            .unwrap_or_else(|| PathBuf::from("./downloads"))
    }
}

/// 校验输出路径是否在允许范围内
pub fn validate_output_path(path: &Path) -> AppResult<()> {
    let normalized = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

    let app_data = app_data_dir();
    let downloads = default_download_dir();

    let in_app_data = normalized.starts_with(&app_data);
    let in_downloads = normalized.starts_with(&downloads);

    if in_app_data || in_downloads {
        Ok(())
    } else {
        Err(AppError::PathNotSafe(format!(
            "路径 '{}' 不在允许的目录范围内（仅允许应用数据目录和下载目录）",
            normalized.display()
        )))
    }
}

#[allow(dead_code)]
fn dirs_fallback() -> Option<PathBuf> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()
        .map(PathBuf::from)
}