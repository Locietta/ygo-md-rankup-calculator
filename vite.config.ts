import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import vuetify from "vite-plugin-vuetify";
import { fileURLToPath, URL } from "node:url";
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  plugins: [
    vue(),
    vuetify({ autoImport: true }),
    tailwindcss(),
  ],
  resolve: {
    alias: [
      {
        find: /^wasm$/,
        replacement: fileURLToPath(new URL("./wasm/pkg/wasm.js", import.meta.url)),
      },
    ],
  },
  optimizeDeps: {
    exclude: ["wasm"],
  },
});
