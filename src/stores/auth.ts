import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { BiliLoginInfo } from '@/types'
import { getLoginStatus } from '@/services/tauri'

export const useAuthStore = defineStore('auth', () => {
  const loginInfo = ref<BiliLoginInfo | null>(null)
  const loaded = ref(false)

  const isLoggedIn = computed(() => loginInfo.value !== null)
  const uname = computed(() => loginInfo.value?.uname ?? '')
  const avatar = computed(() => loginInfo.value?.avatar ?? '')

  async function checkLoginStatus() {
    try {
      loginInfo.value = await getLoginStatus()
    } catch {
      loginInfo.value = null
    } finally {
      loaded.value = true
    }
  }

  function setLoginInfo(info: BiliLoginInfo) {
    loginInfo.value = info
  }

  function clearLogin() {
    loginInfo.value = null
  }

  return { loginInfo, loaded, isLoggedIn, uname, avatar, checkLoginStatus, setLoginInfo, clearLogin }
})