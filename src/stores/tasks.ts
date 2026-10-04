import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { DownloadTask, DownloadStatus } from '@/types'
import {
  listTasks,
  pauseDownload,
  resumeDownload,
  cancelDownload,
  retryDownload,
  openOutputFolder,
  deleteDownload,
  loadTaskHistory,
  listenDownloadProgress,
  listenTaskStatusChange,
  type ProgressEventPayload,
  type TaskStatusEventPayload,
} from '@/services/tauri'
import type { UnlistenFn } from '@tauri-apps/api/event'

export const useTaskStore = defineStore('tasks', () => {
  const tasks = ref<DownloadTask[]>([])
  const loading = ref(false)
  let historyLoaded = false

  let unlistenProgress: UnlistenFn | null = null
  let unlistenStatus: UnlistenFn | null = null

  const activeTasks = computed(() =>
    tasks.value.filter((t) => t.status === 'pending' || t.status === 'downloading')
  )

  const completedTasks = computed(() =>
    tasks.value.filter((t) => t.status === 'completed')
  )

  const failedTasks = computed(() =>
    tasks.value.filter((t) => t.status === 'failed')
  )

  async function initEventListeners() {
    // 监听下载进度
    unlistenProgress = await listenDownloadProgress((payload: ProgressEventPayload) => {
      const task = tasks.value.find((t) => t.id === payload.task_id)
      if (task) {
        task.status = 'downloading'
        task.progress = {
          percentage: payload.percentage,
          downloadedBytes: payload.downloaded_bytes,
          totalBytes: payload.total_bytes,
          speedBytesPerSec: payload.speed_bytes_per_sec,
          etaSeconds: payload.eta_seconds,
        }
      }
    })

    // 监听任务状态变化
    unlistenStatus = await listenTaskStatusChange((payload: TaskStatusEventPayload) => {
      const task = tasks.value.find((t) => t.id === payload.task_id)
      if (task) {
        task.status = payload.status as DownloadStatus
        if (payload.error_message !== null && payload.error_message !== undefined) {
          task.errorMessage = payload.error_message
        }
        if (payload.output_path) {
          task.outputPath = payload.output_path
        }
      }
    })
  }

  function destroyEventListeners() {
    if (unlistenProgress) {
      unlistenProgress()
      unlistenProgress = null
    }
    if (unlistenStatus) {
      unlistenStatus()
      unlistenStatus = null
    }
  }

  async function refreshTasks() {
    loading.value = true
    try {
      if (!historyLoaded) {
        tasks.value = await loadTaskHistory()
        historyLoaded = true
      } else {
        tasks.value = await listTasks()
      }
    } finally {
      loading.value = false
    }
  }

  function addTask(task: DownloadTask) {
    tasks.value.unshift(task)
  }

  function updateTaskProgress(taskId: string, progress: DownloadTask['progress']) {
    const task = tasks.value.find((t) => t.id === taskId)
    if (task) {
      task.progress = progress
      task.status = 'downloading'
    }
  }

  function updateTaskStatus(taskId: string, status: DownloadStatus, errorMsg?: string) {
    const task = tasks.value.find((t) => t.id === taskId)
    if (task) {
      task.status = status
      if (errorMsg) {
        task.errorMessage = errorMsg
      }
    }
  }

  async function pauseTask(taskId: string) {
    await pauseDownload(taskId)
    updateTaskStatus(taskId, 'paused')
  }

  async function resumeTask(taskId: string) {
    await resumeDownload(taskId)
    updateTaskStatus(taskId, 'pending')
  }

  async function cancelTask(taskId: string) {
    await cancelDownload(taskId)
    updateTaskStatus(taskId, 'cancelled')
  }

  async function retryTask(taskId: string) {
    await retryDownload(taskId)
    updateTaskStatus(taskId, 'pending')
  }

  async function removeTask(taskId: string) {
    await deleteDownload(taskId)
    tasks.value = tasks.value.filter(t => t.id !== taskId)
  }

  return {
    tasks,
    loading,
    activeTasks,
    completedTasks,
    failedTasks,
    initEventListeners,
    destroyEventListeners,
    refreshTasks,
    addTask,
    updateTaskProgress,
    updateTaskStatus,
    pauseTask,
    resumeTask,
    cancelTask,
    retryTask,
    removeTask,
    openOutputFolder,
  }
})