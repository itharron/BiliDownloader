<script setup lang="ts">
import { ref, computed, onUnmounted, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { generateQr, pollQr, confirmLogin, logout } from '@/services/tauri'
import { useAuthStore } from '@/stores/auth'
import type { QrStatus } from '@/types'

const props = defineProps<{ visible: boolean }>()
const emit = defineEmits<{ 'update:visible': [value: boolean] }>()

const authStore = useAuthStore()

const qrUrl = ref('')
const qrcodeKey = ref('')
const qrStatus = ref<QrStatus>('pending')
const statusMessage = ref('请使用 B站客户端 扫描二维码')
const loading = ref(false)

let pollTimer: ReturnType<typeof setTimeout> | null = null
let pollErrorCount = 0
const MAX_POLL_ERRORS = 10

const visible = computed({
  get: () => props.visible,
  set: (v) => emit('update:visible', v),
})

onUnmounted(() => stopPolling())

function stopPolling() {
  if (pollTimer) {
    clearTimeout(pollTimer)
    pollTimer = null
  }
  pollErrorCount = 0
}

async function handleGenerate() {
  stopPolling()
  loading.value = true
  qrStatus.value = 'pending'
  statusMessage.value = '正在生成二维码...'

  try {
    const result = await generateQr()
    qrUrl.value = result.qrUrl
    qrcodeKey.value = result.qrcodeKey
    statusMessage.value = '请使用 B站客户端 扫描二维码'
    // 先等 10 秒再开始轮询
    startPolling(10_000)
  } catch (e: any) {
    statusMessage.value = `生成失败: ${e}`
  } finally {
    loading.value = false
  }
}

function startPolling(firstDelay = 2_000) {
  stopPolling()
  pollTimer = setTimeout(async function poll() {
    if (!qrStatus.value || qrStatus.value === 'expired' || qrStatus.value === 'confirmed') return
    if (!qrcodeKey.value) return
    try {
      const result = await pollQr(qrcodeKey.value)
      qrStatus.value = result.status

      if (result.status === 'scanned') {
        statusMessage.value = '已扫码，请在手机上确认登录'
      } else if (result.status === 'confirmed') {
        stopPolling()
        statusMessage.value = '登录成功！'
        if (result.sessdata) {
          try {
            const info = await confirmLogin(
              result.sessdata,
              result.biliJct ?? '',
              result.dedeuserid ?? ''
            )
            authStore.setLoginInfo(info)
            ElMessage.success(`已登录: ${info.uname}`)
            visible.value = false
          } catch (e: any) {
            ElMessage.error(`登录确认失败: ${e}`)
            statusMessage.value = '登录确认失败，请重试'
            qrStatus.value = 'expired'
          }
        } else {
          ElMessage.error('登录未返回有效的 Cookie 信息')
          statusMessage.value = '登录失败，请重试'
          qrStatus.value = 'expired'
        }
      } else if (result.status === 'expired') {
        stopPolling()
        statusMessage.value = '二维码已过期，请重新生成'
      }
    } catch (e: any) {
      pollErrorCount++
      console.error(`[LoginDialog] poll error (${pollErrorCount}/${MAX_POLL_ERRORS}):`, e)
      if (pollErrorCount >= MAX_POLL_ERRORS) {
        stopPolling()
        statusMessage.value = '轮询失败，请检查网络后重试'
        qrStatus.value = 'expired'
        return
      }
    }
    // 递归调度下一次轮询（2 秒间隔）
    if (!pollTimer) return
    pollTimer = setTimeout(poll, 2_000)
  }, firstDelay)
}

async function handleLogout() {
  try {
    await logout()
    authStore.clearLogin()
    ElMessage.success('已退出登录')
    visible.value = false
  } catch (e: any) {
    ElMessage.error(`退出失败: ${e}`)
  }
}

watch(visible, (v) => {
  if (v) {
    handleGenerate()
  } else {
    stopPolling()
  }
})
</script>

<template>
  <el-dialog
    v-model="visible"
    title="B站账号登录"
    :width="380"
    :close-on-click-modal="false"
    destroy-on-close
  >
    <div class="login-dialog">
      <!-- 已登录状态 -->
      <template v-if="authStore.isLoggedIn">
        <div class="logged-in">
          <img
            v-if="authStore.avatar"
            :src="authStore.avatar"
            class="avatar"
            referrerpolicy="no-referrer"
          />
          <p class="uname">{{ authStore.uname }}</p>
          <p class="login-tip">已登录 B站账号，可下载高清晰度视频</p>
          <el-button type="danger" plain @click="handleLogout">退出登录</el-button>
        </div>
      </template>

      <!-- 扫码登录 -->
      <template v-else>
        <div class="qr-container">
          <div class="qr-wrapper" v-loading="loading">
            <img
              v-if="qrUrl"
              :src="`https://api.qrserver.com/v1/create-qr-code/?size=200x200&data=${encodeURIComponent(qrUrl)}`"
              alt="登录二维码"
              class="qr-image"
              :class="{ 'qr-expired': qrStatus === 'expired' }"
            />
            <div v-else class="qr-placeholder">
              <p>点击生成二维码</p>
            </div>
          </div>

          <p class="status-text" :class="`status-${qrStatus}`">
            {{ statusMessage }}
          </p>

          <el-button
            v-if="qrStatus === 'expired'"
            type="primary"
            size="small"
            @click="handleGenerate"
          >
            重新生成
          </el-button>
        </div>
      </template>
    </div>
  </el-dialog>
</template>

<style scoped>
.login-dialog {
  text-align: center;
  padding: var(--spacing-md) 0;
}

.logged-in {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--spacing-sm);
}

.avatar {
  width: 64px;
  height: 64px;
  border-radius: 50%;
  object-fit: cover;
  border: 2px solid var(--color-primary);
}

.uname {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

.login-tip {
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: var(--spacing-sm);
}

.qr-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--spacing-md);
}

.qr-wrapper {
  width: 200px;
  height: 200px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.qr-image {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.qr-image.qr-expired {
  opacity: 0.3;
  filter: grayscale(100%);
}

.qr-placeholder {
  color: var(--text-secondary);
  font-size: 14px;
}

.status-text {
  font-size: 13px;
  color: var(--text-secondary);
}

.status-scanned {
  color: var(--color-primary);
}

.status-confirmed {
  color: var(--color-success);
}

.status-expired {
  color: var(--color-danger);
}
</style>