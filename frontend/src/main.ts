import { mount } from 'svelte';
import App from './App.svelte';
import './app.css';

function renderBootFailure(error: unknown) {
  const root = document.getElementById('app');
  if (!root) return;
  root.innerHTML = `
    <div style="padding: 40px; font-family: sans-serif; max-width: 600px; margin: 0 auto;">
      <h1 style="color: #dc2626;">应用程序启动失败</h1>
      <p style="color: #64748b;">应用程序在初始化过程中遇到错误。</p>
      <pre style="background: #f1f5f9; padding: 16px; border-radius: 8px; overflow-x: auto; margin: 16px 0;">${String(error)}</pre>
      <button onclick="location.reload()" style="padding: 10px 20px; background: #3b82f6; color: white; border: none; border-radius: 6px; cursor: pointer;">重试</button>
    </div>`;
}

window.addEventListener('error', (event) => {
  console.error('[FATAL]', event.error || event.message);
});

window.addEventListener('unhandledrejection', (event) => {
  console.error('[FATAL Unhandled]', event.reason);
});

let app;
try {
  app = mount(App, {
    target: document.getElementById('app')!,
  });
} catch (error) {
  console.error('[FATAL] App mount failed:', error);
  renderBootFailure(error);
}

export default app;
