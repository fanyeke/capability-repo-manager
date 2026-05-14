import { defineConfig } from "vitest/config";
import { svelte } from "@sveltejs/vite-plugin-svelte";

export default defineConfig({
  plugins: [svelte({ hot: false })],
  resolve: {
    conditions: ["browser"],
    alias: {
      $lib: "/src/lib",
    },
  },
  test: {
    environment: "jsdom",
    include: ["src/tests/unit/**/*.test.ts"],
    css: false,
    server: {
      deps: {
        inline: ["@testing-library/svelte"],
      },
    },
  },
});
