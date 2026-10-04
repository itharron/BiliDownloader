use std::path::PathBuf;

/// Sidecar 管理器：负责 sidecar 二进制文件的检测、释放和调用
pub struct SidecarManager {
    sidecar_name: String,
}

impl SidecarManager {
    pub fn new(sidecar_name: &str) -> Self {
        Self {
            sidecar_name: sidecar_name.to_string(),
        }
    }

    /// 检测 sidecar 是否已存在
    pub fn check_exists(&self) -> bool {
        #[cfg(target_os = "windows")]
        let binary_name = format!("{}.exe", self.sidecar_name);
        #[cfg(not(target_os = "windows"))]
        let binary_name = self.sidecar_name.clone();

        let sidecar_path = self.sidecar_base_dir().join(&binary_name);
        sidecar_path.exists()
    }

    fn sidecar_base_dir(&self) -> PathBuf {
        let dir = std::env::current_exe()
            .unwrap_or_default()
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .to_path_buf();

        #[cfg(target_os = "macos")]
        {
            dir.join("../Resources")
        }
        #[cfg(not(target_os = "macos"))]
        {
            dir
        }
    }

    /// 构建 sidecar 调用参数（参数化，防止命令注入）
    pub fn build_args(
        &self,
        url: &str,
        output_dir: &str,
        quality_id: u32,
        filename_template: &str,
    ) -> Vec<String> {
        let mut args = vec![
            url.to_string(),
            "--output".to_string(),
            output_dir.to_string(),
            "--format".to_string(),
            format!("bestvideo[height<={}]+bestaudio", quality_id),
            "--output".to_string(),
            format!("{}/{}", output_dir, filename_template),
            "--no-playlist".to_string(),
            "--no-colors".to_string(),
            "--newline".to_string(),
            "--progress".to_string(),
            "--print".to_string(),
            "after_move:filepath".to_string(),
        ];

        if let Ok(proxy) = std::env::var("HTTP_PROXY") {
            if !proxy.is_empty() {
                args.push("--proxy".to_string());
                args.push(proxy);
            }
        }

        args
    }
}

impl Default for SidecarManager {
    fn default() -> Self {
        Self::new("yt-dlp")
    }
}