<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window';

const appWindow = getCurrentWindow();

async function minimize() {
  await appWindow.minimize();
}

async function toggleMaximize() {
  await appWindow.toggleMaximize();
}

async function close() {
  await appWindow.close();
}
</script>

<template>
  <div class="title-bar" data-tauri-drag-region>
    <div class="title-bar-left" data-tauri-drag-region>
      <div class="title-dot"></div>
      <span class="title-text" data-tauri-drag-region>DiscordQuest</span>
    </div>
    <div class="title-bar-controls">
      <button class="control-btn minimize" @click="minimize" aria-label="Minimize">
        <svg width="10" height="1" viewBox="0 0 10 1" fill="currentColor">
          <rect width="10" height="1" rx="0.5"/>
        </svg>
      </button>
      <button class="control-btn maximize" @click="toggleMaximize" aria-label="Maximize">
        <svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.2">
          <rect x="0.6" y="0.6" width="8.8" height="8.8" rx="1.5"/>
        </svg>
      </button>
      <button class="control-btn close" @click="close" aria-label="Close">
        <svg width="10" height="10" viewBox="0 0 10 10" fill="currentColor">
          <path d="M1.7 0.3L5 3.6L8.3 0.3C8.5 0.1 8.9 0.1 9.1 0.3L9.7 0.9C9.9 1.1 9.9 1.5 9.7 1.7L6.4 5L9.7 8.3C9.9 8.5 9.9 8.9 9.7 9.1L9.1 9.7C8.9 9.9 8.5 9.9 8.3 9.7L5 6.4L1.7 9.7C1.5 9.9 1.1 9.9 0.9 9.7L0.3 9.1C0.1 8.9 0.1 8.5 0.3 8.3L3.6 5L0.3 1.7C0.1 1.5 0.1 1.1 0.3 0.9L0.9 0.3C1.1 0.1 1.5 0.1 1.7 0.3Z"/>
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.title-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 38px;
  padding: 0 12px 0 16px;
  user-select: none;
  -webkit-user-select: none;
  flex-shrink: 0;
}

.title-bar-left {
  display: flex;
  align-items: center;
  gap: 10px;
}

.title-dot {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--accent);
  box-shadow: 0 0 8px var(--accent-glow);
  flex-shrink: 0;
}

.title-text {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-secondary);
  letter-spacing: 0.02em;
}

.title-bar-controls {
  display: flex;
  gap: 2px;
}

.control-btn {
  width: 32px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  border-radius: 6px;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}

.control-btn:hover {
  color: var(--text-primary);
  background: rgba(255, 255, 255, 0.06);
  transform: scale(1.05);
}

.control-btn:active {
  transform: scale(0.95);
}

.control-btn.close:hover {
  color: white;
  background: var(--danger);
}
</style>
