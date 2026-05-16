import { writable } from 'svelte/store';

export const zoomLevel = writable<number>(100);

// Apply zoom to document root
zoomLevel.subscribe((val) => {
  if (typeof document !== 'undefined') {
    document.documentElement.style.fontSize = `${(val / 100) * 14}px`;
  }
});

export function zoomIn(): void {
  zoomLevel.update((z) => Math.min(z + 10, 150));
}

export function zoomOut(): void {
  zoomLevel.update((z) => Math.max(z - 10, 60));
}

export function resetZoom(): void {
  zoomLevel.set(100);
}
