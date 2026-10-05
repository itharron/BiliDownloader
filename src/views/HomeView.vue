<script setup lang="ts">
import { ref, onMounted, watch, computed } from 'vue'
import { useRouter } from 'vue-router'
import { parseVideoUrl, startDownload } from '@/services/tauri'
import { useTaskStore } from '@/stores/tasks'
import { useVideoStore } from '@/stores/video'
import { useAuthStore } from '@/stores/auth'
import { QUALITY_MAP } from '@/types'
import { ElMessage, ElNotification } from 'element-plus'
import LoginDialog from '@/components/auth/LoginDialog.vue'

const url = ref('')
const loading = ref(false)
const error = ref('')
const showLoginTip = ref(false)
const showLoginDialog = ref(false)
const audioOnly = ref(false)
const downloadDanmaku = ref(false)
const downloadSubtitle = ref(false)
const downloadCover = ref(false)
const downloadDescription = ref(false)

const router = useRouter()
const taskStore = useTaskStore()
const videoStore = useVideoStore()
const authStore = useAuthStore()

const isLoggedIn = computed(() => authStore.isLoggedIn)

onMounted(async () => {
  await authStore.checkLoginStatus()
  if (videoStore.currentVideo) {
    url.value = videoStore.currentUrl
    // 如果已登录，且上次解析时未登录，自动重新解析以获取高清选项
    if (authStore.isLoggedIn && videoStore.parsedWithoutLogin) {
      videoStore.parsedWithoutLogin = false
      handleParse()
    }
  }
})

// 登录状态变化时自动重新解析（获取更高清选项）
watch(isLoggedIn, (newVal, oldVal) => {
  if (newVal && !oldVal && videoStore.currentVideo) {
    videoStore.parsedWithoutLogin = false
    handleParse()
    ElMessage.success('登录成功！正在重新获取高清选项...')
  }
})

async function handleParse() {
  if (!url.value.trim()) return
  error.value = ''
  loading.value = true
  showLoginTip.value = false

  try {
    const sessdata = authStore.loginInfo?.sessdata
    if (!sessdata) {
      videoStore.parsedWithoutLogin = true
    }
    const videoInfo = await parseVideoUrl(url.value.trim(), sessdata)
    videoStore.setVideo(url.value.trim(), videoInfo)

    if (videoInfo.qualities.length === 0) {
      showLoginTip.value = true
      error.value = ''
    }
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : e.message || '解析失败'
  } finally {
    loading.value = false
  }
}

async function handleStartDownload() {
  const video = videoStore.currentVideo
  const quality = videoStore.currentQuality
  if (!video || !quality) return

  try {
    const selectedQuality = video.qualities.find(q => q.id === quality)
    const task = await startDownload({
      url: videoStore.currentUrl,
      qualityId: quality,
      audioOnly: audioOnly.value,
      downloadDanmaku: downloadDanmaku.value,
      downloadSubtitle: downloadSubtitle.value,
      downloadCover: downloadCover.value,
      downloadDescription: downloadDescription.value,
      title: video.title,
      coverUrl: video.coverUrl,
      upName: video.upName,
      durationText: video.durationText,
      qualityName: selectedQuality?.description || QUALITY_MAP[quality] || String(quality),
    })
    taskStore.addTask(task)
    ElNotification({
      title: '已添加到下载队列',
      message: `开始下载 ${task.title || task.url}`,
      type: 'success',
      duration: 3000,
    })
    router.push('/tasks')
  } catch (e: any) {
    ElMessage.error(typeof e === 'string' ? e : e.message || '添加下载失败')
  }
}
</script>

<template>
  <div class="home-view">
    <header class="hero">
      <h1 class="hero-title">B 站视频下载器</h1>
      <p class="hero-desc">输入 BV 号或视频链接，一键解析下载</p>
    </header>

    <section class="url-section">
      <div class="url-input-row">
        <el-input
          v-model="url"
          size="large"
          placeholder="粘贴 B 站视频链接，如 https://www.bilibili.com/video/BV1xx..."
          clearable
          @keyup.enter="handleParse"
        >
          <template #prefix>
            <span style="color: var(--text-secondary)">🔗</span>
          </template>
        </el-input>
        <el-button
          type="primary"
          size="large"
          :loading="loading"
          @click="handleParse"
        >
          解析视频
        </el-button>
      </div>

      <p v-if="error" class="error-msg">{{ error }}</p>
      <div v-if="!isLoggedIn && !loading" class="login-tip">
        <span>💡 登录 B站 账号后可以查看和下载更高清的视频</span>
        <el-button type="primary" size="small" @click="showLoginDialog = true">扫码登录</el-button>
      </div>
    </section>

    <section v-if="videoStore.currentVideo" class="video-info-card">
      <div class="card-cover">
        <img
          v-if="videoStore.currentVideo.coverUrl"
          :src="videoStore.currentVideo.coverUrl"
          :alt="videoStore.currentVideo.title"
          referrerpolicy="no-referrer"
        />
      </div>
      <div class="card-body">
        <h2 class="video-title">{{ videoStore.currentVideo.title }}</h2>
        <div class="video-meta">
          <span>UP 主: {{ videoStore.currentVideo.upName }}</span>
          <span>时长: {{ videoStore.currentVideo.durationText }}</span>
          <span>播放: {{ videoStore.currentVideo.viewCount.toLocaleString() }}</span>
          <span>弹幕: {{ videoStore.currentVideo.danmakuCount.toLocaleString() }}</span>
        </div>

        <div class="quality-section">
          <label class="section-label">选择清晰度</label>
          <div v-if="showLoginTip" class="no-quality-tip">
            ⚠️ 未获取到可选清晰度，请登录 B站 账号后重新解析以获取高清选项
            <div style="margin-top: 8px;">
              <el-button type="primary" size="small" @click="showLoginDialog = true">扫码登录</el-button>
            </div>
          </div>
          <div v-else class="quality-list">
            <button
              v-for="q in videoStore.currentVideo.qualities"
              :key="q.id"
              class="quality-btn"
              :class="{ active: videoStore.currentQuality === q.id }"
              @click="videoStore.currentQuality = q.id"
            >
              {{ q.description }}<br />
              <small>{{ QUALITY_MAP[q.id] || q.name }}</small>
            </button>
          </div>
        </div>

        <div class="action-row">
          <el-button type="primary" size="large" @click="handleStartDownload">
            下载选中清晰度
          </el-button>
          <el-button size="large" :loading="loading" @click="handleParse">
            重新解析
          </el-button>
          <el-checkbox v-model="audioOnly" label="仅下载音频 (MP3)" />
          <el-checkbox v-model="downloadDescription" label="下载简介" />
          <el-checkbox v-model="downloadDanmaku" label="下载弹幕" />
          <el-checkbox v-model="downloadCover" label="下载封面" />
          <el-checkbox v-model="downloadSubtitle" label="下载字幕" />
        </div>

        <p class="disclaimer">
          ⚠️ 仅供个人学习存档使用，请勿传播。请遵守版权法规与 Bilibili 用户协议。
        </p>
      </div>
    </section>

    <LoginDialog v-model:visible="showLoginDialog" />
  </div>
