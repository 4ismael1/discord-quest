import { createGlobalState } from '@vueuse/core';
import { ref, watch } from 'vue';

export type ApiRefreshPolicy = 'launch' | 'daily' | 'weekly' | 'manual';

interface StoredSettings {
  multiQuestEnabled?: boolean;
  apiRefreshPolicy?: ApiRefreshPolicy;
}

const STORAGE_KEY = 'discordquest_settings_v1';
const VALID_REFRESH_POLICIES: ApiRefreshPolicy[] = ['launch', 'daily', 'weekly', 'manual'];

function readSettings(): StoredSettings {
  try {
    const parsed = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? '{}') as StoredSettings;
    return parsed && typeof parsed === 'object' ? parsed : {};
  } catch {
    return {};
  }
}

export const useAppSettings = createGlobalState(() => {
  const stored = readSettings();
  const multiQuestEnabled = ref(stored.multiQuestEnabled === true);
  const apiRefreshPolicy = ref<ApiRefreshPolicy>(
    VALID_REFRESH_POLICIES.includes(stored.apiRefreshPolicy as ApiRefreshPolicy)
      ? stored.apiRefreshPolicy as ApiRefreshPolicy
      : 'daily',
  );

  watch([multiQuestEnabled, apiRefreshPolicy], () => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({
      multiQuestEnabled: multiQuestEnabled.value,
      apiRefreshPolicy: apiRefreshPolicy.value,
    }));
  });

  return {
    multiQuestEnabled,
    apiRefreshPolicy,
  };
});
