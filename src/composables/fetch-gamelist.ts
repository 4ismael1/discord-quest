import { Game } from '@/types/types';
import { fetch } from '@tauri-apps/plugin-http';
import { tryOnMounted, useAsyncState } from '@vueuse/core';
import { ref, watch } from 'vue';
import { message } from '@tauri-apps/plugin-dialog';
import { useGlobalState } from './app-state';

// ── Mirror API URLs ──
const MIRROR_URLS = {
    primary: 'https://4ismael1.github.io/discord-detectable-mirror/detectable.json',
    fallback: 'https://cdn.jsdelivr.net/gh/4ismael1/discord-detectable-mirror@main/docs/detectable.json',
};

const META_URLS = {
    primary: 'https://4ismael1.github.io/discord-detectable-mirror/meta.json',
    fallback: 'https://cdn.jsdelivr.net/gh/4ismael1/discord-detectable-mirror@main/docs/meta.json',
};

export interface MirrorMeta {
    last_updated: string;
    etag: string | null;
    source_url: string;
    sha256: string;
    status: string;
    items_count?: number;
}

export function useFetchGameList() {
    const { addLog } = useGlobalState();
    const mirrorMeta = ref<MirrorMeta | null>(null);

    async function fetchFromUrl(url: string): Promise<any> {
        const response = await fetch(url);
        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        return response.json();
    }

    async function fetchMirrorMeta(): Promise<MirrorMeta | null> {
        try {
            addLog('debug', 'Obteniendo metadatos del espejo...');
            const data = await fetchFromUrl(META_URLS.primary);
            return data as MirrorMeta;
        } catch {
            addLog('debug', 'Fallback: metadatos desde CDN...');
            try {
                const data = await fetchFromUrl(META_URLS.fallback);
                return data as MirrorMeta;
            } catch {
                addLog('warning', 'No se pudieron obtener metadatos del espejo');
                return null;
            }
        }
    }

    async function fetchGameListFromMirror(): Promise<Game[] | unknown[]> {
        addLog('info', 'Obteniendo lista de juegos desde espejo...');

        // Fetch meta in parallel
        fetchMirrorMeta().then(meta => {
            if (meta) {
                mirrorMeta.value = meta;
                addLog('debug', `Espejo: ${meta.status} | Actualizado: ${meta.last_updated}`);
            }
        });

        // Try primary mirror
        try {
            const data = await fetchFromUrl(MIRROR_URLS.primary);
            return data;
        } catch {
            addLog('warning', 'Espejo principal no disponible, usando fallback CDN...');
        }

        // Try CDN fallback
        try {
            const data = await fetchFromUrl(MIRROR_URLS.fallback);
            return data;
        } catch (e) {
            addLog('error', 'Fallback CDN tampoco disponible');
            throw e;
        }
    }

    const {
        state: gameListFromMirror,
        error: errorMirror,
        isReady: isReadyDiscord,
        execute: executeMirror,
        isLoading: isLoadingDiscord
    } = useAsyncState(fetchGameListFromMirror, [], {
        immediate: false,
        resetOnExecute: true,
    });

    const {
        state: bundledGameList,
        error: errorBundled,
        isReady: isReadyBundled,
        execute: executeBundled,
        isLoading: isLoadingBundled
    } = useAsyncState(() => {
        const result = import('../assets/gamelist.json').then(res => res.default);
        addLog('info', 'Cargando lista de juegos local...');
        return result;
    }, [], {
        immediate: false,
        resetOnExecute: true,
    });

    const fetchError = ref<string | null>(null);
    const gameDB = ref<Game[]>([]);
    const allFetchDone = ref(false);

    function isValidGameList(data: any): boolean {
        return Array.isArray(data) && data[0] && 'aliases' in data[0] && 'name' in data[0] && 'executables' in data[0];
    }

    watch(() => isReadyDiscord.value, async (newVal) => {
        addLog('debug', 'isReadyMirror: ' + newVal);
    });

    watch(() => isReadyBundled.value, async (newVal) => {
        addLog('debug', 'isReadyBundled: ' + newVal);
    });

    let timeoutId: ReturnType<typeof setTimeout> | null = null;

    async function fetchGameList() {
        allFetchDone.value = false;
        fetchError.value = null;
        addLog('info', 'Obteniendo lista de juegos...');

        try {
            await Promise.all([executeMirror(), executeBundled()]);
        } catch {
            addLog('error', 'Error al obtener la lista de juegos.');
        }

        if (errorMirror.value) {
            fetchError.value = 'Error al obtener lista desde el espejo.';
            addLog('error', 'Error al obtener lista desde espejo');
        }

        if (errorBundled.value) {
            addLog('error', 'Error al cargar lista local');
        }

        if (fetchError.value && errorBundled.value) {
            await message('Hubo un error al obtener la lista de juegos. ' + fetchError.value, {
                title: 'Error al obtener juegos',
                kind: 'error',
                buttons: { ok: 'Aceptar' }
            });
        }

        if (gameListFromMirror.value && gameListFromMirror.value?.length > 0 && isValidGameList(gameListFromMirror.value)) {
            gameDB.value = gameListFromMirror.value as Game[] || [];
            addLog('info', `Usando lista del espejo. ${gameListFromMirror.value.length} juegos.`);
        } else {
            addLog('info', `Usando lista local. ${bundledGameList.value.length} juegos.`);
            gameDB.value = bundledGameList.value;
        }

        timeoutId = setTimeout(() => {
            allFetchDone.value = true;
        }, 1800);
    }

    watch(allFetchDone, (newVal) => {
        if (newVal && timeoutId) {
            clearTimeout(timeoutId);
        }
    });

    tryOnMounted(async () => {
        await fetchGameList();
    });

    return {
        gameListFromMirror,
        bundledGameList,
        fetchError,
        isReadyDiscord,
        isReadyBundled,
        gameDB,
        fetchGameList,
        isLoadingDiscord,
        isLoadingBundled,
        allFetchDone,
        mirrorMeta,
    };
}