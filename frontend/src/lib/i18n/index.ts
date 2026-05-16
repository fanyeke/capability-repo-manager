import { register, init, getLocaleFromNavigator, waitLocale } from "svelte-i18n";

const defaultLocale = "zh";

register("en", () => import("./en.json"));
register("zh", () => import("./zh.json"));

let ready = false;

export function setupI18n() {
  init({
    fallbackLocale: defaultLocale,
    initialLocale: getLocaleFromNavigator(),
  });
}

export async function waitForI18n(): Promise<void> {
  if (ready) return;
  await waitLocale();
  ready = true;
}
