import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { resolve } from "path";

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [react()],
  
  // Tauri uses this to inject the IPC scripts
  build: {
    // Tauri uses an IIFE for the IPC scripts so we must use a custom rollupOptions
    rollupOptions: {
      // Here we ensure that the entry file is index.html
      input: {
        main: resolve(__dirname, 'index.html'),
      },
    },
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
  
  resolve: {
    alias: {
      "@": resolve(__dirname, "./src"),
    },
  },
  
  // Exclude old frontend files from scanning
  optimizeDeps: {
    exclude: []
  },
  
  // Env variables that start with TAURI_ are exposed
  envPrefix: ['VITE_', 'TAURI_'],
}));