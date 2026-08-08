export default defineNuxtConfig({
  compatibilityDate: '2026-08-08',
  devtools: { enabled: false },
  ssr: false,
  telemetry: false,
  css: ['~/assets/css/main.css'],
  ignore: ['**/src-tauri/**'],
  nitro: {
    output: {
      publicDir: 'dist',
    },
  },
  vite: {
    clearScreen: false,
    envPrefix: ['VITE_', 'TAURI_'],
    server: {
      strictPort: true,
    },
  },
})
