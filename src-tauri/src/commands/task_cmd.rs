use crate::downloader::task_mgr::{execute_download, DownloadArgs, TaskManager};
use crate::models::settings::AppSettings;
use crate::models::task::{DownloadProgress, DownloadStatus, DownloadTask};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tauri_plugin_store::StoreExt;
use tokio::sync::Mutex;
use tracing::{error, info};

const STORE_FILENAME: &str = "settings.json";
const SETTINGS_KEY: &str = "app_settings";
const TASKS_HISTORY_KEY: &str = "download_history";

/// 将 TaskManager 中所有终态任务保存到 Store
async fn persist_task_history(app: &AppHandle, mgr: &TaskManager) {
    let tasks = mgr.get_all_tasks().await;
    let history: Vec<&DownloadTask> = tasks
        .iter()
        .filter(|t| {
            matches!(
                t.status,
                DownloadStatus::Completed | DownloadStatus::Failed | DownloadStatus::Cancelled
            )
        })
        .collect();
    if let Ok(store) = app.store(STORE_FILENAME) {
        let value = serde_json::to_value(history).unwrap_or_default();
        store.set(TASKS_HISTORY_KEY, value);
    }
}

/// 从 Store 加载历史任务并添加到 TaskManager
async fn restore_task_history(app: &AppHandle, mgr: &mut TaskManager) {
    if let Ok(store) = app.store(STORE_FILENAME) {
        if let Some(value) = store.get(TASKS_HISTORY_KEY) {
            if let Ok(history) = serde_json::from_value::<Vec<DownloadTask>>(value.clone()) {
                for task in history {
                    mgr.add_task(task);
                }
                info!("已加载 {} 条下载历史", mgr.len());
            }
        }
    }
}

pub struct TaskManagerState {
    pub manager: Arc<Mutex<TaskManager>>,
}

