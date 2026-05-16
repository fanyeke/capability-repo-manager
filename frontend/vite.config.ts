import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  plugins: [
    svelte({
      onwarn: (_warning, _handler) => {
        // Suppress all Svelte warnings in dev to prevent Vite/pnpm from exiting
      },
    }),
  ],
  resolve: {
    conditions: ["browser"],
    alias: {
      $lib: "/src/lib",
    },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: { ignored: ["**/src-tauri/**"] },
  },
});
