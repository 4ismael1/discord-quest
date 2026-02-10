<script setup lang="ts">
import { ref, computed, provide, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { randomString } from '@/utils/random-string';
import { GameActionsProvider, GameExecutable, type Game } from '@/types/types';
import { GameActionsKey } from '@/constants/constants';
import { useFetchGameList, type MirrorMeta } from '@/composables/fetch-gamelist';
import { useGlobalState } from '@/composables/app-state';
import SearchBar from '@/components/SearchBar.vue';
import GameListPanel from '@/components/GameListPanel.vue';
import GameDetailsPanel from '@/components/GameDetailsPanel.vue';
import AppModal from '@/components/AppModal.vue';

type DialogKey = 'none' | 'no_game_selected';

const {
  gameDB,
  isLoadingDiscord,
  isLoadingBundled,
  fetchGameList,
  isReadyDiscord,
  isReadyBundled,
  allFetchDone,
  mirrorMeta,
} = useFetchGameList();

const { addLog } = useGlobalState();

// State
const gameList = ref<Game[]>([]);
const selectedGameId = ref<string | null | undefined>(null);
const currentlyPlaying = ref<string | null>(null);
const dialogKey = ref<DialogKey>('none');
const isDialogOpen = ref(false);
const isBusy = ref(false); // Prevents double-clicks during start/stop
const loadingExeKey = ref<string | null>(null); // Track which executable button is loading

const isLoading = computed(() => isLoadingDiscord.value || isLoadingBundled.value);

// ── Persistence ──
const STORAGE_KEY = 'discordquest_gamelist';

function saveGameList() {
  try {
    // Save only the essential data (strip runtime state)
    const toSave = gameList.value.map(g => ({
      uid: g.uid,
      id: g.id,
      name: g.name,
      executables: g.executables.map(e => ({
        is_launcher: e.is_launcher,
        name: e.name,
        os: e.os,
      })),
      aliases: g.aliases,
      themes: g.themes,
      is_installed: g.is_installed,
    }));
    localStorage.setItem(STORAGE_KEY, JSON.stringify(toSave));
  } catch { /* ignore quota errors */ }
}

function loadGameList() {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) {
      const saved: Game[] = JSON.parse(raw);
      if (Array.isArray(saved) && saved.length > 0) {
        gameList.value = saved.map(g => ({
          ...g,
          is_running: false,
          executables: g.executables.map(e => ({ ...e, is_running: false })),
        }));
      }
    }
  } catch { /* ignore parse errors */ }
}

const selectedGame = computed(() => {
  if (!selectedGameId.value) return null;
  return gameList.value.find(g => g.uid === selectedGameId.value) || null;
});

const currentlyPlayingName = computed(() => {
  if (!currentlyPlaying.value) return '';
  return gameList.value.find(g => g.id === currentlyPlaying.value)?.name || '';
});

// Fetch status summary
const fetchStatus = computed(() => {
  if (allFetchDone.value) return null;
  if (isLoadingDiscord.value) return 'Obteniendo lista de juegos...';
  if (isLoadingBundled.value) return 'Cargando lista local...';
  if (isReadyDiscord.value || isReadyBundled.value) return 'Lista cargada';
  return null;
});

// ── Listen for process exit events from backend ──
let unlistenProcessExited: (() => void) | null = null;

onMounted(async () => {
  loadGameList();
  unlistenProcessExited = await listen<{ app_id: string; executable_name: string; game_name: string }>('process_exited', (event) => {
    const { app_id, game_name } = event.payload;
    addLog('warning', `Proceso finalizado: ${game_name}`);
    
    // Update game state — mark ALL executables as stopped (only 1 runs at a time)
    const game = gameList.value.find(g => g.id === app_id);
    if (game) {
      game.is_running = false;
      game.executables.forEach(e => e.is_running = false);
    }
    if (currentlyPlaying.value === app_id) {
      currentlyPlaying.value = null;
    }
  });
});

onUnmounted(() => {
  unlistenProcessExited?.();
});

// ── Game List Actions ──
function addGameToList(game: Game) {
  if (!gameList.value.some(g => g.id === game.id)) {
    gameList.value.push({ uid: randomString(), ...game });
    saveGameList();
  }
}

function removeGameFromList(game: Game) {
  gameList.value = gameList.value.filter(g => g.uid !== game.uid);
  if (selectedGameId.value === game.uid) {
    selectedGameId.value = null;
  }
  saveGameList();
}

function selectGame(game: Game) {
  selectedGameId.value = game.uid;
}

// ── Game Executable Helpers ──
function canPlayGame(game: Game | null) {
  return game ? (game.is_installed && !game.is_running) ?? false : false;
}
function isGameInstalled(game: Game | null) {
  return game?.is_installed ?? false;
}
function isExecutableRunning(exe: GameExecutable) {
  return exe.is_running ?? false;
}
function isGameExecutableInstalled(exe: GameExecutable) {
  return exe.is_installed ?? false;
}

