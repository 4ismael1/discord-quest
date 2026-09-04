<script setup lang="ts">
import { computed } from 'vue';
import type { Game, GameExecutable } from '@/types/types';
import GameExecutables from './GameExecutables.vue';
import GameIcon from './GameIcon.vue';
import { getGameCompatibility } from '@/utils/game-compat';

const props = defineProps<{
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
  playRpc: [game: Game];
  stopRpc: [game: Game];
  playSteam: [game: Game];
  stopSteam: [game: Game];
}>();

const compat = computed(() => (props.game ? getGameCompatibility(props.game) : null));
const isRpcOnly = computed(() => compat.value?.level === 'rpc-only');
const isSteamCandidate = computed(() => compat.value?.level === 'steam');
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
          <GameIcon :app-id="game.id" :name="game.name" :size="48" />
          <div class="details-title-text">
            <div class="details-title-line">
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
        </div>
      </div>

      <!-- Status Banner -->
      <Transition name="banner">
        <div v-if="currentlyPlaying" class="status-banner">
          <div class="status-dot"></div>
          <span>Jugando: <strong>{{ gameName }}</strong></span>
        </div>
      </Transition>

      <!-- Compatibility Banner -->
      <div v-if="compat" class="compat-banner" :class="compat.level">
        <div class="compat-icon">
          <svg v-if="compat.level === 'compatible'" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <path d="M20 6L9 17l-5-5"/>
          </svg>
          <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M12 9v4M12 17h.01M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z"/>
          </svg>
        </div>
        <div class="compat-text">
          <div class="compat-label">{{ compat.label }}</div>
          <div class="compat-hint">{{ compat.hint }}</div>
        </div>
      </div>

      <!-- Executables -->
      <div class="details-section">
        <h3 class="section-label">{{ isSteamCandidate ? 'Integración Steam' : isRpcOnly ? 'Modo Discord RPC' : 'Ejecutables' }}</h3>

        <div v-if="isSteamCandidate" class="rpc-only-panel steam-panel">
          <div class="rpc-only-row">
            <div class="rpc-only-info">
              <div class="rpc-only-title steam-title">Steam Quest Runner</div>
              <div class="rpc-only-desc">
                Usa el AppID y la ruta oficial de Steam únicamente porque Discord no publica un ejecutable Win32 para este juego.
              </div>
            </div>
            <button
              class="rpc-launch-btn steam-launch-btn"
              :class="{ 'is-stop': game!.is_running }"
              :disabled="isBusy"
              @click="game!.is_running ? emit('stopSteam', game!) : emit('playSteam', game!)"
            >
              <svg v-if="!game!.is_running" width="12" height="12" viewBox="0 0 12 12" fill="currentColor">
                <path d="M2.5 1.5L10.5 6L2.5 10.5V1.5Z"/>
              </svg>
              <svg v-else width="10" height="10" viewBox="0 0 10 10" fill="currentColor">
                <rect width="10" height="10" rx="2"/>
              </svg>
              <span>{{ game!.is_running ? 'Detener' : 'Iniciar Steam' }}</span>
            </button>
          </div>
        </div>

        <!-- RPC-only fallback for incompatible games -->
        <div v-else-if="isRpcOnly" class="rpc-only-panel">
          <div class="rpc-only-row">
            <div class="rpc-only-info">
              <div class="rpc-only-title">Discord Rich Presence</div>
              <div class="rpc-only-desc">
                Conecta vía IPC con el ID del juego ({{ game!.id }}). Mostrará al juego en tu estado de Discord pero <strong>no progresarán quests</strong>.
              </div>
            </div>
            <button
              class="rpc-launch-btn"
              :class="{ 'is-stop': game!.is_running }"
              :disabled="isBusy"
              @click="game!.is_running ? emit('stopRpc', game!) : emit('playRpc', game!)"
            >
              <svg v-if="!game!.is_running" width="12" height="12" viewBox="0 0 12 12" fill="currentColor">
                <path d="M2.5 1.5L10.5 6L2.5 10.5V1.5Z"/>
              </svg>
              <svg v-else width="10" height="10" viewBox="0 0 10 10" fill="currentColor">
                <rect width="10" height="10" rx="2"/>
              </svg>
              <span>{{ game!.is_running ? 'Detener' : 'Iniciar RPC' }}</span>
            </button>
          </div>
        </div>

        <!-- Normal executables list -->
        <GameExecutables
          v-else
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
  align-items: flex-start;
  gap: 12px;
  margin-bottom: 4px;
}

