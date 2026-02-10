<script setup lang="ts">
import { ref, computed } from 'vue';
import { shallowRef } from 'vue';
import { refDebounced } from '@vueuse/core';
import { useFuse } from '@vueuse/integrations/useFuse';
import { UseFuseOptions } from '@vueuse/integrations';
import Fuse from 'fuse.js';
import type { Game } from '@/types/types';

const props = defineProps<{
  gameDB: Game[];
  isLoading: boolean;
  addedIds?: string[];
}>();

const emit = defineEmits<{
  select: [game: Game];
  refetch: [];
}>();

const searchQuery = shallowRef('');
const debouncedSearchQuery = refDebounced(searchQuery, 300);
const isOpen = ref(false);

const COPYRIGHT_SYMBOL = '\u00A9';
const TRADEMARK_SYMBOL = '\u2122';
const REGISTERED_SYMBOL = '\u00AE';
const ignoredSymbolsRegex = new RegExp(`[${[COPYRIGHT_SYMBOL, TRADEMARK_SYMBOL, REGISTERED_SYMBOL].join('')}]`, 'g');

const fuseOptions = computed<UseFuseOptions<Game>>(() => ({
  fuseOptions: {
    keys: [
      { name: 'name', weight: 0.7 },
      { name: 'aliases', weight: 0.2 },
      { name: 'executables.name', weight: 0.1 },
    ],
    getFn: (obj: any, path: string[] | string) => {
      const value = Fuse.config.getFn(obj, path);
      return typeof value === 'string' ? value.replace(ignoredSymbolsRegex, '') : value;
    },
    isCaseSensitive: false,
    threshold: 0.5,
    includeScore: true,
    includeMatches: false,
  },
  resultLimit: 10,
  matchAllWhenSearchEmpty: false,
}));

const { results: searchResults } = useFuse(debouncedSearchQuery, () => props.gameDB, fuseOptions);

function handleSelect(game: Game) {
  if (props.addedIds?.includes(game.id)) return; // Already added
  emit('select', game);
  // Close dropdown and clear search immediately for clear feedback
  isOpen.value = false;
  searchQuery.value = '';
}

function handleFocus() {
  isOpen.value = true;
}

function handleClickOutside(_e: FocusEvent) {
  setTimeout(() => {
    isOpen.value = false;
  }, 150);
}
</script>

<template>
  <div class="search-wrapper">
    <div class="search-bar glass-input">
      <svg class="search-icon" width="16" height="16" viewBox="0 0 16 16" fill="none">
        <circle cx="7" cy="7" r="5.5" stroke="currentColor" stroke-width="1.5"/>
        <path d="M11 11L14 14" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
      </svg>
      <input
        v-model="searchQuery"
        type="text"
        placeholder="Buscar juegos..."
        class="search-input"
        @focus="handleFocus"
        @blur="handleClickOutside"
      />
      <button class="refetch-btn" @click="emit('refetch')" :class="{ spinning: isLoading }">
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M1.5 8a6.5 6.5 0 0 1 11.3-4.4M14.5 8a6.5 6.5 0 0 1-11.3 4.4"/>
          <path d="M13.5 1v3.5H10M2.5 15v-3.5H6" stroke-linecap="round" stroke-linejoin="round"/>
        </svg>
      </button>
    </div>

    <!-- Dropdown Results -->
    <Transition name="dropdown">
      <div v-if="isOpen && searchQuery.length > 0" class="search-dropdown glass-elevated"
        @mousedown.prevent
      >
        <div v-if="searchResults.length === 0" class="dropdown-empty">
          <span>No se encontraron juegos</span>
        </div>
        <button
          v-for="result in searchResults"
          :key="result.item.id"
          class="dropdown-item"
          :class="{
            'already-added': addedIds?.includes(result.item.id),
          }"
          @mousedown.prevent="handleSelect(result.item)"
          tabindex="-1"
        >
          <div class="dropdown-item-info">
            <span class="dropdown-item-name">{{ result.item.name }}</span>
            <span class="dropdown-item-id">{{ result.item.id }}</span>
          </div>
          <Transition name="badge-swap" mode="out-in">
            <span v-if="addedIds?.includes(result.item.id)" key="added" class="dropdown-item-added">
              <svg width="12" height="12" viewBox="0 0 12 12" fill="currentColor"><path d="M10 3L4.5 8.5L2 6" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round"/></svg>
              Añadido
            </span>
            <span v-else key="add" class="dropdown-item-add">+ Añadir</span>
          </Transition>
        </button>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.search-wrapper {
  position: relative;
  margin-bottom: 20px;
}

.search-bar {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 0 14px;
  height: 44px;
  border-radius: var(--radius-md);
  transition: border-color 0.2s ease, box-shadow 0.2s ease;
}

.search-bar:focus-within {
  border-color: var(--accent);
  box-shadow: 0 0 0 3px var(--accent-glow);
}

.search-icon {
  color: var(--text-muted);
  flex-shrink: 0;
}

.search-input {
  flex: 1;
  background: none;
  border: none;
  outline: none;
  color: var(--text-primary);
  font-size: 14px;
  font-family: inherit;
}

.search-input::placeholder {
  color: var(--text-muted);
}

.refetch-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border: none;
  background: rgba(255, 255, 255, 0.05);
  color: var(--text-muted);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
  flex-shrink: 0;
}

.refetch-btn:hover {
  color: var(--text-primary);
  background: rgba(255, 255, 255, 0.1);
}

.refetch-btn.spinning svg {
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* Dropdown */
.search-dropdown {
  position: absolute;
  top: calc(100% + 6px);
  left: 0;
  right: 0;
  z-index: 50;
  border-radius: var(--radius-md);
  max-height: 320px;
  overflow-y: auto;
  padding: 6px;
}

.dropdown-empty {
  padding: 20px;
  text-align: center;
  color: var(--text-muted);
  font-size: 13px;
}

.dropdown-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  padding: 10px 12px;
  border: none;
  background: transparent;
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: background 0.15s ease;
  text-align: left;
}

.dropdown-item:hover {
  background: rgba(255, 255, 255, 0.06);
}

.dropdown-item-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.dropdown-item-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.dropdown-item-id {
  font-size: 11px;
  color: var(--text-muted);
  font-family: 'SF Mono', 'Cascadia Code', 'Consolas', monospace;
}

.dropdown-item-add {
  font-size: 12px;
  font-weight: 600;
  color: var(--accent);
  flex-shrink: 0;
  padding: 4px 10px;
  border-radius: 6px;
  background: var(--accent-soft);
  transition: background 0.15s ease;
}

.dropdown-item:hover .dropdown-item-add {
  background: var(--accent-glow);
}

.dropdown-item.already-added {
  opacity: 0.5;
  cursor: default;
}

.dropdown-item-added {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  font-weight: 600;
  color: #2ecc71;
  flex-shrink: 0;
  padding: 4px 10px;
  border-radius: 6px;
  background: rgba(46, 204, 113, 0.1);
}

/* Badge swap transition */
.badge-swap-enter-active {
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
}
.badge-swap-leave-active {
  transition: all 0.1s ease;
}
.badge-swap-enter-from {
  opacity: 0;
  transform: scale(0.8);
}
.badge-swap-leave-to {
  opacity: 0;
  transform: scale(0.8);
}

/* Dropdown Transition */
.dropdown-enter-active,
.dropdown-leave-active {
  transition: opacity 0.15s ease, transform 0.15s ease;
}

.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>
