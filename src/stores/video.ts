import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { VideoInfo } from '@/types'

export const useVideoStore = defineStore('video', () => {
  const currentVideo = ref<VideoInfo | null>(null)
  const currentUrl = ref('')
  const currentQuality = ref(0)
  const parsedWithoutLogin = ref(false)

  function setVideo(url: string, video: VideoInfo) {
    currentUrl.value = url
    currentVideo.value = video
    if (video.qualities.length > 0) {
      currentQuality.value = video.qualities[0].id
    }
  }

  function clearVideo() {
    currentVideo.value = null
    currentUrl.value = ''
    currentQuality.value = 0
    parsedWithoutLogin.value = false
  }

  return { currentVideo, currentUrl, currentQuality, parsedWithoutLogin, setVideo, clearVideo }
})