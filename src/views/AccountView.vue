<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useAuthStore } from '@/stores/auth'
import { logout as doLogout, getDataDir } from '@/services/tauri'
import { Icon } from '@iconify/vue'
import LoginDialog from '@/components/auth/LoginDialog.vue'

const authStore = useAuthStore()
const showLoginDialog = ref(false)
const showCredentials = ref(false)
const dataDir = ref('')

onMounted(async () => {
  try {
    dataDir.value = await getDataDir()
  } catch {
    dataDir.value = '无法获取'
  }
})

async function handleLogout() {
  try {
    await ElMessageBox.confirm('确定要退出登录吗？退出后高清晰度视频可能无法下载。', '确认退出', {
      confirmButtonText: '退出',
      cancelButtonText: '取消',
      type: 'warning',
    })
  } catch {
    return
  }

  try {
    await doLogout()
    authStore.clearLogin()
    ElMessage.success('已退出登录')
  } catch (e: any) {
    ElMessage.error(`退出失败: ${e}`)
  }
}
</script>

<template>
  <div class="account-view">
    <h1 class="page-title">我的账户</h1>

    <!-- 已登录状态 -->
    <section v-if="authStore.isLoggedIn" class="account-card">
      <div class="account-header">
        <div class="avatar-wrapper">
          <img
            v-if="authStore.loginInfo?.avatar"
            :src="authStore.loginInfo.avatar"
            :alt="authStore.loginInfo.uname"
            class="avatar-img"
            referrerpolicy="no-referrer"
          />
          <div v-else class="avatar-placeholder">
            <Icon icon="mdi:account" :width="40" />
          </div>
        </div>

        <div class="user-summary">
          <h2 class="user-name">{{ authStore.loginInfo?.uname }}</h2>
          <span class="user-uid">UID: {{ authStore.loginInfo?.dedeuserid }}</span>
          <div class="status-row">
            <el-tag type="success" size="small">已登录</el-tag>
          </div>
        </div>

        <div class="header-actions">
          <el-button type="danger" plain size="small" @click="handleLogout">
            <Icon icon="mdi:logout" :width="16" />
            退出登录
          </el-button>
        </div>
      </div>

      <div class="credentials-toggle" @click="showCredentials = !showCredentials">
        <Icon :icon="showCredentials ? 'mdi:chevron-up' : 'mdi:chevron-down'" :width="16" />
        <span>登录凭证</span>
        <span class="toggle-hint">{{ showCredentials ? '收起' : '展开' }}</span>
      </div>

      <div v-if="showCredentials" class="credentials-panel">
        <div class="cred-item">
          <span class="cred-label">SESSDATA</span>
          <code class="cred-value">{{ authStore.loginInfo?.sessdata }}</code>
        </div>
        <div class="cred-item">
          <span class="cred-label">bili_jct</span>
          <code class="cred-value">{{ authStore.loginInfo?.biliJct }}</code>
        </div>
        <div class="cred-item">
          <span class="cred-label">DedeUserID</span>
          <code class="cred-value">{{ authStore.loginInfo?.dedeuserid }}</code>
        </div>
        <div class="storage-hint">
          <Icon icon="mdi:information-outline" :width="14" />
          <span>登录凭证存储在本地 <code>{{ dataDir }}\settings.json</code>，仅用于请求 B站 API 获取高清晰度视频流，不会上传到任何服务器。</span>
        </div>
      </div>
    </section>

    <!-- 未登录状态 -->
    <section v-else class="account-card not-logged-in">
      <div class="empty-state">
        <Icon icon="mdi:account-off-outline" :width="64" class="empty-icon" />
        <h2>暂未登录</h2>
        <p class="empty-desc">
          登录 B站账号后可下载 1080P+ 高清晰度视频（大会员专属清晰度）
        </p>
        <el-button type="primary" size="large" @click="showLoginDialog = true">
          <Icon icon="mdi:login" :width="18" />
          扫码登录
        </el-button>
      </div>
    </section>

    <LoginDialog v-model:visible="showLoginDialog" />
  </div>
</template>

<style scoped>
.account-view {
  padding: var(--spacing-xl);
  max-width: 640px;
  margin: 0 auto;
}

.page-title {
  font-size: 20px;
  font-weight: 700;
  margin-bottom: var(--spacing-lg);
  color: var(--text-primary);
}

/* ---------- 已登录卡片 ---------- */
.account-card {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  padding: var(--spacing-xl);
}

.account-header {
  display: flex;
  align-items: center;
  gap: var(--spacing-lg);
}

.avatar-wrapper {
  flex-shrink: 0;
}

.avatar-img {
  width: 72px;
  height: 72px;
  border-radius: 50%;
  object-fit: cover;
  border: 3px solid var(--color-primary);
}

.avatar-placeholder {
  width: 72px;
  height: 72px;
  border-radius: 50%;
  background: var(--bg-tertiary);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
}

.user-summary {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex: 1;
}

.user-name {
  font-size: 20px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0;
}

.user-uid {
  font-size: 13px;
  color: var(--text-secondary);
}

.status-row {
  margin-top: 4px;
}

.header-actions {
  flex-shrink: 0;
  align-self: flex-start;
}

/* ---------- 登录凭证折叠面板 ---------- */
.credentials-toggle {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: var(--spacing-lg);
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--bg-tertiary);
  border-radius: var(--radius-md);
  cursor: pointer;
  user-select: none;
  font-size: 13px;
  color: var(--text-secondary);
  transition: background var(--transition-fast);
}

.credentials-toggle:hover {
  background: var(--border-color-light);
}

.toggle-hint {
  margin-left: auto;
  font-size: 12px;
  opacity: 0.7;
}

.credentials-panel {
  margin-top: var(--spacing-sm);
  padding: var(--spacing-md);
  background: var(--bg-tertiary);
  border-radius: var(--radius-md);
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.cred-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.cred-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.cred-value {
  font-size: 12px;
  color: var(--text-primary);
  font-family: 'Cascadia Code', 'Fira Code', 'Consolas', monospace;
  background: var(--bg-secondary);
  padding: 4px 8px;
  border-radius: 4px;
  word-break: break-all;
  line-height: 1.5;
}

.storage-hint {
  display: flex;
  align-items: flex-start;
  gap: var(--spacing-sm);
  margin-top: var(--spacing-sm);
  padding-top: var(--spacing-sm);
  border-top: 1px solid var(--border-color-light);
  font-size: 12px;
  color: var(--text-secondary);
  line-height: 1.6;
}

.storage-hint code {
  background: var(--bg-secondary);
  padding: 1px 4px;
  border-radius: 3px;
  font-size: 11px;
}

.storage-hint :deep(.iconify) {
  flex-shrink: 0;
  margin-top: 1px;
  color: var(--color-primary);
}

/* ---------- 未登录空状态 ---------- */
.not-logged-in {
  text-align: center;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--spacing-sm);
  padding: var(--spacing-xl) 0;
}

.empty-icon {
  color: var(--text-secondary);
}

.empty-state h2 {
  font-size: 18px;
  color: var(--text-primary);
  margin: 0;
}

.empty-desc {
  font-size: 13px;
  color: var(--text-secondary);
  max-width: 300px;
  line-height: 1.6;
  margin-bottom: var(--spacing-sm);
}
</style>