</template>

<style scoped>
.home-view {
  width: min(94%, 2025px);
  margin: 0 auto;
}

.hero {
  text-align: center;
  padding: clamp(24px, 4vw, 48px) 0 clamp(16px, 3vw, 32px);
}

.hero-title {
  font-size: clamp(22px, 4vw, 32px);
  font-weight: 700;
  color: var(--text-primary);
  margin-bottom: var(--spacing-sm);
}

.hero-desc {
  font-size: clamp(13px, 2vw, 16px);
  color: var(--text-secondary);
}

.url-section {
  margin-bottom: var(--spacing-lg);
}

.url-input-row {
  display: flex;
  gap: clamp(6px, 1vw, 12px);
}

.url-input-row .el-input {
  flex: 1;
}

.error-msg {
  margin-top: var(--spacing-sm);
  color: var(--color-danger);
  font-size: 13px;
}

.login-tip {
  margin-top: var(--spacing-sm);
  padding: var(--spacing-sm) var(--spacing-md);
  background: var(--bg-tertiary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  font-size: 13px;
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--spacing-md);
}

.no-quality-tip {
  padding: var(--spacing-md);
  background: var(--bg-tertiary);
  border: 1px dashed var(--border-color);
  border-radius: var(--radius-md);
  font-size: 13px;
  color: var(--text-secondary);
  text-align: center;
}

.no-quality-tip a {
  color: var(--color-primary);
  text-decoration: none;
}

.no-quality-tip a:hover {
  text-decoration: underline;
}

.video-info-card {
  display: flex;
  flex-direction: column;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  overflow: hidden;
  box-shadow: var(--shadow-md);
}

.card-cover {
  width: 100%;
  aspect-ratio: 16 / 9;
  flex-shrink: 0;
  background: var(--bg-tertiary);
  border-radius: var(--radius-md);
  overflow: hidden;
}

.card-cover img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.card-body {
  padding: clamp(12px, 2vw, 24px);
  display: flex;
  flex-direction: column;
  gap: clamp(8px, 1.5vw, 16px);
}

.video-title {
  font-size: clamp(15px, 2.2vw, 20px);
  font-weight: 600;
  color: var(--text-primary);
  line-height: 1.5;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.video-meta {
  display: flex;
  flex-wrap: wrap;
  gap: clamp(8px, 1.5vw, 16px);
  font-size: clamp(12px, 1.5vw, 14px);
  color: var(--text-secondary);
}

.section-label {
  font-size: clamp(13px, 1.6vw, 15px);
  font-weight: 500;
  color: var(--text-primary);
  margin-bottom: var(--spacing-sm);
  display: block;
}

.quality-list {
  display: flex;
  flex-wrap: wrap;
  gap: clamp(4px, 0.8vw, 8px);
}

.quality-btn {
  padding: clamp(6px, 1vw, 10px) clamp(10px, 1.5vw, 16px);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  background: var(--bg-primary);
  color: var(--text-primary);
  cursor: pointer;
  transition: all var(--transition-fast);
  font-size: clamp(11px, 1.4vw, 14px);
  text-align: center;
  min-width: clamp(70px, 10vw, 100px);
}

.quality-btn:hover {
  border-color: var(--color-primary);
}

.quality-btn.active {
  background: var(--color-primary);
  border-color: var(--color-primary);
  color: #fff;
}

.quality-btn small {
  opacity: 0.8;
}

.action-row {
  display: flex;
  align-items: center;
  gap: clamp(6px, 1vw, 12px);
  flex-wrap: wrap;
}

.disclaimer {
  font-size: clamp(11px, 1.2vw, 13px);
  color: var(--text-placeholder);
  border-top: 1px solid var(--border-color-light);
  padding-top: var(--spacing-sm);
}

@media (min-width: 800px) {
  .video-info-card {
    flex-direction: row;
    align-items: flex-start;
  }
  .card-cover {
    width: clamp(180px, 25%, 360px);
    flex-shrink: 0;
  }
  .card-body {
    flex: 1;
  }
}
</style>