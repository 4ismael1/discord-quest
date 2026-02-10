<template>
  <div class="logs-view">
    <div class="logs-header">
      <h2 class="logs-title">Registro de Actividad</h2>
      <button class="clear-btn" @click="clearLogs" v-if="logs.length > 0">
        <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor">
          <path d="M3 1.5L6 4.5L9 1.5L10.5 3L7.5 6L10.5 9L9 10.5L6 7.5L3 10.5L1.5 9L4.5 6L1.5 3Z"/>
        </svg>
        Limpiar
      </button>
    </div>

    <div class="logs-container glass">
      <div v-if="logs.length === 0" class="logs-empty">
        <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2">
          <path d="M12 20h9M16.5 3.5a2.121 2.121 0 1 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/>
        </svg>
        <span>Sin registros aún</span>
      </div>
      <div v-else class="logs-list">
        <div v-for="(log, index) in logs" :key="index" class="log-entry">
          <span class="log-time">{{ new Date(log.timestamp).toLocaleTimeString() }}</span>
          <span class="log-type" :class="log.type">{{ log.type }}</span>
          <span class="log-message">{{ log.message }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useGlobalState } from '@/composables/app-state';

const { logs, clearLogs } = useGlobalState();
</script>

<style scoped>
.logs-view {
  padding: 16px 20px 20px;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.logs-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 14px;
}

.logs-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.clear-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-muted);
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all 0.15s ease;
  font-family: inherit;
}

.clear-btn:hover {
  color: var(--danger);
  border-color: rgba(237, 66, 69, 0.3);
  background: var(--danger-soft);
}

.logs-container {
  flex: 1;
  border-radius: var(--radius-lg);
  overflow-y: auto;
  min-height: 0;
}

.logs-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 60px 20px;
  color: var(--text-muted);
  font-size: 13px;
  opacity: 0.5;
}

.logs-list {
  padding: 8px;
}

.log-entry {
  display: flex;
  align-items: baseline;
  gap: 10px;
  padding: 6px 10px;
  border-radius: 6px;
  font-size: 12px;
  transition: background 0.1s ease;
}

.log-entry:hover {
  background: rgba(255, 255, 255, 0.03);
}

.log-time {
  color: var(--text-muted);
  font-family: 'SF Mono', 'Cascadia Code', 'Consolas', monospace;
  font-size: 11px;
  flex-shrink: 0;
}

.log-type {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  padding: 1px 6px;
  border-radius: 4px;
  flex-shrink: 0;
}

.log-type.info {
  color: var(--accent);
  background: var(--accent-soft);
}

.log-type.error {
  color: var(--danger);
  background: var(--danger-soft);
}

.log-type.warning {
  color: var(--warning);
  background: rgba(254, 231, 92, 0.1);
}

.log-type.debug {
  color: var(--success);
  background: var(--success-soft);
}

.log-message {
  color: var(--text-secondary);
  font-family: 'SF Mono', 'Cascadia Code', 'Consolas', monospace;
  word-break: break-word;
}
</style>