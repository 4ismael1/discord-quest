<script setup lang="ts">
import { onMounted } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import MainLayout from './components/MainLayout.vue';
import { Pages, useGlobalState } from './composables/app-state';
import HomeView from './pages/HomeView.vue';
import Playground from './pages/Playground.vue';

const appState = useGlobalState();
const { page } = appState;

onMounted(async () => {
  // Show window only after Vue has rendered — avoids white flash
  const win = getCurrentWindow();
  await win.show();
  await win.setFocus();
});
</script>

<template>
  <MainLayout>
    <HomeView v-show="page === Pages.HOME" />
    <Playground v-show="page === Pages.PLAYGROUND" />
  </MainLayout>
</template>