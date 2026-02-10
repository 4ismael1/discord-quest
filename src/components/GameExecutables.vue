<template>
    <div class="executables-list">
        <div v-if="filteredExecutables.length === 0" class="exe-empty">
            No se encontraron ejecutables compatibles
        </div>
        <div v-for="(executable) in filteredExecutables" :key="executable.name"
            class="exe-row">
            <div class="exe-os-badge">
                {{ executable.os === 'win32' ? 'WIN' : executable.os.toUpperCase() }}
            </div>

            <div class="exe-path-container">
                <div class="exe-path-scroll">
                    <span v-for="(section, i) in splitExecutableName(executable)" :key="i"
                        class="exe-path-segment">
                        {{ section }}
                    </span>
                </div>
            </div>

            <button class="exe-action-btn"
                :class="{
                    'is-stop': gameActions?.isExecutableRunning(executable),
                    'is-loading': isThisLoading(executable),
                }"
                :disabled="isBusy"
                @click="handleLaunch(executable)"
            >
                <!-- Loading spinner -->
                <svg v-if="isThisLoading(executable)" class="spinner" width="14" height="14" viewBox="0 0 14 14" fill="none">
                    <circle cx="7" cy="7" r="5.5" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-dasharray="20 12" />
                </svg>
                <!-- Play icon -->
                <svg v-else-if="!gameActions?.isExecutableRunning(executable)" width="12" height="12" viewBox="0 0 12 12" fill="currentColor">
                    <path d="M2.5 1.5L10.5 6L2.5 10.5V1.5Z"/>
                </svg>
                <!-- Stop icon -->
                <svg v-else width="10" height="10" viewBox="0 0 10 10" fill="currentColor">
                    <rect width="10" height="10" rx="2"/>
                </svg>
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { EXECUTABLE_OS, GameActionsKey } from '@/constants/constants';
import { GameActionsProvider, type Game, type GameExecutable } from '@/types/types';
import { path, app } from '@tauri-apps/api';
import { computed, inject } from 'vue';

const props = defineProps<{
    game: Game;
    isBusy?: boolean;
    loadingExeKey?: string | null;
}>();

const emit = defineEmits<{
    play: [{game: Game, executable: GameExecutable}]
    stop: [{game: Game, executable: GameExecutable}]
    install_and_play: [{game: Game, executable: GameExecutable}]
}>();

const gameActions = inject<GameActionsProvider>(GameActionsKey);

const filteredExecutables = computed(() => {
    return props.game.executables.filter(executable => {
        return executable.os !== EXECUTABLE_OS.LINUX && executable.os !== EXECUTABLE_OS.DARWIN
            && !isValidPath(executable.name);
    });
});

function isThisLoading(executable: GameExecutable): boolean {
    if (!props.loadingExeKey) return false;
    const key = `${props.game.id}:${executable.name}`;
    return props.loadingExeKey === key;
}

function splitExecutableName(executable: GameExecutable) {
    const allSections = executable.name.split(/\\|\//);
    
    const last = executable.name.split(/\\|\//).pop();
    // remove file extension if there was none, just return the last section
    const name = last?.split('.').slice(0, -1).join('.') || last;
    return [
        ...allSections.slice(0, -1),
        name,
    ];
}

function getExecutablePath(executable: GameExecutable) {
    const allSections = executable.name.split(/\\|\//);
    const last = executable.name.split(/\\|\//).pop();
    // remove file extension if there was none, just return the last section
    const name = last?.split('.').slice(0, -1).join('.') || last;
    return [
        ...allSections.slice(0, -1)
    ].join(path.sep())
}

function getFilename(executable: GameExecutable) {
    const last = executable.name.split(/\\|\//).pop();
    // remove file extension if there was none, just return the last section
    return last;
}

function isValidPath(path: string) {
    const illegalChars = ['>', '<', ':', '"', '|', '?', '*'];
    return illegalChars.some(char => path.includes(char));
}

function handleLaunch(executable: GameExecutable) {
    // Handle the launch logic here
    console.log('Launching game:', props.game);
    if(executable.is_running) {
        emit('stop', {
            game: props.game,
            executable: {
                path: getExecutablePath(executable),
                segments: splitExecutableName(executable).length,
                filename: getFilename(executable),
                ...executable
            },
        });
    } else {
        if (!gameActions?.isGameExecutableInstalled(executable)) {
            emit('install_and_play', {
                game: props.game,
                executable: {
                    path: getExecutablePath(executable),
                    segments: splitExecutableName(executable).length,
                    filename: getFilename(executable),
                    ...executable
                },
            });
        } else {
            emit('play', {
                game: props.game,
                executable: {
                    path: getExecutablePath(executable),
                    segments: splitExecutableName(executable).length,
                    filename: getFilename(executable),
                    ...executable
                },
            });
        }
     
    }
    
}

</script>

<style scoped>
.executables-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.exe-empty {
  font-size: 12px;
  color: var(--text-muted);
  text-align: center;
  padding: 12px;
}

.exe-row {
  display: grid;
  grid-template-columns: auto 1fr auto;
  gap: 8px;
  align-items: center;
  padding: 8px 10px;
  border-radius: var(--radius-sm);
  background: rgba(255, 255, 255, 0.02);
  border: 1px solid var(--border-subtle);
  transition: background 0.15s ease;
}

.exe-row:hover {
  background: rgba(255, 255, 255, 0.05);
}

.exe-os-badge {
  font-size: 9px;
  font-weight: 700;
  color: var(--text-muted);
  background: rgba(255, 255, 255, 0.06);
  padding: 3px 8px;
  border-radius: 6px;
  letter-spacing: 0.05em;
  text-transform: uppercase;
}

.exe-path-container {
  position: relative;
  overflow: hidden;
  -webkit-mask-image: linear-gradient(to right, black 85%, transparent 100%);
  mask-image: linear-gradient(to right, black 85%, transparent 100%);
}

.exe-path-scroll {
  display: flex;
  flex-wrap: nowrap;
  gap: 4px;
  overflow-x: auto;
  scrollbar-width: none;
  -ms-overflow-style: none;
}

.exe-path-scroll::-webkit-scrollbar {
  display: none;
}

.exe-path-segment {
  font-size: 11px;
  color: var(--text-secondary);
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid var(--border-subtle);
  padding: 2px 8px;
  border-radius: 5px;
  white-space: nowrap;
  font-family: 'SF Mono', 'Cascadia Code', 'Consolas', monospace;
}

.exe-action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  background: var(--accent);
  color: white;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s cubic-bezier(0.4, 0, 0.2, 1);
  flex-shrink: 0;
}

.exe-action-btn:hover:not(:disabled) {
  background: var(--accent-hover);
  transform: scale(1.08);
  box-shadow: 0 0 12px var(--accent-glow);
}

.exe-action-btn:active:not(:disabled) {
  transform: scale(0.95);
}

.exe-action-btn:disabled:not(.is-loading) {
  opacity: 0.4;
  cursor: not-allowed;
}

.exe-action-btn.is-stop {
  background: var(--danger);
}

.exe-action-btn.is-stop:hover:not(:disabled) {
  background: #d63638;
  box-shadow: 0 0 12px rgba(237, 66, 69, 0.3);
}

.exe-action-btn.is-loading {
  background: var(--accent);
  cursor: pointer;
  opacity: 1;
}

.exe-action-btn.is-stop.is-loading {
  background: var(--danger);
}

/* Spinner animation */
.spinner {
  animation: spin 0.7s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>