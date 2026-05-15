import { register, init, getLocaleFromNavigator } from "svelte-i18n";

const defaultLocale = "en";

register("en", () => import("./en.json"));
register("zh", () => import("./zh.json"));

export function setupI18n() {
  init({
    fallbackLocale: defaultLocale,
    initialLocale: getLocaleFromNavigator(),
  });
}