impl TaskManagerState {
    pub fn new() -> Self {
        Self {
            manager: Arc::new(Mutex::new(TaskManager::new(3))),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartDownloadParams {
    pub url: String,
    pub quality_id: u32,
    pub audio_only: bool,
    pub output_dir: Option<String>,
    pub filename_template: Option<String>,
    pub download_danmaku: bool,
    pub download_subtitle: bool,
    pub download_cover: bool,
    pub title: Option<String>,
    pub cover_url: Option<String>,
    pub up_name: Option<String>,
    pub duration_text: Option<String>,
    pub quality_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProgressEvent {
    pub task_id: String,
    pub percentage: f32,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub speed_bytes_per_sec: u64,
    pub eta_seconds: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaskStatusEvent {
    pub task_id: String,
    pub status: String,
    pub error_message: Option<String>,
    pub output_path: Option<String>,
}

#[tauri::command]
pub async fn start_download(
    app: AppHandle,
    state: tauri::State<'_, TaskManagerState>,
    params: StartDownloadParams,
) -> Result<DownloadTask, String> {
    info!("开始下载任务: {:?}", params);

    let mut task = DownloadTask::new(
        &params.url,
        params.quality_id,
        params.title.as_deref().unwrap_or(""),
        params.up_name.as_deref().unwrap_or(""),
        params.cover_url.as_deref().unwrap_or(""),
        params.duration_text.as_deref().unwrap_or(""),
        params.quality_name.as_deref().unwrap_or(""),
    );
    task.download_danmaku = params.download_danmaku;
    task.download_subtitle = params.download_subtitle;
    task.download_cover = params.download_cover;
    task.status = DownloadStatus::Downloading;

    // 读取设置中的 cookie、代理和下载目录
    let (cookies_sessdata, cookies_bili_jct, cookies_dedeuserid, proxy, settings_download_dir, settings_filename_template) = {
        if let Ok(store) = app.store(STORE_FILENAME) {
            if let Some(value) = store.get(SETTINGS_KEY) {
                let settings: AppSettings =
                    serde_json::from_value(value.clone()).unwrap_or_default();
                (
                    settings.cookie_sessdata,
                    settings.cookie_bili_jct,
                    settings.cookie_dedeuserid,
                    settings.http_proxy,
                    settings.download_dir,
                    settings.filename_template,
                )
            } else {
                (None, None, None, None, String::new(), String::new())
            }
        } else {
            (None, None, None, None, String::new(), String::new())
        }
    };

    // 输出目录优先级：params > 设置 > 系统下载目录
    let output_dir = params
        .output_dir
        .filter(|d| !d.is_empty())
        .or_else(|| {
            if !settings_download_dir.is_empty() {
                Some(settings_download_dir)
            } else {
                None
            }
        })
        .unwrap_or_else(|| dirs_next().unwrap_or_else(|| ".".to_string()));
    task.output_path = output_dir.clone();

    // 文件名模板：验证是否为有效的 yt-dlp 模板（必须包含 %( 语法）
    let filename_template = params
        .filename_template
        .filter(|t| !t.is_empty() && t.contains("%("))
        .or_else(|| {
            if !settings_filename_template.is_empty() && settings_filename_template.contains("%(") {
                Some(settings_filename_template)
            } else {
                None
            }
        })
        .unwrap_or_else(|| "[%(uploader)s] %(title)s.%(ext)s".to_string());

    let handle = {
        let mut mgr = state.manager.lock().await;
        mgr.add_task(task.clone())
    };

    let task_id = task.id.clone();
    let task_id_for_closure = task_id.clone();
    let task_clone = task.clone();
    let mgr = state.manager.clone();

    // 在后台启动下载任务
    tokio::spawn(async move {
        let download_args = DownloadArgs {
            url: params.url.clone(),
            output_dir,
            filename_template,
            quality_id: params.quality_id,
            audio_only: params.audio_only,
            cookies_sessdata,
            cookies_bili_jct,
            cookies_dedeuserid,
            proxy,
        };

        let app_for_progress = app.clone();
        let handle_for_progress = handle.clone();

        let result = execute_download(
            download_args,
            handle.clone(),
            move |progress: DownloadProgress| {
                // 同步写入 handle，确保 refreshTasks() 能读到进度
                if let Ok(mut guard) = handle_for_progress.try_lock() {
                    guard.task.progress = Some(DownloadProgress {
                        percentage: progress.percentage,
                        downloaded_bytes: progress.downloaded_bytes,
                        total_bytes: progress.total_bytes,
                        speed_bytes_per_sec: progress.speed_bytes_per_sec,
                        eta_seconds: progress.eta_seconds,
                    });
                    guard.task.status = DownloadStatus::Downloading;
                }
                // 发送实时事件到前端
                let event = ProgressEvent {
                    task_id: task_id_for_closure.clone(),
                    percentage: progress.percentage,
                    downloaded_bytes: progress.downloaded_bytes,
                    total_bytes: progress.total_bytes,
                    speed_bytes_per_sec: progress.speed_bytes_per_sec,
                    eta_seconds: progress.eta_seconds,
                };
                let _ = app_for_progress.emit("download-progress", event);
            },
        )
        .await;

        // 更新任务状态
        match result {
            Ok(output_file) => {
                info!("任务 {} 下载完成: {}", task_id, output_file);
                // 更新内存中的任务状态
                {
                    let mgr = mgr.lock().await;
                    if let Some(h) = mgr.get_task(&task_id) {
                        let mut guard = h.lock().await;
                        guard.task.status = DownloadStatus::Completed;
                        guard.task.output_path = output_file.clone();
                    }
                    drop(mgr);
                }
                // 持久化历史
                {
                    let mgr_lock = mgr.lock().await;
                    persist_task_history(&app, &mgr_lock).await;
                }
                let _ = app.emit(
                    "task-status-change",
                    TaskStatusEvent {
                        task_id: task_id.clone(),
                        status: "completed".to_string(),
                        error_message: None,
                        output_path: Some(output_file),
                    },
                );
            }
            Err(e) => {
                error!("任务 {} 下载失败: {}", task_id, e);
                {
                    let mgr = mgr.lock().await;
                    if let Some(h) = mgr.get_task(&task_id) {
                        let mut guard = h.lock().await;
                        guard.task.status = DownloadStatus::Failed;
                        guard.task.error_message = Some(e.clone());
                    }
                    drop(mgr);
                }
                // 持久化历史
                {
                    let mgr_lock = mgr.lock().await;
                    persist_task_history(&app, &mgr_lock).await;
                }
                let _ = app.emit(
                    "task-status-change",
                    TaskStatusEvent {
                        task_id: task_id.clone(),
                        status: "failed".to_string(),
                        error_message: Some(e),
                        output_path: None,
                    },
                );
            }
        }
    });

    Ok(task_clone)
}

#[tauri::command]
pub async fn pause_download(
    state: tauri::State<'_, TaskManagerState>,
    task_id: String,
) -> Result<(), String> {
    info!("暂停任务: {}", task_id);
    let mgr = state.manager.lock().await;
    mgr.cancel_task(&task_id).await;
    if let Some(h) = mgr.get_task(&task_id) {
        let mut guard = h.lock().await;
        guard.task.status = DownloadStatus::Paused;
    }
    Ok(())
}

#[tauri::command]
pub async fn resume_download(
    state: tauri::State<'_, TaskManagerState>,
    task_id: String,
) -> Result<(), String> {
    info!("恢复任务: {}", task_id);
    let mgr = state.manager.lock().await;
    if let Some(h) = mgr.get_task(&task_id) {
        let mut guard = h.lock().await;
        guard.task.status = DownloadStatus::Downloading;
    }
    Ok(())
}

#[tauri::command]
pub async fn cancel_download(
    state: tauri::State<'_, TaskManagerState>,
    task_id: String,
) -> Result<(), String> {
    info!("取消任务: {}", task_id);
    let mgr = state.manager.lock().await;
    mgr.cancel_task(&task_id).await;
    if let Some(h) = mgr.get_task(&task_id) {
        let mut guard = h.lock().await;
        guard.task.status = DownloadStatus::Cancelled;
    }
    Ok(())
}

#[tauri::command]
pub async fn retry_download(
    state: tauri::State<'_, TaskManagerState>,
    task_id: String,
) -> Result<(), String> {
    info!("重试任务: {}", task_id);
    let mgr = state.manager.lock().await;
    if let Some(h) = mgr.get_task(&task_id) {
        let mut guard = h.lock().await;
        guard.task.status = DownloadStatus::Pending;
        guard.task.retry_count += 1;
    }
    Ok(())
}

#[tauri::command]
pub async fn load_task_history(
    app: AppHandle,
    state: tauri::State<'_, TaskManagerState>,
) -> Result<Vec<DownloadTask>, String> {
    let mut mgr = state.manager.lock().await;
    restore_task_history(&app, &mut mgr).await;
    Ok(mgr.get_all_tasks().await)
}

#[tauri::command]
pub async fn list_tasks(state: tauri::State<'_, TaskManagerState>) -> Result<Vec<DownloadTask>, String> {
    let mgr = state.manager.lock().await;
    Ok(mgr.get_all_tasks().await)
}

#[tauri::command]
pub async fn open_output_folder(state: tauri::State<'_, TaskManagerState>, task_id: String) -> Result<(), String> {
    let mgr = state.manager.lock().await;
    if let Some(h) = mgr.get_task(&task_id) {
        let guard = h.lock().await;
        let path = if !guard.task.output_path.is_empty() {
            guard.task.output_path.clone()
        } else {
            return Err("输出路径为空".to_string());
        };
        drop(guard);
        if let Some(parent) = std::path::Path::new(&path).parent() {
            let _ = open::that(parent);
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn delete_download(
    app: AppHandle,
    state: tauri::State<'_, TaskManagerState>,
    task_id: String,
) -> Result<(), String> {
    info!("删除任务: {}", task_id);
    // 先尝试取消正在运行的任务
    {
        let mgr = state.manager.lock().await;
        mgr.cancel_task(&task_id).await;
    }
    // 从列表移除
    {
        let mut mgr = state.manager.lock().await;
        mgr.remove_task(&task_id);
        // 持久化历史
        persist_task_history(&app, &mgr).await;
    }
    info!("任务已删除: {}", task_id);
    Ok(())
}

fn dirs_next() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("USERPROFILE")
            .ok()
            .map(|p| format!("{}\\Downloads", p))
    }
    #[cfg(target_os = "macos")]
    {
        dirs_next_mac()
    }
    #[cfg(target_os = "linux")]
    {
        dirs_next_linux()
    }
}

#[cfg(target_os = "macos")]
fn dirs_next_mac() -> Option<String> {
    std::env::var("HOME")
        .ok()
        .map(|p| format!("{}/Downloads", p))
}

#[cfg(target_os = "linux")]
fn dirs_next_linux() -> Option<String> {
    std::env::var("HOME")
        .ok()
        .map(|p| format!("{}/Downloads", p))
}