// ── Game Actions ──
async function createDummyGame(game: Game, executable: GameExecutable) {
  const gameToInstall = gameList.value.find(g => g.uid === game.uid);
  const executableItem = gameToInstall?.executables.find(e => e.name === executable.name);
  if (gameToInstall && executableItem) {
    await invoke('create_fake_game', {
      path: executable.path,
      executable_name: executable.filename,
      path_len: executable.segments,
      app_id: gameToInstall.id,
    });
    gameToInstall.is_installed = true;
    executableItem.is_installed = true;
    saveGameList();
    return true;
  }
  return false;
}

async function playGame({ game, executable }: { game: Game; executable: GameExecutable }) {
  if (isBusy.value) return;
  const gameToPlay = gameList.value.find(g => g.uid === game.uid);
  const executableItem = gameToPlay?.executables.find(e => e.name === executable.name);
  if (gameToPlay && executableItem) {
    // Stop any currently running game first (one at a time)
    if (currentlyPlaying.value && currentlyPlaying.value !== game.id) {
      const runningGame = gameList.value.find(g => g.id === currentlyPlaying.value);
      if (runningGame) {
        const runningExe = runningGame.executables.find(e => e.is_running);
        runningGame.is_running = false;
        runningGame.executables.forEach(e => e.is_running = false);
        if (runningExe) {
          invoke('stop_process', { exec_name: runningExe.filename || runningExe.name.split(/\\|\//).pop() || '', app_id: runningGame.id }).catch(() => {});
        }
        addLog('info', `Detenido: ${runningGame.name}`);
      }
      currentlyPlaying.value = null;
    }

    // Optimistic: update UI instantly
    addLog('info', `Iniciando: ${game.name}`);
    currentlyPlaying.value = game.id;
    gameToPlay.is_running = true;
    executableItem.is_running = true;

    // Fire invoke in background — rollback on error
    invoke('run_background_process', {
      name: game.name,
      path: executable.path,
      executable_name: executable.filename,
      path_len: executable.segments,
      app_id: gameToPlay.id,
    }).catch((error) => {
      addLog('error', `Error al iniciar: ${error}`);
      gameToPlay.is_running = false;
      executableItem.is_running = false;
      currentlyPlaying.value = null;
    });
  }
}

function stopPlaying({ game, executable }: { game: Game; executable: GameExecutable }) {
  const gameToPlay = gameList.value.find(g => g.uid === game.uid);
  const executableItem = gameToPlay?.executables.find(e => e.name === executable.name);
  if (gameToPlay && executableItem) {
    // Optimistic: update UI instantly
    gameToPlay.is_running = false;
    executableItem.is_running = false;
    currentlyPlaying.value = null;
    addLog('info', `Detenido: ${game.name}`);

    // Fire invoke in background
    invoke('stop_process', { exec_name: executable.filename!, app_id: gameToPlay.id }).catch(() => {});
  }
}

async function installAndPlay({ game, executable }: { game: Game; executable: GameExecutable }) {
  if (isBusy.value) return;
  isBusy.value = true;
  const exeKey = `${game.id}:${executable.name}`;
  loadingExeKey.value = exeKey;
  try {
    const created = await createDummyGame(game, executable);
    if (created) {
      isBusy.value = false;
      loadingExeKey.value = null;
      playGame({ game, executable });
    } else {
      addLog('error', 'Error al crear el juego');
    }
  } catch (error) {
    addLog('error', `Error: ${error}`);
  } finally {
    isBusy.value = false;
    loadingExeKey.value = null;
  }
}

function closeDialog() {
  isDialogOpen.value = false;
  dialogKey.value = 'none';
}

// ── Helpers ──
function formatMetaDate(iso: string): string {
  try {
    const d = new Date(iso);
    return d.toLocaleDateString('es-ES', { day: '2-digit', month: 'short', year: 'numeric', hour: '2-digit', minute: '2-digit' });
  } catch {
    return iso;
  }
}

provide<GameActionsProvider>(GameActionsKey, {
  canPlayGame,
  isGameInstalled,
  isExecutableRunning,
  isGameExecutableInstalled,
});
</script>

<template>
  <div class="home-view">
    <!-- Fetch Status Indicator -->
    <Transition name="fade">
      <div v-if="fetchStatus && !allFetchDone" class="fetch-indicator">
        <div class="fetch-dot"></div>
        <span>{{ fetchStatus }}</span>
      </div>
    </Transition>

    <!-- Search Row -->
    <div class="search-row">
      <SearchBar
        :game-d-b="gameDB"
        :is-loading="isLoading"
        :added-ids="gameList.map(g => g.id)"
        @select="addGameToList"
        @refetch="fetchGameList()"
      />
    </div>

    <!-- API Info Panel -->
    <Transition name="fade">
      <div v-if="mirrorMeta" class="api-info-panel glass">
        <div class="api-info-header">
          <div class="api-status-badge" :class="mirrorMeta.status === 'ok' ? 'ok' : 'warn'">
            <div class="api-status-dot"></div>
            {{ mirrorMeta.status === 'ok' ? 'Conectado' : 'Advertencia' }}
          </div>
          <span class="api-info-label">API Mirror</span>
        </div>
        <div class="api-info-details">
          <div class="api-info-item">
            <span class="api-info-key">Juegos disponibles</span>
            <span class="api-info-value accent">
              {{ mirrorMeta.items_count ? mirrorMeta.items_count.toLocaleString() : '—' }}
            </span>
          </div>
          <div class="api-info-item">
            <span class="api-info-key">Última actualización</span>
            <span class="api-info-value">{{ formatMetaDate(mirrorMeta.last_updated) }}</span>
          </div>
          <div class="api-info-item">
            <span class="api-info-key">Integridad</span>
            <span class="api-info-value mono">{{ mirrorMeta.sha256?.slice(0, 12) }}…</span>
          </div>
        </div>
      </div>
    </Transition>

    <!-- Two Column Layout -->
    <div class="home-grid">
      <GameListPanel
        :games="gameList"
        :selected-id="selectedGameId"
        @select="selectGame"
        @remove="removeGameFromList"
      />
      <GameDetailsPanel
        :game="selectedGame"
        :currently-playing="currentlyPlaying"
        :game-name="currentlyPlayingName"
        :is-busy="isBusy"
        :loading-exe-key="loadingExeKey"
        @play="playGame"
        @stop="stopPlaying"
        @install-and-play="installAndPlay"
      />
    </div>

    <!-- Modal: No Game Selected -->
    <AppModal :open="isDialogOpen && dialogKey === 'no_game_selected'" title="Sin juego seleccionado" @close="closeDialog">
      <p>Selecciona un juego de la lista primero.</p>
      <template #actions>
        <button class="modal-btn secondary" @click="closeDialog">Aceptar</button>
      </template>
    </AppModal>
  </div>
</template>

<style scoped>
.home-view {
  padding: 16px 20px 20px;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.search-row {
  margin-bottom: 0;
}

/* Fetch Indicator */
.fetch-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 14px;
  margin-bottom: 14px;
  border-radius: var(--radius-sm);
  background: var(--accent-soft);
  font-size: 12px;
  color: var(--accent);
  font-weight: 500;
  animation: slideDown 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

@keyframes slideDown {
  from { opacity: 0; transform: translateY(-8px); }
  to { opacity: 1; transform: translateY(0); }
}

.fetch-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--accent);
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.3; }
}

/* API Info Panel */
.api-info-panel {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 14px;
  padding: 10px 16px;
  margin-bottom: 14px;
  border-radius: var(--radius-md);
}

.api-info-header {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex-shrink: 0;
}

.api-status-badge {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  font-weight: 600;
  padding: 3px 10px;
  border-radius: 20px;
  width: fit-content;
}

.api-status-badge.ok {
  background: rgba(46, 204, 113, 0.12);
  color: #2ecc71;
}

.api-status-badge.warn {
  background: rgba(241, 196, 15, 0.12);
  color: #f1c40f;
}

.api-status-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: currentColor;
  animation: pulse 2s ease-in-out infinite;
}

