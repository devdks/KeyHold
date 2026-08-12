<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useAppUpdater } from "~/composables/useAppUpdater";
import { useKeyHold } from "~/composables/useKeyHold";

const {
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
} = useKeyHold();

const {
  updateStatus,
  updateVersion,
  updateProgress,
  isUpdateVisible,
  checkForUpdates,
} = useAppUpdater();

const isDesktop = import.meta.client && "__TAURI_INTERNALS__" in window;

async function minimizeWindow() {
  if (isDesktop) await getCurrentWindow().minimize();
}

async function hideWindow() {
  if (isDesktop) await getCurrentWindow().hide();
}

function handleKeyClick() {
  if (isCompact.value) return exitCompactMode();
  startCapture();
}

onMounted(() => {
  window.setTimeout(() => {
    void checkForUpdates(async () => {
      if (isHolding.value) await toggleHold();
    });
  }, 800);
});
</script>

<template>
  <main
    class="window"
    :class="{ 'is-active': isHolding, 'is-compact': isCompact }"
  >
    <section
      v-if="isUpdateVisible"
      class="update-overlay"
      role="status"
      aria-live="polite"
    >
      <span class="update-mark" aria-hidden="true">
        <svg viewBox="0 0 24 24">
          <path d="M12 3v12m0 0 5-5m-5 5-5-5M5 20h14" />
        </svg>
      </span>
      <strong>Installation de KeyHold {{ updateVersion }}</strong>
      <p>
        {{
          updateStatus === "installing"
            ? "Mise à jour installée, redémarrage…"
            : `Téléchargement… ${updateProgress}%`
        }}
      </p>
      <div
        class="update-progress"
        role="progressbar"
        :aria-valuenow="updateProgress"
        aria-valuemin="0"
        aria-valuemax="100"
      >
        <span :style="{ width: `${updateProgress}%` }" />
      </div>
    </section>

    <section v-if="isCompact" class="compact-panel">
      <button
        type="button"
        class="key-orbit active compact"
        :class="{ single: selectedKeys.length === 1 }"
        title="Cliquer pour rouvrir KeyHold"
        :aria-label="`Touches ${selectedKeysLabel} maintenues. Cliquer pour rouvrir KeyHold`"
        @click="handleKeyClick"
      >
        <span class="orbit-pulse" aria-hidden="true" />
        <span class="key-stack">
          <kbd
            v-for="key in selectedKeys"
            :key="key.code"
            :class="{ long: key.label.length > 5 }"
            :title="key.label"
          >
            {{ key.label }}
          </kbd>
        </span>
      </button>
    </section>

    <template v-else>
      <header class="titlebar" data-tauri-drag-region>
        <div class="brand" data-tauri-drag-region>
          <span class="brand-mark" aria-hidden="true">
            <svg viewBox="0 0 24 24" role="img">
              <rect x="3" y="5" width="18" height="14" rx="4" />
              <path d="M7 10h2m2 0h2m2 0h2M7 14h10" />
            </svg>
          </span>
          <span data-tauri-drag-region>KeyHold</span>
        </div>

        <div class="window-actions">
          <button
            type="button"
            aria-label="Réduire"
            title="Réduire"
            @click="minimizeWindow"
          >
            <svg viewBox="0 0 16 16" aria-hidden="true">
              <path d="M4 8h8" />
            </svg>
          </button>
          <button
            type="button"
            aria-label="Masquer"
            title="Masquer dans la barre système"
            @click="hideWindow"
          >
            <svg viewBox="0 0 16 16" aria-hidden="true">
              <path d="m5 5 6 6m0-6-6 6" />
            </svg>
          </button>
        </div>
      </header>

      <section class="key-panel" aria-live="polite">
        <button
          type="button"
          class="key-orbit"
          :class="{
            active: isHolding,
            capturing: isCapturing,
            multi: displayedKeys.length > 1,
          }"
          :disabled="isHolding"
          :aria-label="
            isCapturing
              ? 'Appuyez maintenant sur une ou plusieurs touches à utiliser'
              : `Touches sélectionnées : ${selectedKeysLabel}. Cliquer pour changer`
          "
          @click="handleKeyClick"
        >
          <span class="orbit-pulse" aria-hidden="true" />
          <span
            v-if="isCapturing && !displayedKeys.length"
            class="capture-placeholder"
          >
            …
          </span>
          <span v-else class="key-stack">
            <kbd
              v-for="key in displayedKeys"
              :key="key.code"
              :class="{ long: key.label.length > 5 }"
              :title="key.label"
            >
              {{ key.label }}
            </kbd>
          </span>
        </button>

        <div class="state-line">
          <span class="state-dot" aria-hidden="true" />
          <span>{{ statusLabel }}</span>
          <strong v-if="isHolding && timerEnabled">{{ remainingLabel }}</strong>
        </div>
        <p class="change-hint">
          {{
            isCapturing
              ? "Maintiens la combinaison, puis relâche toutes les touches"
              : "Clique pour choisir une ou plusieurs touches"
          }}
        </p>
      </section>

      <section class="timer-card">
        <label class="timer-toggle">
          <span>
            <strong>Arrêt automatique</strong>
            <small>Relâcher après une durée</small>
          </span>
          <input v-model="timerEnabled" type="checkbox" :disabled="isHolding" />
          <span class="switch" aria-hidden="true" />
        </label>

        <div v-if="timerEnabled" class="duration-row">
          <label>
            <span>Minutes</span>
            <input
              v-model.number="minutes"
              type="number"
              inputmode="numeric"
              min="0"
              max="999"
              :disabled="isHolding"
            />
          </label>
          <span class="duration-separator">:</span>
          <label>
            <span>Secondes</span>
            <input
              v-model.number="seconds"
              type="number"
              inputmode="numeric"
              min="0"
              max="59"
              :disabled="isHolding"
            />
          </label>
        </div>
      </section>

      <button
        type="button"
        class="primary-action"
        :class="{ stop: isHolding }"
        @click="toggleHold"
      >
        <svg v-if="isHolding" viewBox="0 0 20 20" aria-hidden="true">
          <rect x="5" y="5" width="10" height="10" rx="2" />
        </svg>
        <svg v-else viewBox="0 0 20 20" aria-hidden="true">
          <path d="m8 5 6 5-6 5Z" />
        </svg>
        {{
          isHolding
            ? selectedKeys.length > 1
              ? "Relâcher les touches"
              : "Relâcher la touche"
            : selectedKeys.length > 1
              ? "Maintenir les touches"
              : "Maintenir la touche"
        }}
      </button>

      <footer>
        <span>Arrêt d’urgence</span>
        <kbd>Ctrl</kbd><span>+</span><kbd>Shift</kbd><span>+</span
        ><kbd>F12</kbd>
      </footer>
    </template>
  </main>
</template>
