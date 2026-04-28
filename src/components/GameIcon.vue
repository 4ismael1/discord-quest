<script setup lang="ts">
import { computed, toRef } from 'vue';
import { useAppIcon } from '@/composables/use-app-icon';

const props = withDefaults(defineProps<{
  appId: string;
  name: string;
  size?: number;
}>(), {
  size: 36,
});

const appIdRef = toRef(props, 'appId');
const { url } = useAppIcon(appIdRef);

const initials = computed(() => {
  const words = props.name.trim().split(/\s+/).slice(0, 2);
  return words.map(w => w[0]).join('').toUpperCase();
});

const sizePx = computed(() => `${props.size}px`);
</script>

<template>
  <div class="game-icon" :style="{ width: sizePx, height: sizePx }">
    <img
      v-if="url"
      :src="url"
      :alt="name"
      class="game-icon-img"
      loading="lazy"
      decoding="async"
      @error="url = null"
    />
    <span v-else class="game-icon-fallback">{{ initials || '?' }}</span>
  </div>
</template>

<style scoped>
.game-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid var(--border-subtle);
  overflow: hidden;
  flex-shrink: 0;
  position: relative;
}

.game-icon-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.game-icon-fallback {
  font-size: 11px;
  font-weight: 700;
  color: var(--text-muted);
  letter-spacing: 0.02em;
  user-select: none;
}
</style>
