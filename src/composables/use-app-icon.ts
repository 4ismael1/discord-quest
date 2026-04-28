import { ref, watchEffect, type Ref } from 'vue';
import { getAppIconUrl } from '@/utils/fetch-app-icon';

/**
 * Reactive composable that resolves a Discord app icon URL for the given
 * application id. Returns null while loading, the URL when ready, or null
 * if the app has no icon.
 */
export function useAppIcon(appId: Ref<string | null | undefined>) {
  const url = ref<string | null>(null);

  watchEffect(async () => {
    const id = appId.value;
    url.value = null;
    if (!id) return;
    const resolved = await getAppIconUrl(id);
    // Re-check the input didn't change while we were awaiting
    if (appId.value === id) {
      url.value = resolved;
    }
  });

  return { url };
}
