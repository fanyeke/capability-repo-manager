export type LoadState<T> =
  | { status: 'idle' }
  | { status: 'loading' }
  | { status: 'success'; data: T }
  | { status: 'error'; error: string }
  | { status: 'stale'; data: T };

export function isLoaded<T>(state: LoadState<T>): state is { status: 'success'; data: T } {
  return state.status === 'success';
}

export function isLoading<T>(state: LoadState<T>): boolean {
  return state.status === 'loading';
}

export function hasError<T>(state: LoadState<T>): state is { status: 'error'; error: string } {
  return state.status === 'error';
}

export function getData<T>(state: LoadState<T>): T | null {
  if (state.status === 'success' || state.status === 'stale') {
    return state.data;
  }
  return null;
}
