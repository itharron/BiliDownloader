use crate::models::task::{DownloadProgress, DownloadStatus, DownloadTask};
use std::collections::HashMap;
use std::io::Write;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command as TokioCommand;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use super::progress::ProgressParser;

/// 读取一行（容忍非 UTF-8 字节，使用 lossy 转换）
async fn read_lossy_line<R: tokio::io::AsyncRead + Unpin>(
    reader: &mut BufReader<R>,
) -> std::io::Result<Option<String>> {
    let mut buf = Vec::new();
    let n = reader.read_until(b'\n', &mut buf).await?;
    if n == 0 {
        return Ok(None);
    }
    if buf.ends_with(&[b'\n']) {
        buf.pop();
    }
    if buf.ends_with(&[b'\r']) {
        buf.pop();
    }
    Ok(Some(String::from_utf8_lossy(&buf).to_string()))
}

pub struct TaskManager {
    tasks: HashMap<String, Arc<Mutex<TaskHandle>>>,
    #[allow(dead_code)]
    max_concurrent: usize,
}

pub struct TaskHandle {
    pub task: DownloadTask,
    pub cancel_token: tokio_util::sync::CancellationToken,
}

impl TaskManager {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            tasks: HashMap::new(),
            max_concurrent,
        }
    }

    pub fn add_task(&mut self, task: DownloadTask) -> Arc<Mutex<TaskHandle>> {
        let id = task.id.clone();
        let handle = Arc::new(Mutex::new(TaskHandle {
            task,
            cancel_token: tokio_util::sync::CancellationToken::new(),
        }));
        self.tasks.insert(id, handle.clone());
        handle
    }

    pub fn get_task(&self, id: &str) -> Option<Arc<Mutex<TaskHandle>>> {
        self.tasks.get(id).cloned()
    }

    pub async fn get_all_tasks(&self) -> Vec<DownloadTask> {
        let mut tasks = Vec::new();
        for h in self.tasks.values() {
            tasks.push(h.lock().await.task.clone());
        }
        tasks
    }

    pub async fn cancel_task(&self, id: &str) {
        if let Some(handle) = self.tasks.get(id) {
            let h = handle.lock().await;
            h.cancel_token.cancel();
            drop(h);
            info!("任务取消信号已发送: {}", id);
        }
    }

    pub fn remove_task(&mut self, id: &str) {
        self.tasks.remove(id);
    }

    pub async fn active_count(&self) -> usize {
        let mut count = 0;
        for h in self.tasks.values() {
            if h.lock().await.task.status == DownloadStatus::Downloading {
                count += 1;
            }
        }
        count
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }
}

/// yt-dlp 下载参数
pub struct DownloadArgs {
    pub url: String,
    pub output_dir: String,
    pub filename_template: String,
    pub quality_id: u32,
    pub audio_only: bool,
    pub cookies_sessdata: Option<String>,
    pub cookies_bili_jct: Option<String>,
    pub cookies_dedeuserid: Option<String>,
    pub proxy: Option<String>,
}