.details-title-text {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.details-title-line {
  display: flex;
  align-items: center;
  gap: 10px;
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

/* Compatibility Banner */
.compat-banner {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  margin: 12px 18px 0;
  padding: 10px 12px;
  border-radius: var(--radius-sm);
  border: 1px solid;
}

.compat-banner.compatible {
  background: rgba(46, 204, 113, 0.06);
  border-color: rgba(46, 204, 113, 0.2);
}

.compat-banner.rpc-only {
  background: rgba(241, 196, 15, 0.06);
  border-color: rgba(241, 196, 15, 0.2);
}

.compat-banner.steam {
  border-color: rgba(102, 192, 244, 0.28);
  background: rgba(27, 40, 56, 0.48);
}

.compat-icon {
  flex-shrink: 0;
  margin-top: 1px;
}

.compat-banner.compatible .compat-icon { color: #2ecc71; }
.compat-banner.rpc-only .compat-icon { color: #f1c40f; }
.compat-banner.steam .compat-icon { color: #66c0f4; }

.compat-text {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.compat-label {
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.02em;
}

.compat-banner.compatible .compat-label { color: #2ecc71; }
.compat-banner.rpc-only .compat-label { color: #f1c40f; }
.compat-banner.steam .compat-label { color: #66c0f4; }

.rpc-only-title.steam-title { color: #66c0f4; }

.rpc-launch-btn.steam-launch-btn {
  background: rgba(102, 192, 244, 0.16);
  border-color: rgba(102, 192, 244, 0.32);
  color: #66c0f4;
}

.compat-hint {
  font-size: 11px;
  color: var(--text-secondary);
  line-height: 1.4;
}

/* RPC-only panel */
.rpc-only-panel {
  border-radius: var(--radius-sm);
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid var(--border-subtle);
  padding: 12px;
}

.rpc-only-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.rpc-only-info {
  flex: 1;
  min-width: 0;
}

.rpc-only-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 4px;
}

.rpc-only-desc {
  font-size: 11px;
  color: var(--text-muted);
  line-height: 1.45;
}

.rpc-only-desc strong {
  color: #f1c40f;
  font-weight: 600;
}

.rpc-launch-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border: none;
  background: var(--accent);
  color: white;
  border-radius: 8px;
  cursor: pointer;
  font-size: 12px;
  font-weight: 600;
  transition: all 0.15s cubic-bezier(0.4, 0, 0.2, 1);
  flex-shrink: 0;
  font-family: inherit;
}

.rpc-launch-btn:hover:not(:disabled) {
  background: var(--accent-hover);
  transform: scale(1.04);
  box-shadow: 0 0 14px var(--accent-glow);
}

.rpc-launch-btn.steam-launch-btn:hover:not(:disabled) {
  background: rgba(102, 192, 244, 0.24);
  box-shadow: 0 0 14px rgba(102, 192, 244, 0.2);
}

.rpc-launch-btn:active:not(:disabled) {
  transform: scale(0.96);
}

.rpc-launch-btn:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.rpc-launch-btn.is-stop {
  background: var(--danger);
}

.rpc-launch-btn.is-stop:hover:not(:disabled) {
  background: #d63638;
  box-shadow: 0 0 14px rgba(237, 66, 69, 0.3);
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
