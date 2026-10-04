<script setup lang="ts">
import { onActivated } from 'vue'
import { useTaskStore } from '@/stores/tasks'
import { Icon } from '@iconify/vue'

const taskStore = useTaskStore()

onActivated(() => {
  taskStore.refreshTasks()
})

const statusLabels: Record<string, string> = {
  pending: '等待中',
  parsing: '解析中',
  downloading: '下载中',
  paused: '已暂停',
  completed: '已完成',
  failed: '失败',
  cancelled: '已取消',
}

const statusColors: Record<string, string> = {
  pending: 'var(--color-info)',
  parsing: 'var(--color-info)',
  downloading: 'var(--color-primary)',
  paused: 'var(--color-warning)',
  completed: 'var(--color-success)',
  failed: 'var(--color-danger)',
  cancelled: 'var(--text-placeholder)',
}

function formatSpeed(bytesPerSec: number): string {
  if (bytesPerSec === 0) return '--'
  const units = ['B/s', 'KB/s', 'MB/s', 'GB/s']
  let i = 0
  let speed = bytesPerSec
  while (speed >= 1024 && i < units.length - 1) {
    speed /= 1024
    i++
  }
  return `${speed.toFixed(1)} ${units[i]}`
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB']
  let i = 0
  let size = bytes
  while (size >= 1024 && i < units.length - 1) {
    size /= 1024
    i++
  }
  return `${size.toFixed(1)} ${units[i]}`
}

function formatEta(seconds: number): string {
  if (seconds <= 0) return '--'
  const m = Math.floor(seconds / 60)
  const s = seconds % 60
  return `${m}:${s.toString().padStart(2, '0')}`
}

function formatTime(dateStr: string): string {
  if (!dateStr) return ''
  try {
    const d = new Date(dateStr)
    return d.toLocaleString('zh-CN', {
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    })
  } catch {
    return dateStr
  }
}

function getProgressPercentage(task: any): number {
  if (task.progress) {
    return Math.round(task.progress.percentage)
  }
  if (task.status === 'completed') return 100
  return 0
}

function getProgressColor(task: any): string {
  if (task.status === 'completed') return 'var(--color-success)'
  if (task.status === 'failed') return 'var(--color-danger)'
  if (task.status === 'paused') return 'var(--color-warning)'
  return 'var(--color-primary)'
}
</script>

<template>
  <div class="tasks-view">
    <header class="page-header">
      <h2>下载任务</h2>
      <span class="task-count">{{ taskStore.tasks.length }} 个任务</span>
    </header>

    <div v-if="taskStore.tasks.length === 0" class="empty-state">
      <Icon icon="mdi:download-off" :width="48" />
      <p>暂无下载任务</p>
      <p class="sub">前往首页解析视频并开始下载</p>
    </div>

    <div v-else class="task-list">
      <div
        v-for="task in taskStore.tasks"
        :key="task.id"
        class="task-card"
      >
        <!-- 封面 -->
        <div class="task-cover">
          <img
            v-if="task.coverUrl"
            :src="task.coverUrl"
            :alt="task.title"
            referrerpolicy="no-referrer"
          />
          <div v-else class="task-cover-placeholder">
            <Icon icon="mdi:video-outline" :width="32" />
          </div>
        </div>

        <!-- 主体信息 -->
        <div class="task-body">
          <div class="task-title" :title="task.title || task.url">
            {{ task.title || task.url }}
          </div>
          <div class="task-meta">
            <span v-if="task.upName" class="meta-up">
              <Icon icon="mdi:account-circle-outline" :width="14" />
              {{ task.upName }}
            </span>
            <span v-if="task.durationText" class="meta-duration">
              <Icon icon="mdi:clock-outline" :width="14" />
              {{ task.durationText }}
            </span>
            <span v-if="task.qualityName" class="meta-quality">
              <Icon icon="mdi:video-box" :width="14" />
              {{ task.qualityName }}
            </span>
          </div>

          <!-- 进度条 -->
          <div class="task-progress">
            <el-progress
              :percentage="getProgressPercentage(task)"
              :color="getProgressColor(task)"
              :stroke-width="6"
              :show-text="false"
            />
            <div class="progress-stats" v-if="task.progress && task.status === 'downloading'">
              <span>{{ task.progress.percentage.toFixed(1) }}%</span>
              <span>{{ formatBytes(task.progress.downloadedBytes) }} / {{ formatBytes(task.progress.totalBytes) }}</span>
              <span>{{ formatSpeed(task.progress.speedBytesPerSec) }}</span>
              <span>剩余 {{ formatEta(task.progress.etaSeconds) }}</span>
            </div>
            <div class="progress-stats" v-else-if="task.status === 'completed'">
              <span class="stat-completed">下载完成</span>
            </div>
            <div class="progress-stats" v-else-if="task.status === 'failed' && task.errorMessage">
              <span class="stat-error">{{ task.errorMessage }}</span>
            </div>
          </div>
        </div>

        <!-- 状态标签 -->
        <div class="task-status-badge" :style="{ background: statusColors[task.status] + '20', color: statusColors[task.status] }">
          {{ statusLabels[task.status] || task.status }}
        </div>

        <!-- 时间 -->
        <div class="task-time" v-if="task.createdAt">
          {{ formatTime(task.createdAt) }}
        </div>

        <!-- 操作按钮 -->
        <div class="task-actions">
          <!-- 下载中：暂停 + 取消 -->
          <template v-if="task.status === 'downloading'">
            <el-tooltip content="暂停" placement="top">
              <button class="action-btn" @click="taskStore.pauseTask(task.id)">
                <Icon icon="mdi:pause" :width="20" />
              </button>
            </el-tooltip>
            <el-tooltip content="取消" placement="top">
              <button class="action-btn action-btn-danger" @click="taskStore.cancelTask(task.id)">
                <Icon icon="mdi:close" :width="20" />
              </button>
            </el-tooltip>
          </template>

          <!-- 已暂停：继续 + 取消 -->
          <template v-else-if="task.status === 'paused'">
            <el-tooltip content="继续下载" placement="top">
              <button class="action-btn" @click="taskStore.resumeTask(task.id)">
                <Icon icon="mdi:play" :width="20" />
              </button>
            </el-tooltip>
            <el-tooltip content="取消" placement="top">
              <button class="action-btn action-btn-danger" @click="taskStore.cancelTask(task.id)">
                <Icon icon="mdi:close" :width="20" />
              </button>
            </el-tooltip>
          </template>

          <!-- 失败：重试 -->
          <template v-else-if="task.status === 'failed'">
            <el-tooltip content="重试" placement="top">
              <button class="action-btn" @click="taskStore.retryTask(task.id)">
                <Icon icon="mdi:refresh" :width="20" />
              </button>
            </el-tooltip>
            <el-tooltip content="取消" placement="top">
              <button class="action-btn action-btn-danger" @click="taskStore.cancelTask(task.id)">
                <Icon icon="mdi:close" :width="20" />
              </button>
            </el-tooltip>
          </template>

          <!-- 等待中 / 解析中：取消 -->
          <template v-else-if="task.status === 'pending' || task.status === 'parsing'">
            <el-tooltip content="取消" placement="top">
              <button class="action-btn action-btn-danger" @click="taskStore.cancelTask(task.id)">
                <Icon icon="mdi:close" :width="20" />
              </button>
            </el-tooltip>
          </template>

          <!-- 已完成：打开文件夹 + 删除 -->
          <template v-else-if="task.status === 'completed'">
            <el-tooltip content="打开文件夹" placement="top">
              <button class="action-btn" @click="taskStore.openOutputFolder(task.id)">
                <Icon icon="mdi:folder-open" :width="20" />
              </button>
            </el-tooltip>
            <el-tooltip content="删除" placement="top">
              <button class="action-btn action-btn-danger" @click="taskStore.removeTask(task.id)">
                <Icon icon="mdi:delete-outline" :width="20" />
              </button>
            </el-tooltip>
          </template>

          <!-- 已取消：删除 -->
          <template v-else-if="task.status === 'cancelled'">
            <el-tooltip content="删除" placement="top">
              <button class="action-btn action-btn-danger" @click="taskStore.removeTask(task.id)">
                <Icon icon="mdi:delete-outline" :width="20" />
              </button>
            </el-tooltip>
          </template>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.tasks-view {
  max-width: 960px;
  margin: 0 auto;
}

