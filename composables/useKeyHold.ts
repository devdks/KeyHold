import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

interface KeySelection {
  key: string
  code: string
  label: string
}

const DEFAULT_KEY: KeySelection = { key: ' ', code: 'Space', label: 'Espace' }

function displayKey(event: KeyboardEvent): string {
  const labels: Record<string, string> = {
    ' ': 'Espace',
    ArrowUp: '↑',
    ArrowDown: '↓',
    ArrowLeft: '←',
    ArrowRight: '→',
    Control: 'Ctrl',
    Escape: 'Échap',
    Backspace: 'Retour',
    CapsLock: 'Verr. Maj',
    Enter: 'Entrée',
  }

  const knownLabel = labels[event.key]
  if (knownLabel) return knownLabel
  if (event.key === 'Meta') return navigator.platform.includes('Mac') ? '⌘' : 'Win'
  if (event.key.length === 1) return event.key.toLocaleUpperCase()
  return event.key.replace(/^Key|^Digit/, '')
}

function isModifierOnly(event: KeyboardEvent): boolean {
  return ['AltGraph'].includes(event.key)
}

export function useKeyHold() {
  const selectedKey = ref<KeySelection>({ ...DEFAULT_KEY })
  const isCapturing = ref(false)
  const isHolding = ref(false)
  const isCompact = ref(false)
  const timerEnabled = ref(false)
  const minutes = ref(5)
  const seconds = ref(0)
  const remainingSeconds = ref(0)
  const errorMessage = ref('')
  let timerId: ReturnType<typeof setInterval> | undefined
  let unlisten: UnlistenFn | undefined

  const isDesktop = () => import.meta.client && '__TAURI_INTERNALS__' in window

  const remainingLabel = computed(() => {
    const mins = Math.floor(remainingSeconds.value / 60)
    const secs = remainingSeconds.value % 60
    return `${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}`
  })

  const statusLabel = computed(() => {
    if (errorMessage.value) return errorMessage.value
    if (isCapturing.value) return 'En attente d’une touche'
    if (isHolding.value) return 'Touche maintenue'
    return 'Prêt'
  })

  function persist() {
    localStorage.setItem('keyhold:settings', JSON.stringify({
      selectedKey: selectedKey.value,
      timerEnabled: timerEnabled.value,
      minutes: minutes.value,
      seconds: seconds.value,
    }))
  }

  function restore() {
    const saved = localStorage.getItem('keyhold:settings')
    if (!saved) return

    try {
      const settings = JSON.parse(saved)
      if (settings.selectedKey?.key && settings.selectedKey?.code) selectedKey.value = settings.selectedKey
      timerEnabled.value = Boolean(settings.timerEnabled)
      minutes.value = Math.max(0, Number(settings.minutes) || 0)
      seconds.value = Math.min(59, Math.max(0, Number(settings.seconds) || 0))
    } catch {
      localStorage.removeItem('keyhold:settings')
    }
  }

  function onKeyCaptured(event: KeyboardEvent) {
    if (!isCapturing.value || isModifierOnly(event)) return
    event.preventDefault()
    event.stopPropagation()
    selectedKey.value = { key: event.key, code: event.code, label: displayKey(event) }
    isCapturing.value = false
    window.removeEventListener('keydown', onKeyCaptured, true)
    persist()
  }

  function startCapture() {
    errorMessage.value = ''
    isCapturing.value = true
    window.addEventListener('keydown', onKeyCaptured, true)
  }

  function clearTimer() {
    if (timerId) clearInterval(timerId)
    timerId = undefined
  }

  async function setCompactMode(compact: boolean) {
    isCompact.value = compact
    if (!isDesktop()) return

    try {
      await invoke('set_compact_mode', { compact })
    } catch (error) {
      isCompact.value = false
      errorMessage.value = String(error)
    }
  }

  function exitCompactMode() {
    return setCompactMode(false)
  }

  async function stopHold() {
    clearTimer()
    if (isDesktop()) {
      try {
        await invoke('release_key')
      } catch (error) {
        errorMessage.value = String(error)
      }
    }
    isHolding.value = false
    remainingSeconds.value = 0
    await exitCompactMode()
  }

  async function startHold() {
    errorMessage.value = ''
    const duration = Math.max(0, minutes.value * 60 + seconds.value)
    if (timerEnabled.value && duration === 0) {
      errorMessage.value = 'Choisis une durée'
      return
    }

    try {
      if (isDesktop()) {
        await invoke('hold_key', {
          key: selectedKey.value,
          durationSeconds: timerEnabled.value ? duration : null,
        })
      }
      isHolding.value = true
      persist()
      await setCompactMode(true)

      if (timerEnabled.value) {
        remainingSeconds.value = duration
        timerId = setInterval(() => {
          remainingSeconds.value -= 1
          if (remainingSeconds.value <= 0) void stopHold()
        }, 1000)
      }
    } catch (error) {
      errorMessage.value = String(error)
      isHolding.value = false
    }
  }

  function toggleHold() {
    return isHolding.value ? stopHold() : startHold()
  }

  onMounted(async () => {
    restore()
    if (isDesktop()) {
      unlisten = await listen('keyhold://released', () => {
        clearTimer()
        isHolding.value = false
        remainingSeconds.value = 0
        void exitCompactMode()
      })
    }
  })

  onBeforeUnmount(() => {
    window.removeEventListener('keydown', onKeyCaptured, true)
    clearTimer()
    unlisten?.()
    if (isHolding.value && isDesktop()) void invoke('release_key')
  })

  return {
    selectedKey,
    isCapturing,
    isHolding,
    isCompact,
    timerEnabled,
    minutes,
    seconds,
    remainingLabel,
    statusLabel,
    startCapture,
    toggleHold,
    exitCompactMode,
  }
}
