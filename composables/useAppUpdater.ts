import { relaunch } from '@tauri-apps/plugin-process'
import { check } from '@tauri-apps/plugin-updater'

type UpdateStatus = 'idle' | 'checking' | 'downloading' | 'installing'

export function useAppUpdater() {
  const updateStatus = ref<UpdateStatus>('idle')
  const updateVersion = ref('')
  const downloadedBytes = ref(0)
  const totalBytes = ref(0)

  const isUpdateVisible = computed(() =>
    updateStatus.value === 'downloading' || updateStatus.value === 'installing',
  )

  const updateProgress = computed(() => {
    if (updateStatus.value === 'installing') return 100
    if (!totalBytes.value) return 0
    return Math.min(100, Math.round((downloadedBytes.value / totalBytes.value) * 100))
  })

  async function checkForUpdates(beforeInstall?: () => Promise<void>) {
    if (!import.meta.client || !('__TAURI_INTERNALS__' in window)) return

    updateStatus.value = 'checking'

    try {
      const update = await check()
      if (!update) {
        updateStatus.value = 'idle'
        return
      }

      updateVersion.value = update.version
      updateStatus.value = 'downloading'
      await beforeInstall?.()

      await update.downloadAndInstall((event) => {
        if (event.event === 'Started') {
          totalBytes.value = event.data.contentLength ?? 0
          downloadedBytes.value = 0
        } else if (event.event === 'Progress') {
          downloadedBytes.value += event.data.chunkLength
        } else if (event.event === 'Finished') {
          updateStatus.value = 'installing'
        }
      })

      updateStatus.value = 'installing'
      await relaunch()
    } catch (error) {
      console.warn('KeyHold update check failed:', error)
      updateStatus.value = 'idle'
    }
  }

  return {
    updateStatus,
    updateVersion,
    updateProgress,
    isUpdateVisible,
    checkForUpdates,
  }
}