/// 自动检测 ffmpeg 路径
fn find_ffmpeg() -> Option<String> {
    // 1. 先检查 PATH
    let exe_name = if cfg!(target_os = "windows") {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    };
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let full = dir.join(exe_name);
            if full.exists() {
                return Some(full.to_string_lossy().to_string());
            }
        }
    }
    // 2. 常见安装路径
    #[cfg(target_os = "windows")]
    {
        // winget
        if let Ok(users) = std::fs::read_dir("C:\\Users") {
            for user_entry in users.flatten() {
                let pkg_dir = user_entry
                    .path()
                    .join("AppData\\Local\\Microsoft\\WinGet\\Packages");
                if let Ok(pkgs) = std::fs::read_dir(&pkg_dir) {
                    for pkg in pkgs.flatten() {
                        let name = pkg.file_name().to_string_lossy().to_string();
                        if name.contains("FFmpeg") || name.contains("ffmpeg") {
                            let ffmpeg_path = pkg.path().to_path_buf();
                            // 遍历子目录找 ffmpeg.exe
                            fn search_bin(dir: &std::path::Path) -> Option<String> {
                                if let Ok(entries) = std::fs::read_dir(dir) {
                                    for e in entries.flatten() {
                                        let p = e.path();
                                        if p.is_dir() {
                                            let bin = p.join("bin").join("ffmpeg.exe");
                                            if bin.exists() {
                                                return Some(bin.to_string_lossy().to_string());
                                            }
                                            if let Some(found) = search_bin(&p) {
                                                return Some(found);
                                            }
                                        }
                                    }
                                }
                                None
                            }
                            if let Some(found) = search_bin(&ffmpeg_path) {
                                return Some(found);
                            }
                        }
                    }
                }
            }
        }
        // chocolatey
        let choco = std::path::Path::new("C:\\ProgramData\\chocolatey\\bin\\ffmpeg.exe");
        if choco.exists() {
            return Some(choco.to_string_lossy().to_string());
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        for candidate in &["/usr/bin/ffmpeg", "/usr/local/bin/ffmpeg", "/opt/homebrew/bin/ffmpeg"] {
            let p = std::path::Path::new(candidate);
            if p.exists() {
                return Some(p.to_string_lossy().to_string());
            }
        }
    }
    None
}
fn quality_to_ytdlp_format(quality_id: u32) -> &'static str {
    match quality_id {
        127 => "bestvideo[height<=4320]+bestaudio/best[height<=4320]",
        125 => "bestvideo+bestaudio/best",
        120 => "bestvideo[height<=2160]+bestaudio/best[height<=2160]",
        116 => "bestvideo[height<=1080][fps>30]+bestaudio/best[height<=1080]",
        112 => "bestvideo[height<=1080]+bestaudio/best[height<=1080]",
        80  => "bestvideo[height<=1080]+bestaudio/best[height<=1080]",
        74  => "bestvideo[height<=720][fps>30]+bestaudio/best[height<=720]",
        64  => "bestvideo[height<=720]+bestaudio/best[height<=720]",
        32  => "bestvideo[height<=480]+bestaudio/best[height<=480]",
        16  => "bestvideo[height<=360]+bestaudio/best[height<=360]",
        _   => "bestvideo+bestaudio/best",
    }
}

/// 生成 Netscape 格式的 cookie 文本
fn build_cookie_netscape(sessdata: &str, bili_jct: &str, dedeuserid: &str) -> String {
    let domain = ".bilibili.com";
    let flag = "TRUE";
    let path = "/";
    let secure = "TRUE";
    let expiration = (chrono::Utc::now() + chrono::Duration::days(30)).timestamp();

    format!(
        "# Netscape HTTP Cookie File\n\
         # This is a generated file! Do not edit.\n\n\
         {domain}\t{flag}\t{path}\t{secure}\t{expiration}\tSESSDATA\t{sessdata}\n\
         {domain}\t{flag}\t{path}\t{secure}\t{expiration}\tbili_jct\t{bili_jct}\n\
         {domain}\t{flag}\t{path}\t{secure}\t{expiration}\tDedeUserID\t{dedeuserid}\n",
        domain = domain,
        flag = flag,
        path = path,
        secure = secure,
        expiration = expiration,
        sessdata = sessdata,
        bili_jct = bili_jct,
        dedeuserid = dedeuserid,
    )
}

