import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { VideoInfo, DownloadTask, AppSettings, StartDownloadParams, QrGenerateResult, QrPollResult, BiliLoginInfo } from '@/types'

// ---------- 解析视频 ----------
export async function parseVideoUrl(url: string, cookieSessdata?: string): Promise<VideoInfo> {
  return invoke<VideoInfo>('parse_video_url', { url, cookieSessdata: cookieSessdata ?? null })
}

// ---------- 登录 ----------
export async function generateQr(): Promise<QrGenerateResult> {
  return invoke<QrGenerateResult>('generate_qr')
}

export async function pollQr(qrcodeKey: string): Promise<QrPollResult> {
  return invoke<QrPollResult>('poll_qr', { qrcodeKey })
}

export async function confirmLogin(
  sessdata: string,
  biliJct: string,
  dedeuserid: string
): Promise<BiliLoginInfo> {
  return invoke<BiliLoginInfo>('confirm_login', { sessdata, biliJct, dedeuserid })
}

export async function logout(): Promise<void> {
  return invoke('logout')
}

export async function getLoginStatus(): Promise<BiliLoginInfo | null> {
  return invoke<BiliLoginInfo | null>('get_login_status')
}

export async function getDataDir(): Promise<string> {
  return invoke<string>('get_data_dir')
}

// ---------- 任务管理 ----------
export async function startDownload(params: StartDownloadParams): Promise<DownloadTask> {
  return invoke<DownloadTask>('start_download', { params })
}

export async function pauseDownload(taskId: string): Promise<void> {
  return invoke('pause_download', { taskId })
}

export async function resumeDownload(taskId: string): Promise<void> {
  return invoke('resume_download', { taskId })
}

export async function cancelDownload(taskId: string): Promise<void> {
  return invoke('cancel_download', { taskId })
}

export async function retryDownload(taskId: string): Promise<void> {
  return invoke('retry_download', { taskId })
}

export async function listTasks(): Promise<DownloadTask[]> {
  return invoke<DownloadTask[]>('list_tasks')
}

export async function openOutputFolder(taskId: string): Promise<void> {
  return invoke('open_output_folder', { taskId })
}

export async function deleteDownload(taskId: string): Promise<void> {
  return invoke('delete_download', { taskId })
}

export async function loadTaskHistory(): Promise<DownloadTask[]> {
  return invoke<DownloadTask[]>('load_task_history')
}

// ---------- 下载进度事件 ----------
export interface ProgressEventPayload {
  task_id: string
  percentage: number
  downloaded_bytes: number
  total_bytes: number
  speed_bytes_per_sec: number
  eta_seconds: number
}

export function listenDownloadProgress(
  callback: (payload: ProgressEventPayload) => void
): Promise<UnlistenFn> {
  return listen<ProgressEventPayload>('download-progress', (event) => {
    callback(event.payload)
  })
}

export interface TaskStatusEventPayload {
  task_id: string
  status: string
  error_message: string | null
  output_path: string | null
}

export function listenTaskStatusChange(
  callback: (payload: TaskStatusEventPayload) => void
): Promise<UnlistenFn> {
  return listen<TaskStatusEventPayload>('task-status-change', (event) => {
    callback(event.payload)
  })
}

// ---------- 设置 ----------
export async function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>('get_settings')
}

export async function saveSettings(settings: AppSettings): Promise<void> {
  return invoke('save_settings', { settings })
}

export async function selectDownloadDir(): Promise<string> {
  return invoke<string>('select_download_dir')
}