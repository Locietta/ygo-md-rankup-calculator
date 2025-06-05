import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import wasmPack from "vite-plugin-wasm-pack";
import vuetify from "vite-plugin-vuetify";
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  plugins: [
    vue(),
    wasmPack("./wasm"),
    vuetify({ autoImport: true }),
    tailwindcss(),
  ],
  optimizeDeps: {
    exclude: ["@wasm/wasm"],
  },
});
