import { writable } from "svelte/store";

export const currentPage = writable<string>("dashboard");

export function navigateTo(page: string) {
  currentPage.set(page);
}

export function navigateToRepo(repoId: string) {
  currentPage.set(`repo:${repoId}`);
}
