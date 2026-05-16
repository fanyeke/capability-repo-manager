import { writable } from 'svelte/store';

export const currentPage = writable<string>('dashboard');

export const theme = writable<'dark' | 'light' | 'system'>('dark');
export const reducedMotion = writable<boolean>(false);

// Apply theme to document
theme.subscribe((val) => {
  if (typeof document !== 'undefined') {
    const resolved = val === 'system'
      ? window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
      : val;
    document.documentElement.dataset.theme = resolved;
  }
});

// Apply reduced motion to document
reducedMotion.subscribe((val) => {
  if (typeof document !== 'undefined') {
    document.documentElement.dataset.motion = val ? 'reduced' : '';
  }
});

export function navigateTo(page: string) {
  currentPage.set(page);
}

export function navigateToRepo(repoId: string) {
  currentPage.set(`repo:${repoId}`);
}