.api-info-label {
  font-size: 10px;
  color: var(--text-muted);
  font-weight: 500;
  letter-spacing: 0.04em;
  padding-left: 2px;
}

.api-info-details {
  display: flex;
  gap: 20px;
}

.api-info-item {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.api-info-key {
  font-size: 10px;
  color: var(--text-muted);
  font-weight: 500;
}

.api-info-value {
  font-size: 12px;
  color: var(--text-secondary);
  font-weight: 500;
}

.api-info-value.accent {
  color: var(--accent);
  font-weight: 700;
  font-size: 14px;
}

.api-info-value.mono {
  font-family: 'SF Mono', 'Cascadia Code', 'Consolas', monospace;
  font-size: 11px;
}

/* Grid */
.home-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
  flex: 1;
  min-height: 0;
}

/* Fade transition */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s cubic-bezier(0.4, 0, 0.2, 1), transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}

/* Modal Buttons */
.modal-btn {
  padding: 8px 18px;
  border: none;
  border-radius: var(--radius-sm);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  font-family: inherit;
}

.modal-btn.secondary {
  background: rgba(255, 255, 255, 0.06);
  color: var(--text-secondary);
  border: 1px solid var(--border-default);
}

.modal-btn.secondary:hover {
  background: rgba(255, 255, 255, 0.1);
  color: var(--text-primary);
}

.modal-btn.danger {
  background: var(--danger);
  color: white;
}

.modal-btn.danger:hover {
  background: #d63638;
}

</style>