/// 执行下载任务：启动 yt-dlp 进程，解析输出，通过回调推送进度
pub async fn execute_download<F>(
    args: DownloadArgs,
    handle: Arc<Mutex<TaskHandle>>,
    progress_callback: F,
) -> Result<String, String>
where
    F: Fn(DownloadProgress) + Send + 'static,
{
    let task_id;
    let cancel_token;
    {
        let h = handle.lock().await;
        task_id = h.task.id.clone();
        cancel_token = h.cancel_token.clone();
    }

    info!("开始执行下载任务: {} (url={})", task_id, args.url);

    // 构建 yt-dlp 参数
    let mut cmd_args: Vec<String> = vec![
        args.url.clone(),
        "--newline".to_string(),
        "--progress".to_string(),
        "--no-colors".to_string(),
        "--no-playlist".to_string(),
        "--add-header".to_string(),
        "Referer:https://www.bilibili.com".to_string(),
        "--add-header".to_string(),
        "User-Agent:Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36".to_string(),
        "--add-header".to_string(),
        "Origin:https://www.bilibili.com".to_string(),
        "--embed-metadata".to_string(),
        "--print".to_string(),
        "after_move:filepath".to_string(),
        "-o".to_string(),
        format!(
            "{}/{}",
            args.output_dir,
            args.filename_template
        ),
    ];

    // 清晰度选择：使用 yt-dlp 格式选择器
    if args.audio_only {
        cmd_args.push("-f".to_string());
        cmd_args.push("bestaudio/best".to_string());
        cmd_args.push("-x".to_string());
        cmd_args.push("--audio-format".to_string());
        cmd_args.push("mp3".to_string());
    } else if args.quality_id > 0 {
        cmd_args.push("-f".to_string());
        cmd_args.push(quality_to_ytdlp_format(args.quality_id).to_string());
        cmd_args.push("--merge-output-format".to_string());
        cmd_args.push("mp4".to_string());
    }

    // Cookie 文件：生成临时 Netscape 格式 cookie 文件
    let cookie_file: Option<std::path::PathBuf> = if let Some(ref sessdata) = args.cookies_sessdata {
        if !sessdata.is_empty() {
            let bili_jct = args.cookies_bili_jct.as_deref().unwrap_or("");
            let dedeuserid = args.cookies_dedeuserid.as_deref().unwrap_or("0");
            let cookie_content = build_cookie_netscape(sessdata, bili_jct, dedeuserid);
            let tmp = std::env::temp_dir().join(format!("bili_cookies_{}.txt", uuid::Uuid::new_v4()));
            let mut f = std::fs::File::create(&tmp)
                .map_err(|e| format!("无法创建临时 cookie 文件: {}", e))?;
            f.write_all(cookie_content.as_bytes())
                .map_err(|e| format!("无法写入 cookie 文件: {}", e))?;
            cmd_args.push("--cookies".to_string());
            cmd_args.push(tmp.to_string_lossy().to_string());
            Some(tmp)
        } else {
            None
        }
    } else {
        None
    };

    // 代理
    if let Some(ref proxy) = args.proxy {
        if !proxy.is_empty() {
            cmd_args.push("--proxy".to_string());
            cmd_args.push(proxy.clone());
        }
    }

    // 自动检测 ffmpeg 路径（PATH 可能未刷新，显式传入避免报错）
    if let Some(ffmpeg_path) = find_ffmpeg() {
        cmd_args.push("--ffmpeg-location".to_string());
        cmd_args.push(ffmpeg_path);
    }

    info!("yt-dlp 参数: {:?}", cmd_args);

    // 启动 yt-dlp 进程
    let mut child = TokioCommand::new("yt-dlp")
        .args(&cmd_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("无法启动 yt-dlp: {}. 请确认已安装 yt-dlp。", e))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "无法获取 yt-dlp stdout (进程已启动但管道不可用)".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "无法获取 yt-dlp stderr (进程已启动但管道不可用)".to_string())?;

    let reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    let mut output_path = String::new();

    // 并行读取 stdout 和 stderr，各自返回 (path, error_lines)
    let stdout_handle = {
        let cancel = cancel_token.clone();
        tokio::spawn(async move {
            let mut lines = reader;
            let mut path = String::new();
            let mut err_lines: Vec<String> = Vec::new();
            loop {
                tokio::select! {
                    _ = cancel.cancelled() => {
                        info!("下载任务被取消: stdout 读取停止");
                        break;
                    }
                    result = read_lossy_line(&mut lines) => {
                        match result {
                            Ok(Some(line)) => {
                                // 检测文件路径输出（不以 [ 开头，且以常见媒体扩展名结尾）
                                let is_filepath = !line.starts_with('[')
                                    && (line.ends_with(".mp4")
                                        || line.ends_with(".mkv")
                                        || line.ends_with(".flv")
                                        || line.ends_with(".webm")
                                        || line.ends_with(".mp3")
                                        || line.ends_with(".m4a")
                                        || line.ends_with(".aac")
                                        || line.ends_with(".opus"));
                                if is_filepath {
                                    path = line.clone();
                                } else if !line.starts_with("[download]") && !line.is_empty() {
                                    err_lines.push(line);
                                }
                            }
                            Ok(None) => break,
                            Err(e) => {
                                warn!("读取 yt-dlp stdout 出错: {}", e);
                                break;
                            }
                        }
                    }
                }
            }
            (path, err_lines)
        })
    };

    let stderr_handle = {
        let cancel = cancel_token.clone();
        tokio::spawn(async move {
            let mut lines = stderr_reader;
            let mut last_progress = DownloadProgress {
                percentage: 0.0,
                downloaded_bytes: 0,
                total_bytes: 0,
                speed_bytes_per_sec: 0,
                eta_seconds: 0,
            };
            let mut error_lines: Vec<String> = Vec::new();
            loop {
                tokio::select! {
                    _ = cancel.cancelled() => {
                        info!("下载任务被取消: stderr 读取停止");
                        break;
                    }
                    result = read_lossy_line(&mut lines) => {
                        match result {
                            Ok(Some(line)) => {
                                if let Some(progress) = ProgressParser::parse_ytdlp_line(&line) {
                                    last_progress = progress.clone();
                                    progress_callback(progress);
                                } else {
                                    // 收集非进度行（可能是错误信息）
                                    debug!("yt-dlp stderr: {}", line);
                                    error_lines.push(line);
                                }
                            }
                            Ok(None) => break,
                            Err(e) => {
                                warn!("读取 yt-dlp stderr 出错: {}", e);
                                break;
                            }
                        }
                    }
                }
            }
            (last_progress, error_lines)
        })
    };

    // 等待进程结束
    let status = child.wait().await.map_err(|e| format!("等待 yt-dlp 进程出错: {}", e))?;

    let (stdout_path, stdout_err_lines) = stdout_handle
        .await
        .map_err(|e| format!("stdout 读取任务失败: {}", e))?;

    let stderr_result = stderr_handle
        .await
        .map_err(|e| format!("stderr 读取任务失败: {}", e))?;
    let (_last_progress, stderr_err_lines) = stderr_result;

    // 合并两边的错误行
    let mut error_lines = stdout_err_lines;
    error_lines.extend(stderr_err_lines);

    if !stdout_path.is_empty() {
        output_path = stdout_path;
    }

    if cancel_token.is_cancelled() {
        // 清理临时 cookie 文件
        if let Some(ref cookie_path) = cookie_file {
            let _ = std::fs::remove_file(cookie_path);
        }
        info!("下载任务已被取消: {}", task_id);
        return Err("任务已取消".to_string());
    }

    // 清理临时 cookie 文件
    if let Some(ref cookie_path) = cookie_file {
        let _ = std::fs::remove_file(cookie_path);
    }

    if status.success() {
        info!("下载完成: {} -> {}", task_id, output_path);
        Ok(output_path)
    } else {
        let exit_code = status.code().unwrap_or(-1);
        let stderr_summary: String = if error_lines.is_empty() {
            String::new()
        } else {
            // 取最后几行有用的错误信息（包含 WARNING/ERROR 等所有行）
            let relevant: Vec<&str> = error_lines
                .iter()
                .map(|s| s.as_str())
                .filter(|s| !s.is_empty())
                .collect();
            let summary: String = relevant.join(" | ");
            if summary.len() > 500 {
                format!(": {}", &summary[..500])
            } else {
                format!(": {}", summary)
            }
        };
        error!(
            "下载失败: {} (exit_code={}, output_path={}){}",
            task_id, exit_code, output_path, stderr_summary
        );
        Err(format!(
            "yt-dlp 异常退出 (code={}){}",
            exit_code, stderr_summary
        ))
    }
}