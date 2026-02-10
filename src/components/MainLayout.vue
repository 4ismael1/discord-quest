<script setup lang="ts">
import { Pages, useGlobalState } from '@/composables/app-state';
import { useFetchGameList, type MirrorMeta } from '@/composables/fetch-gamelist';
import TitleBar from './TitleBar.vue';
import { ref, onMounted, onUnmounted } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

const appState = useGlobalState();
const { page, setPage } = appState;
const { mirrorMeta } = useFetchGameList();

const activeProcessCount = ref(0);

function formatMetaDate(iso: string): string {
  try {
    const d = new Date(iso);
    return d.toLocaleDateString('es-ES', { day: '2-digit', month: 'short', hour: '2-digit', minute: '2-digit' });
  } catch {
    return iso;
  }
}

// Poll active processes every 2 seconds
let pollInterval: ReturnType<typeof setInterval> | null = null;
let unlistenProcessExited: (() => void) | null = null;

async function refreshProcessCount() {
  try {
    const procs = await invoke('get_active_processes') as any[];
    activeProcessCount.value = procs.length;
  } catch {
    activeProcessCount.value = 0;
  }
}

onMounted(async () => {
  refreshProcessCount();
  pollInterval = setInterval(refreshProcessCount, 3000);
  unlistenProcessExited = await listen('process_exited', () => {
    refreshProcessCount();
  });
});

onUnmounted(() => {
  if (pollInterval) clearInterval(pollInterval);
  unlistenProcessExited?.();
});
</script>

<template>
  <div class="app-shell">
    <TitleBar />

    <!-- Navigation -->
    <nav class="app-nav">
      <button
        v-for="tab in [
          { key: Pages.HOME, label: 'Inicio', icon: '⌂' },
          { key: Pages.PLAYGROUND, label: 'Registros', icon: '⚡' },
        ]"
        :key="tab.key"
        class="nav-tab"
        :class="{ active: page === tab.key }"
        @click="setPage(tab.key)"
      >
        <span class="nav-icon">{{ tab.icon }}</span>
        <span>{{ tab.label }}</span>
      </button>
    </nav>

    <!-- Content -->
    <main class="app-content">
      <slot />
    </main>

    <!-- Status Bar -->
    <div class="status-bar">
      <div class="status-bar-left">
        <div class="status-item">
          <div class="status-dot-mini" :class="activeProcessCount > 0 ? 'active' : 'inactive'"></div>
          <span>{{ activeProcessCount }} {{ activeProcessCount === 1 ? 'proceso activo' : 'procesos activos' }}</span>
        </div>
        <template v-if="mirrorMeta">
          <span class="status-separator">·</span>
          <div class="status-item">
            <span class="status-api-badge" :class="mirrorMeta.status === 'ok' ? 'ok' : 'warn'">API</span>
            <span class="status-meta-date">{{ formatMetaDate(mirrorMeta.last_updated) }}</span>
          </div>
        </template>
      </div>
      <div class="status-bar-right">
        <span>DiscordQuest v1.0.0</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.app-shell {
  display: flex;
  flex-direction: column;
  height: 100dvh;
  overflow: hidden;
  background: var(--bg-base);
  border-radius: 10px;
  border: 1px solid var(--border-subtle);
}

.app-nav {
  display: flex;
  gap: 2px;
  padding: 0 16px 0;
  flex-shrink: 0;
}

.nav-tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-muted);
  background: transparent;
  border: none;
  border-bottom: 2px solid transparent;
  cursor: pointer;
  transition: all 0.2s ease;
  border-radius: 8px 8px 0 0;
}

.nav-tab:hover {
  color: var(--text-secondary);
  background: rgba(255, 255, 255, 0.03);
}

.nav-tab.active {
  color: var(--accent);
  border-bottom-color: var(--accent);
  background: var(--accent-soft);
}

.nav-icon {
  font-size: 14px;
}

.app-content {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding: 0;
}

/* Status bar extras */
.status-separator {
  color: var(--text-muted);
  opacity: 0.4;
  font-size: 10px;
}

.status-api-badge {
  font-size: 9px;
  font-weight: 700;
  letter-spacing: 0.05em;
  padding: 1px 5px;
  border-radius: 4px;
}

.status-api-badge.ok {
  background: rgba(46, 204, 113, 0.15);
  color: #2ecc71;
}

.status-api-badge.warn {
  background: rgba(241, 196, 15, 0.15);
  color: #f1c40f;
}

.status-meta-date {
  font-size: 11px;
  color: var(--text-muted);
}
</style>