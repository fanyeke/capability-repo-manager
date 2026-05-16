import { writable } from 'svelte/store';

export interface ToastMessage {
  id: string;
  message: string;
  type: 'success' | 'error' | 'warning' | 'info';
  duration: number;
}

export const toasts = writable<ToastMessage[]>([]);

let counter = 0;

export function showToast(
  message: string,
  type: ToastMessage['type'] = 'info',
  duration: number = 4000,
): string {
  const id = `toast-${++counter}`;
  toasts.update((t) => [...t, { id, message, type, duration }]);
  if (duration > 0) {
    setTimeout(() => dismissToast(id), duration);
  }
  return id;
}

export function dismissToast(id: string): void {
  toasts.update((t) => t.filter((msg) => msg.id !== id));
}
