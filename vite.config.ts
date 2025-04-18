import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import vuetify from 'vite-plugin-vuetify'
import path from 'path'

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [
    vue(),
    vuetify({
      autoImport: true
    })
  ],

  resolve: {
    alias: {
      '@': path.resolve(__dirname, 'src'),
    }
  },


  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 31420,
    strictPort: true,
  }
}));
