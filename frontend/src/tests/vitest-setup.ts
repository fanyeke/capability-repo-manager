import { register, init } from "svelte-i18n";
import en from "$lib/i18n/en.json";
import zh from "$lib/i18n/zh.json";

register("en", () => Promise.resolve({ default: en }));
register("zh", () => Promise.resolve({ default: zh }));

init({
  fallbackLocale: "zh",
  initialLocale: "zh",
});