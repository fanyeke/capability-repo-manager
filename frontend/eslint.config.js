import js from "@eslint/js";
import svelte from "eslint-plugin-svelte";
import prettier from "eslint-config-prettier";

export default [
  js.configs.recommended,
  ...svelte.configs["flat/recommended"],
  prettier,
  {
    rules: {
      "no-unused-vars": ["error", { argsIgnorePattern: "^_" }],
      "svelte/no-at-html-tags": "warn",
    },
  },
  {
    ignores: ["dist/", "build/", "src-tauri/"],
  },
];
