<script setup lang="ts">
import { onMounted, onUnmounted, computed } from 'vue'
import { useAppStore } from '@/stores/app'
import { useSettingsStore } from '@/stores/settings'
import { useTaskStore } from '@/stores/tasks'
import { useAuthStore } from '@/stores/auth'
import { useRoute, useRouter } from 'vue-router'
import { Icon } from '@iconify/vue'

const appStore = useAppStore()
const settingsStore = useSettingsStore()
const taskStore = useTaskStore()
const authStore = useAuthStore()
const route = useRoute()
const router = useRouter()

const activeTaskCount = computed(() => taskStore.activeTasks.length)

onMounted(async () => {
  await settingsStore.loadSettings()
  settingsStore.applyTheme(settingsStore.settings.theme)
  taskStore.initEventListeners()
  authStore.checkLoginStatus()
})

onUnmounted(() => {
  taskStore.destroyEventListeners()
})

const navItems = [
  { path: '/', name: 'home', label: '首页', icon: 'mdi:home' },
  { path: '/tasks', name: 'tasks', label: '下载任务', icon: 'mdi:download', badge: activeTaskCount },
  { path: '/account', name: 'account', label: '我的账户', icon: 'mdi:account-circle' },
  { path: '/settings', name: 'settings', label: '设置', icon: 'mdi:cog' },
]

function toggleTheme() {
  const next = settingsStore.settings.theme === 'dark' ? 'light' : 'dark'
  settingsStore.setTheme(next)
}
</script>

<template>
  <div class="app-layout">
    <aside class="sidebar" :class="{ collapsed: appStore.sidebarCollapsed }">
      <div class="sidebar-header">
        <Icon icon="mdi:video-check" :width="24" />
        <span v-show="!appStore.sidebarCollapsed" class="app-title">BiliDownloader</span>
      </div>

      <nav class="sidebar-nav">
        <router-link
          v-for="item in navItems"
          :key="item.name"
          :to="item.path"
          class="nav-item"
          :class="{ active: route.path === item.path }"
        >
          <Icon :icon="item.icon" :width="20" />
          <span v-show="!appStore.sidebarCollapsed" class="nav-label">{{ item.label }}</span>
          <span
            v-if="item.badge && item.badge.value > 0"
            class="nav-badge"
          >{{ item.badge.value }}</span>
        </router-link>
      </nav>

      <div class="sidebar-footer">
        <button class="icon-btn" @click="router.push('/account')" :title="authStore.isLoggedIn ? authStore.uname : '登录B站账号'">
          <Icon
            :icon="authStore.isLoggedIn ? 'mdi:account-check' : 'mdi:account-outline'"
            :width="18"
            :style="{ color: authStore.isLoggedIn ? 'var(--color-success)' : undefined }"
          />
        </button>
        <button class="icon-btn" @click="toggleTheme" :title="settingsStore.settings.theme">
          <Icon
            :icon="settingsStore.settings.theme === 'dark' ? 'mdi:weather-sunny' : 'mdi:weather-night'"
            :width="18"
          />
        </button>
        <button class="icon-btn" @click="appStore.toggleSidebar" title="折叠侧边栏">
          <Icon :icon="appStore.sidebarCollapsed ? 'mdi:chevron-right' : 'mdi:chevron-left'" :width="18" />
        </button>
      </div>
    </aside>

    <main class="main-content">
      <slot />
    </main>
  </div>
</template>

<style scoped>
.app-layout {
  display: flex;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
}

.sidebar {
  display: flex;
  flex-direction: column;
  width: var(--sidebar-width);
  background: var(--bg-secondary);
  border-right: 1px solid var(--border-color);
  transition: width var(--transition-normal);
  flex-shrink: 0;
  padding: var(--spacing-md);
}

.sidebar.collapsed {
  width: 68px;
  padding: var(--spacing-md) var(--spacing-sm);
}

.sidebar.collapsed .sidebar-footer {
  flex-direction: column;
  align-items: center;
  gap: var(--spacing-xs);
}

.sidebar-header {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-sm) 0;
  margin-bottom: var(--spacing-lg);
  color: var(--color-primary);
}

.app-title {
  font-size: 16px;
  font-weight: 700;
  white-space: nowrap;
}

.sidebar-nav {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-xs);
  flex: 1;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-sm) var(--spacing-md);
  border-radius: var(--radius-md);
  color: var(--text-regular);
  transition: all var(--transition-fast);
  white-space: nowrap;
}

.nav-item:hover {
  background: var(--bg-overlay);
  color: var(--color-primary);
}

.nav-item.active {
  background: var(--color-primary);
  color: #fff;
}

.nav-label {
  font-size: 14px;
}

.nav-badge {
  margin-left: auto;
  min-width: 20px;
  height: 20px;
  padding: 0 6px;
  border-radius: 10px;
  background: var(--color-primary);
  color: #fff;
  font-size: 11px;
  font-weight: 600;
  display: flex;
  align-items: center;
  justify-content: center;
  line-height: 1;
}

.sidebar-footer {
  display: flex;
  justify-content: center;
  gap: var(--spacing-sm);
  padding-top: var(--spacing-md);
  border-top: 1px solid var(--border-color-light);
}

.icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border: none;
  border-radius: var(--radius-md);
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.icon-btn:hover {
  background: var(--bg-overlay);
  color: var(--color-primary);
}

.main-content {
  flex: 1;
  overflow: auto;
  padding: var(--spacing-lg);
  background: var(--bg-primary);
}
</style>