// ============================================================
// BiliDownloader 全局类型定义
// ============================================================

export interface VideoInfo {
  bvid: string
  aid: number
  cid: number
  title: string
  upName: string
  upAvatar: string
  coverUrl: string
  duration: number
  durationText: string
  viewCount: number
  danmakuCount: number
  qualities: QualityInfo[]
  pages: PageInfo[]
  hasSubtitle: boolean
}

export interface QualityInfo {
  id: number
  name: string
  description: string
  width: number
  height: number
  codecs: string
}

export interface PageInfo {
  cid: number
  page: number
  title: string
  duration: number
}

export interface DownloadProgress {
  percentage: number
  downloadedBytes: number
  totalBytes: number
  speedBytesPerSec: number
  etaSeconds: number
}

export type DownloadStatus =
  | 'pending'
  | 'parsing'
  | 'downloading'
  | 'paused'
  | 'completed'
  | 'failed'
  | 'cancelled'

export interface DownloadTask {
  id: string
  url: string
  title: string
  upName: string
  coverUrl: string
  durationText: string
  qualityId: number
  qualityName: string
  status: DownloadStatus
  progress: DownloadProgress | null
  outputPath: string
  errorMessage: string | null
  retryCount: number
  maxRetries: number
  createdAt: string
  updatedAt: string
  downloadDanmaku: boolean
  downloadSubtitle: boolean
  downloadCover: boolean
  audioOnly: boolean
}

export type ThemeMode = 'auto' | 'light' | 'dark'

export interface AppSettings {
  downloadDir: string
  maxConcurrent: number
  maxRetries: number
  theme: ThemeMode
  locale: string
  httpProxy: string | null
  cookieSessdata: string | null
  cookieBiliJct: string | null
  cookieDedeuserid: string | null
  loginUname: string | null
  loginAvatar: string | null
  filenameTemplate: string
  autoStart: boolean
  minimizeToTray: boolean
  showNotification: boolean
}

export interface StartDownloadParams {
  url: string
  qualityId: number
  audioOnly: boolean
  outputDir?: string
  filenameTemplate?: string
  downloadDanmaku: boolean
  downloadSubtitle: boolean
  downloadCover: boolean
  title?: string
  coverUrl?: string
  upName?: string
  durationText?: string
  qualityName?: string
}

// ============================================================
// 登录相关类型
// ============================================================

export interface QrGenerateResult {
  qrUrl: string
  qrcodeKey: string
}

export type QrStatus = 'pending' | 'scanned' | 'confirmed' | 'expired'

export interface QrPollResult {
  status: QrStatus
  message: string
  sessdata: string | null
  biliJct: string | null
  dedeuserid: string | null
  uname: string | null
}

export interface BiliLoginInfo {
  sessdata: string
  biliJct: string
  dedeuserid: string
  uname: string
  avatar: string
}

export const QUALITY_MAP: Record<number, string> = {
  127: '超高清 8K',
  126: '杜比视界',
  125: 'HDR 真彩',
  120: '超清 4K',
  116: '高清 1080P60',
  112: '高清 1080P+',
  80: '高清 1080P',
  74: '高清 720P60',
  64: '高清 720P',
  32: '清晰 480P',
  16: '流畅 360P',
}