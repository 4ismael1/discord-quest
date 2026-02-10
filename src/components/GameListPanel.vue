<script setup lang="ts">
import type { Game } from '@/types/types';

const props = defineProps<{
  games: Game[];
  selectedId: string | null | undefined;
}>();

const emit = defineEmits<{
  select: [game: Game];
  remove: [game: Game];
}>();
</script>

<template>
  <div class="game-list-panel glass">
    <div class="panel-header">
      <h2 class="panel-title">Mis Juegos</h2>
      <span class="panel-count">{{ games.length }}</span>
    </div>

    <!-- Empty State -->
    <div v-if="games.length === 0" class="empty-state">
      <div class="empty-icon">
        <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2">
          <rect x="2" y="6" width="20" height="12" rx="3"/>
          <circle cx="8" cy="12" r="1.5" fill="currentColor"/>
          <circle cx="16" cy="12" r="1.5" fill="currentColor"/>
          <path d="M12 9v6M9 12h6" stroke-width="1"/>
        </svg>
      </div>
      <p class="empty-text">No hay juegos añadidos</p>
      <p class="empty-hint">Busca y añade juegos arriba</p>
    </div>

    <!-- Games List -->
    <div v-else class="games-scroll">
      <button
        v-for="game in games"
        :key="game.uid"
        class="game-item"
        :class="{ selected: selectedId === game.uid }"
        @click="emit('select', game)"
      >
        <div class="game-item-left">
          <div class="game-status-dot" :class="{ running: game.is_running }"></div>
          <div class="game-item-info">
            <span class="game-item-name">{{ game.name }}</span>
            <span v-if="game.is_running" class="game-item-status">En ejecución</span>
          </div>
        </div>
        <button
          v-if="!game.is_running"
          class="game-remove-btn"
          @click.stop="emit('remove', game)"
          aria-label="Remove game"
        >
          <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
            <path d="M3.5 1.5L7 5L10.5 1.5L12.5 3.5L9 7L12.5 10.5L10.5 12.5L7 9L3.5 12.5L1.5 10.5L5 7L1.5 3.5Z" opacity="0.7"/>
          </svg>
        </button>
      </button>
    </div>
  </div>
</template>

<style scoped>
.game-list-panel {
  border-radius: var(--radius-lg);
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 18px 12px;
  flex-shrink: 0;
}

.panel-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  letter-spacing: 0.01em;
}

.panel-count {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-muted);
  background: rgba(255, 255, 255, 0.06);
  padding: 2px 8px;
  border-radius: 10px;
}

/* Empty State */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  gap: 8px;
}

.empty-icon {
  color: var(--text-muted);
  opacity: 0.4;
  margin-bottom: 4px;
}

.empty-text {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-secondary);
}

.empty-hint {
  font-size: 12px;
  color: var(--text-muted);
}

/* Games Scroll */
.games-scroll {
  flex: 1;
  overflow-y: auto;
  padding: 0 8px 8px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

/* Game Item */
.game-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 12px 14px;
  border: 1px solid transparent;
  background: transparent;
  cursor: pointer;
  border-radius: var(--radius-md);
  transition: all 0.15s cubic-bezier(0.4, 0, 0.2, 1);
  text-align: left;
  position: relative;
}

.game-item:hover {
  background: rgba(255, 255, 255, 0.04);
  border-color: var(--border-subtle);
}

.game-item:active {
  transform: scale(0.98);
  background: rgba(88, 101, 242, 0.08);
}

.game-item.selected {
  background: var(--accent-soft);
  border-color: rgba(88, 101, 242, 0.3);
  box-shadow: 0 0 16px -4px var(--accent-glow), inset 0 0 0 1px rgba(88, 101, 242, 0.1);
}

.game-item.selected::before {
  content: '';
  position: absolute;
  left: 0;
  top: 50%;
  transform: translateY(-50%);
  width: 3px;
  height: 60%;
  background: var(--accent);
  border-radius: 0 3px 3px 0;
}

.game-item-left {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.game-status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--text-muted);
  opacity: 0.3;
  flex-shrink: 0;
  transition: all 0.15s cubic-bezier(0.4, 0, 0.2, 1);
}

.game-status-dot.running {
  background: var(--success);
  opacity: 1;
  box-shadow: 0 0 8px rgba(87, 242, 135, 0.4);
  animation: pulse-glow 2s ease-in-out infinite;
}

@keyframes pulse-glow {
  0%, 100% { box-shadow: 0 0 8px rgba(87, 242, 135, 0.4); }
  50% { box-shadow: 0 0 14px rgba(87, 242, 135, 0.6); }
}

.game-item-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.game-item-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.game-item-status {
  font-size: 11px;
  color: var(--success);
  font-weight: 500;
}

.game-remove-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  border-radius: 6px;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  opacity: 0;
  flex-shrink: 0;
}

.game-item:hover .game-remove-btn {
  opacity: 1;
}

.game-remove-btn:hover {
  color: var(--danger);
  background: var(--danger-soft);
}
</style>
