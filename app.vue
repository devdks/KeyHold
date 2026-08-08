<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useKeyHold } from "~/composables/useKeyHold";

const {
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
} = useKeyHold();

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
</script>

<template>
  <main
    class="window"
    :class="{ 'is-active': isHolding, 'is-compact': isCompact }"
  >
    <section v-if="isCompact" class="compact-panel">
      <button
        type="button"
        class="key-orbit active compact"
        title="Cliquer pour rouvrir KeyHold"
        :aria-label="`Touche ${selectedKey.label} maintenue. Cliquer pour rouvrir KeyHold`"
        @click="handleKeyClick"
      >
        <span class="orbit-pulse" aria-hidden="true" />
        <kbd>{{ selectedKey.label }}</kbd>
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
          :class="{ active: isHolding, capturing: isCapturing }"
          :disabled="isHolding"
          :aria-label="
            isCapturing
              ? 'Appuyez maintenant sur la touche à utiliser'
              : `Touche sélectionnée : ${selectedKey.label}. Cliquer pour changer`
          "
          @click="handleKeyClick"
        >
          <span class="orbit-pulse" aria-hidden="true" />
          <kbd>{{ isCapturing ? "…" : selectedKey.label }}</kbd>
        </button>

        <div class="state-line">
          <span class="state-dot" aria-hidden="true" />
          <span>{{ statusLabel }}</span>
          <strong v-if="isHolding && timerEnabled">{{ remainingLabel }}</strong>
        </div>
        <p class="change-hint">
          {{
            isCapturing
              ? "Appuie sur la touche de ton choix"
              : "Clique sur la touche pour la modifier"
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
        {{ isHolding ? "Relâcher la touche" : "Maintenir la touche" }}
      </button>

      <footer>
        <span>Arrêt d’urgence</span>
        <kbd>Ctrl</kbd><span>+</span><kbd>Shift</kbd><span>+</span
        ><kbd>F12</kbd>
      </footer>
    </template>
  </main>
</template>
