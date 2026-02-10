<script setup lang="ts">
import type { Game, GameExecutable } from '@/types/types';
import GameExecutables from './GameExecutables.vue';

defineProps<{
  game: Game | null;
  currentlyPlaying: string | null;
  gameName?: string;
  isBusy?: boolean;
  loadingExeKey?: string | null;
}>();

const emit = defineEmits<{
  play: [payload: { game: Game; executable: GameExecutable }];
  stop: [payload: { game: Game; executable: GameExecutable }];
  installAndPlay: [payload: { game: Game; executable: GameExecutable }];
}>();
</script>

<template>
  <div class="details-panel glass">
    <!-- Empty State -->
    <div v-if="!game" class="details-empty">
      <div class="details-empty-icon">
        <svg width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2">
          <path d="M15 15l6 6M10 4a6 6 0 1 0 0 12 6 6 0 0 0 0-12z"/>
        </svg>
      </div>
      <p class="details-empty-text">Selecciona un juego</p>
      <p class="details-empty-hint">Elige un juego de la lista para ver detalles y acciones</p>
    </div>

    <!-- Game Details -->
    <template v-else>
      <div class="details-header">
        <div class="details-title-row">
          <div class="details-dot" :class="{ running: game.is_running }"></div>
          <h2 class="details-title">{{ game.name }}</h2>
        </div>
        <div class="details-meta">
          <span class="details-id">{{ game.id }}</span>
          <span v-if="game.aliases && game.aliases.length > 0" class="details-aliases">
            {{ game.aliases.slice(0, 3).join(' · ') }}
          </span>
        </div>
      </div>

      <!-- Status Banner -->
      <Transition name="banner">
        <div v-if="currentlyPlaying" class="status-banner">
          <div class="status-dot"></div>
          <span>Jugando: <strong>{{ gameName }}</strong></span>
        </div>
      </Transition>

      <!-- Executables -->
      <div class="details-section">
        <h3 class="section-label">Ejecutables</h3>
        <GameExecutables
          :game="game"
          :is-busy="isBusy"
          :loading-exe-key="loadingExeKey"
          @play="emit('play', $event)"
          @stop="emit('stop', $event)"
          @install_and_play="emit('installAndPlay', $event)"
        />
      </div>
    </template>
  </div>
</template>

<style scoped>
.details-panel {
  border-radius: var(--radius-lg);
  display: flex;
  flex-direction: column;
  min-height: 0;
  overflow-y: auto;
}

/* Empty */
.details-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 50px 20px;
  gap: 8px;
}

.details-empty-icon {
  color: var(--text-muted);
  opacity: 0.3;
  margin-bottom: 4px;
}

.details-empty-text {
  font-size: 15px;
  font-weight: 500;
  color: var(--text-secondary);
}

.details-empty-hint {
  font-size: 12px;
  color: var(--text-muted);
  text-align: center;
}

/* Header */
.details-header {
  padding: 18px 18px 0;
}

.details-title-row {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 8px;
}

.details-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: var(--text-muted);
  opacity: 0.3;
  flex-shrink: 0;
}

.details-dot.running {
  background: var(--success);
  opacity: 1;
  box-shadow: 0 0 10px rgba(87, 242, 135, 0.4);
}

.details-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  line-height: 1.2;
}

.details-meta {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding-left: 20px;
}

.details-id {
  font-size: 11px;
  color: var(--text-muted);
  font-family: 'SF Mono', 'Cascadia Code', 'Consolas', monospace;
}

.details-aliases {
  font-size: 11px;
  color: var(--text-muted);
  font-style: italic;
}

/* Status Banner */
.status-banner {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 14px 18px 0;
  padding: 10px 14px;
  border-radius: var(--radius-sm);
  background: var(--success-soft);
  font-size: 12px;
  color: var(--success);
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--success);
  animation: pulse-glow 2s ease-in-out infinite;
}

@keyframes pulse-glow {
  0%, 100% { box-shadow: 0 0 6px rgba(87, 242, 135, 0.4); }
  50% { box-shadow: 0 0 12px rgba(87, 242, 135, 0.6); }
}

/* Section */
.details-section {
  padding: 14px 18px;
}

.section-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.06em;
  margin-bottom: 10px;
}

/* Divider */
.details-divider {
  height: 1px;
  background: var(--border-subtle);
  margin: 0 18px;
}

/* Banner transition */
.banner-enter-active {
  transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
}
.banner-leave-active {
  transition: all 0.15s cubic-bezier(0.4, 0, 0.2, 1);
}
.banner-enter-from {
  opacity: 0;
  transform: translateY(-6px);
  max-height: 0;
  margin-top: 0;
  padding: 0 14px;
}
.banner-leave-to {
  opacity: 0;
  transform: translateY(-4px);
  max-height: 0;
  margin-top: 0;
  padding: 0 14px;
}
</style>
