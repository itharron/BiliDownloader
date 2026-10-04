<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useSettingsStore } from '@/stores/settings'
import type { ThemeMode } from '@/types'

const settingsStore = useSettingsStore()

const downloadDir = ref('')
const maxConcurrent = ref(3)
const maxRetries = ref(3)
const theme = ref<ThemeMode>('auto')
const httpProxy = ref('')
const showNotification = ref(true)
const autoStart = ref(false)

onMounted(async () => {
  await settingsStore.loadSettings()
  downloadDir.value = settingsStore.settings.downloadDir
  maxConcurrent.value = settingsStore.settings.maxConcurrent
  maxRetries.value = settingsStore.settings.maxRetries
  theme.value = settingsStore.settings.theme
  httpProxy.value = settingsStore.settings.httpProxy || ''
  showNotification.value = settingsStore.settings.showNotification
  autoStart.value = settingsStore.settings.autoStart
})

async function handleSave() {
  settingsStore.settings.downloadDir = downloadDir.value
  settingsStore.settings.maxConcurrent = maxConcurrent.value
  settingsStore.settings.maxRetries = maxRetries.value
  settingsStore.settings.httpProxy = httpProxy.value || null
  settingsStore.settings.showNotification = showNotification.value
  settingsStore.settings.autoStart = autoStart.value
  settingsStore.setTheme(theme.value)
  await settingsStore.persistSettings()
}

async function handleSelectDir() {
  // 使用 Tauri dialog 插件选择目录
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({ directory: true, multiple: false })
    if (selected && typeof selected === 'string') {
      downloadDir.value = selected
    }
  } catch {
    downloadDir.value = ''
  }
}
</script>

<template>
  <div class="settings-view">
    <h2 class="page-title">设置</h2>

    <div class="settings-card">
      <h3>通用</h3>

      <div class="setting-item">
        <label>下载目录</label>
        <div class="dir-row">
          <el-input v-model="downloadDir" placeholder="选择或输入下载目录" readonly>
            <template #suffix>
              <el-button @click="handleSelectDir">浏览</el-button>
            </template>
          </el-input>
        </div>
      </div>

      <div class="setting-item">
        <label>最大并发下载数</label>
        <el-input-number v-model="maxConcurrent" :min="1" :max="10" size="small" />
      </div>

      <div class="setting-item">
        <label>最大重试次数</label>
        <el-input-number v-model="maxRetries" :min="0" :max="10" size="small" />
      </div>
    </div>

    <div class="settings-card">
      <h3>外观</h3>
      <div class="setting-item">
        <label>主题</label>
        <el-radio-group v-model="theme" size="small">
          <el-radio-button value="auto">跟随系统</el-radio-button>
          <el-radio-button value="light">浅色</el-radio-button>
          <el-radio-button value="dark">深色</el-radio-button>
        </el-radio-group>
      </div>
    </div>

    <div class="settings-card">
      <h3>网络</h3>
      <div class="setting-item">
        <label>HTTP 代理</label>
        <el-input v-model="httpProxy" placeholder="例: http://127.0.0.1:7890" size="small" />
      </div>
    </div>

    <div class="settings-card">
      <h3>行为</h3>
      <div class="setting-item">
        <label>下载完成通知</label>
        <el-switch v-model="showNotification" />
      </div>
      <div class="setting-item">
        <label>开机自启</label>
        <el-switch v-model="autoStart" />
      </div>
    </div>

    <div class="save-row">
      <el-button type="primary" @click="handleSave">保存设置</el-button>
    </div>

    <div class="about-card">
      <p>BiliDownloader v0.1.0</p>
      <p class="about-sub">基于 Tauri 2 + Vue 3 + Rust 构建的 B 站视频下载客户端</p>
      <p class="about-warning">仅供个人学习存档使用，请勿传播</p>
    </div>
  </div>
</template>

<style scoped>
.settings-view {
  max-width: 700px;
  margin: 0 auto;
}

.page-title {
  font-size: 22px;
  font-weight: 600;
  margin-bottom: var(--spacing-lg);
}

.settings-card {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  padding: var(--spacing-lg);
  margin-bottom: var(--spacing-md);
}

.settings-card h3 {
  font-size: 15px;
  font-weight: 600;
  margin-bottom: var(--spacing-md);
  color: var(--text-primary);
}

.setting-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-sm) 0;
  gap: var(--spacing-md);
}

.setting-item + .setting-item {
  border-top: 1px solid var(--border-color-light);
}

.setting-item label {
  font-size: 14px;
  color: var(--text-regular);
  white-space: nowrap;
}

.dir-row {
  flex: 1;
  max-width: 400px;
}

.save-row {
  margin-top: var(--spacing-lg);
}

.about-card {
  margin-top: var(--spacing-xl);
  padding: var(--spacing-lg);
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  text-align: center;
  font-size: 13px;
  color: var(--text-secondary);
}

.about-sub {
  margin-top: 4px;
  font-size: 12px;
  color: var(--text-placeholder);
}

.about-warning {
  margin-top: var(--spacing-sm);
  color: var(--color-warning);
  font-size: 12px;
}
</style>