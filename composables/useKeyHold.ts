import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface KeySelection {
  key: string;
  code: string;
  label: string;
}

const DEFAULT_KEYS: KeySelection[] = [
  { key: " ", code: "Space", label: "Espace" },
];
const MAX_KEYS = 8;

function displayKey(event: KeyboardEvent): string {
  const labels: Record<string, string> = {
    " ": "Espace",
    ArrowUp: "↑",
    ArrowDown: "↓",
    ArrowLeft: "←",
    ArrowRight: "→",
    Control: "Ctrl",
    Escape: "Échap",
    Backspace: "Retour",
    CapsLock: "Verr. Maj",
    Enter: "Entrée",
    AltGraph: "AltGr",
  };

  const knownLabel = labels[event.key];
  if (knownLabel) return knownLabel;
  if (event.key === "Meta")
    return navigator.platform.includes("Mac") ? "⌘" : "Win";
  if (event.key.length === 1) return event.key.toLocaleUpperCase();
  return event.key.replace(/^Key|^Digit/, "");
}

function selectionFromEvent(event: KeyboardEvent): KeySelection {
  return { key: event.key, code: event.code, label: displayKey(event) };
}

export function useKeyHold() {
  const selectedKeys = ref<KeySelection[]>(
    DEFAULT_KEYS.map((key) => ({ ...key })),
  );
  const pendingKeys = ref<KeySelection[]>([]);
  const isCapturing = ref(false);
  const isHolding = ref(false);
  const isCompact = ref(false);
  const timerEnabled = ref(false);
  const minutes = ref(5);
  const seconds = ref(0);
  const remainingSeconds = ref(0);
  const errorMessage = ref("");
  const pressedCodes = new Set<string>();
  let timerId: ReturnType<typeof setInterval> | undefined;
  let unlisten: UnlistenFn | undefined;

  const isDesktop = () => import.meta.client && "__TAURI_INTERNALS__" in window;
  const displayedKeys = computed(() =>
    isCapturing.value ? pendingKeys.value : selectedKeys.value,
  );
  const selectedKeysLabel = computed(() =>
    selectedKeys.value.map((key) => key.label).join(" + "),
  );

  const remainingLabel = computed(() => {
    const mins = Math.floor(remainingSeconds.value / 60);
    const secs = remainingSeconds.value % 60;
    return `${String(mins).padStart(2, "0")}:${String(secs).padStart(2, "0")}`;
  });

  const statusLabel = computed(() => {
    if (errorMessage.value) return errorMessage.value;
    if (isCapturing.value) {
      const count = pendingKeys.value.length;
      return count
        ? `${count} touche${count > 1 ? "s" : ""} détectée${count > 1 ? "s" : ""}`
        : "En attente des touches";
    }
    if (isHolding.value)
      return selectedKeys.value.length > 1
        ? "Touches maintenues"
        : "Touche maintenue";
    return "Prêt";
  });

  function persist() {
    localStorage.setItem(
      "keyhold:settings",
      JSON.stringify({
        selectedKeys: selectedKeys.value,
        timerEnabled: timerEnabled.value,
        minutes: minutes.value,
        seconds: seconds.value,
      }),
    );
  }

  function restore() {
    const saved = localStorage.getItem("keyhold:settings");
    if (!saved) return;

    try {
      const settings = JSON.parse(saved);
      const restoredKeys = Array.isArray(settings.selectedKeys)
        ? settings.selectedKeys
        : settings.selectedKey
          ? [settings.selectedKey]
          : [];
      if (
        restoredKeys.length &&
        restoredKeys.every((key: KeySelection) => key?.key && key?.code)
      ) {
        selectedKeys.value = restoredKeys.slice(0, MAX_KEYS);
      }
      timerEnabled.value = Boolean(settings.timerEnabled);
      minutes.value = Math.max(0, Number(settings.minutes) || 0);
      seconds.value = Math.min(59, Math.max(0, Number(settings.seconds) || 0));
    } catch {
      localStorage.removeItem("keyhold:settings");
    }
  }

  function finishCapture() {
    if (!pendingKeys.value.length) return;
    selectedKeys.value = pendingKeys.value.map((key) => ({ ...key }));
    isCapturing.value = false;
    pressedCodes.clear();
    window.removeEventListener("keydown", onKeyDownCaptured, true);
    window.removeEventListener("keyup", onKeyUpCaptured, true);
    persist();
  }

  function onKeyDownCaptured(event: KeyboardEvent) {
    if (!isCapturing.value) return;
    event.preventDefault();
    event.stopPropagation();
    if (event.repeat || pressedCodes.has(event.code)) return;

    pressedCodes.add(event.code);
    if (pendingKeys.value.length < MAX_KEYS) {
      pendingKeys.value.push(selectionFromEvent(event));
    } else {
      errorMessage.value = `Maximum ${MAX_KEYS} touches`;
    }
  }

  function onKeyUpCaptured(event: KeyboardEvent) {
    if (!isCapturing.value || !pressedCodes.has(event.code)) return;
    event.preventDefault();
    event.stopPropagation();
    pressedCodes.delete(event.code);
    if (!pressedCodes.size) finishCapture();
  }

  function startCapture() {
    errorMessage.value = "";
    pendingKeys.value = [];
    pressedCodes.clear();
    isCapturing.value = true;
    window.addEventListener("keydown", onKeyDownCaptured, true);
    window.addEventListener("keyup", onKeyUpCaptured, true);
  }

  function clearTimer() {
    if (timerId) clearInterval(timerId);
    timerId = undefined;
  }

  async function setCompactMode(compact: boolean) {
    isCompact.value = compact;
    if (!isDesktop()) return;

    try {
      await invoke("set_compact_mode", {
        compact,
        keyCount: selectedKeys.value.length,
      });
    } catch (error) {
      isCompact.value = false;
      errorMessage.value = String(error);
    }
  }

  function exitCompactMode() {
    return setCompactMode(false);
  }

  async function stopHold() {
    clearTimer();
    if (isDesktop()) {
      try {
        await invoke("release_keys");
      } catch (error) {
        errorMessage.value = String(error);
      }
    }
    isHolding.value = false;
    remainingSeconds.value = 0;
    await exitCompactMode();
  }

  async function startHold() {
    errorMessage.value = "";
    const duration = Math.max(0, minutes.value * 60 + seconds.value);
    if (timerEnabled.value && duration === 0) {
      errorMessage.value = "Choisis une durée";
      return;
    }

    try {
      if (isDesktop()) {
        await invoke("hold_keys", {
          keys: selectedKeys.value,
          durationSeconds: timerEnabled.value ? duration : null,
        });
      }
      isHolding.value = true;
      persist();
      await setCompactMode(true);

      if (timerEnabled.value) {
        remainingSeconds.value = duration;
        timerId = setInterval(() => {
          remainingSeconds.value -= 1;
          if (remainingSeconds.value <= 0) void stopHold();
        }, 1000);
      }
    } catch (error) {
      errorMessage.value = String(error);
      isHolding.value = false;
    }
  }

  function toggleHold() {
    return isHolding.value ? stopHold() : startHold();
  }

  onMounted(async () => {
    restore();
    if (isDesktop()) {
      unlisten = await listen("keyhold://released", () => {
        clearTimer();
        isHolding.value = false;
        remainingSeconds.value = 0;
        void exitCompactMode();
      });
    }
  });

  onBeforeUnmount(() => {
    window.removeEventListener("keydown", onKeyDownCaptured, true);
    window.removeEventListener("keyup", onKeyUpCaptured, true);
    clearTimer();
    unlisten?.();
    if (isHolding.value && isDesktop()) void invoke("release_keys");
  });

  return {
    selectedKeys,
    displayedKeys,
    selectedKeysLabel,
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
  };
}