.page-header {
  display: flex;
  align-items: baseline;
  gap: var(--spacing-md);
  margin-bottom: var(--spacing-lg);
}

.page-header h2 {
  font-size: 22px;
  font-weight: 600;
}

.task-count {
  font-size: 13px;
  color: var(--text-secondary);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: var(--spacing-xl) 0;
  color: var(--text-secondary);
  gap: var(--spacing-sm);
}

.empty-state .sub {
  font-size: 13px;
  color: var(--text-placeholder);
}

.task-list {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.task-card {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  padding: var(--spacing-md);
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  transition: box-shadow var(--transition-fast);
}

.task-card:hover {
  box-shadow: var(--shadow-sm);
}

/* 封面 */
.task-cover {
  width: 120px;
  min-width: 120px;
  aspect-ratio: 16 / 9;
  background: var(--bg-tertiary);
  border-radius: var(--radius-sm);
  overflow: hidden;
  flex-shrink: 0;
}

.task-cover img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.task-cover-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-placeholder);
}

/* 主体 */
.task-body {
  flex: 1;
  min-width: 0;
}

.task-title {
  font-size: 14px;
  font-weight: 600;
  line-height: 1.4;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 4px;
}

.task-meta {
  display: flex;
  flex-wrap: wrap;
  gap: var(--spacing-md);
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: var(--spacing-sm);
}

.meta-up,
.meta-duration,
.meta-quality {
  display: inline-flex;
  align-items: center;
  gap: 2px;
}

/* 进度条 */
.task-progress {
  min-width: 0;
}

.progress-stats {
  display: flex;
  flex-wrap: wrap;
  gap: var(--spacing-md);
  font-size: 11px;
  color: var(--text-placeholder);
  margin-top: 4px;
}

.stat-completed {
  color: var(--color-success);
  font-weight: 500;
}

.stat-error {
  color: var(--color-danger);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 状态角标 */
.task-status-badge {
  font-size: 11px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: var(--radius-full);
  white-space: nowrap;
  flex-shrink: 0;
}

/* 时间 */
.task-time {
  font-size: 11px;
  color: var(--text-placeholder);
  white-space: nowrap;
  flex-shrink: 0;
}

/* 操作按钮 */
.task-actions {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.action-btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: var(--bg-primary);
  color: var(--text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.action-btn:hover {
  background: var(--bg-tertiary);
  color: var(--text-primary);
}

.action-btn-danger:hover {
  background: var(--color-danger);
  color: #fff;
  border-color: var(--color-danger);
}
</style>