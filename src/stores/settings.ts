import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { AppSettings, ThemeMode } from '@/types'
import { getSettings, saveSettings } from '@/services/tauri'

export const useSettingsStore = defineStore('settings', () => {
  const settings = ref<AppSettings>({
    downloadDir: '',
    maxConcurrent: 3,
    maxRetries: 3,
    theme: 'auto',
    locale: 'zh-CN',
    httpProxy: null,
    cookieSessdata: null,
    cookieBiliJct: null,
    cookieDedeuserid: null,
    loginUname: null,
    loginAvatar: null,
    filenameTemplate: '[%(uploader)s] %(title)s.%(ext)s',
    autoStart: false,
    minimizeToTray: true,
    showNotification: true,
  })

  async function loadSettings() {
    try {
      const s = await getSettings()
      settings.value = s
    } catch {
      // 使用默认设置
    }
  }

  async function persistSettings() {
    try {
      await saveSettings(settings.value)
    } catch (e) {
      console.error('保存设置失败:', e)
    }
  }

  function setTheme(mode: ThemeMode) {
    settings.value.theme = mode
    applyTheme(mode)
    persistSettings()
  }

  function applyTheme(mode: ThemeMode) {
    const html = document.documentElement
    if (mode === 'dark') {
      html.classList.add('dark')
    } else if (mode === 'light') {
      html.classList.remove('dark')
    } else {
      // auto: 跟随系统
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
      html.classList.toggle('dark', prefersDark)
    }
  }

  return {
    settings,
    loadSettings,
    persistSettings,
    setTheme,
    applyTheme,
  }
})