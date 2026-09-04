import { Game } from '@/types/types';
import { fetch } from '@tauri-apps/plugin-http';
import { createGlobalState, tryOnMounted } from '@vueuse/core';
import { ref } from 'vue';
import { useGlobalState } from './app-state';
import { useAppSettings, type ApiRefreshPolicy } from './app-settings';
import { idbGet, idbSet } from '@/utils/idb-cache';

// ── Mirror endpoints ──
const MIRROR_URLS = {
    primary: 'https://4ismael1.github.io/discord-detectable-mirror/detectable.json',
    fallback: 'https://cdn.jsdelivr.net/gh/4ismael1/discord-detectable-mirror@main/docs/detectable.json',
};

const META_URLS = {
    primary: 'https://4ismael1.github.io/discord-detectable-mirror/meta.json',
    fallback: 'https://cdn.jsdelivr.net/gh/4ismael1/discord-detectable-mirror@main/docs/meta.json',
};

// ── Cache keys (IndexedDB) ──
const CACHE_DB_KEY = 'gamedb_v2';
const CACHE_META_KEY = 'gamedb_meta_v2';

export interface MirrorMeta {
    last_updated: string;
    etag: string | null;
    source_url: string;
    sha256: string;
    status: string;
    items_count?: number;
}

interface CachedMeta {
    sha256: string;
    fetched_at: number;
    checked_at?: number;
    mirror?: MirrorMeta;
}

const REFRESH_INTERVALS: Record<Exclude<ApiRefreshPolicy, 'launch' | 'manual'>, number> = {
    daily: 24 * 60 * 60 * 1000,
    weekly: 7 * 24 * 60 * 60 * 1000,
};

function isValidGameList(data: any): boolean {
    return Array.isArray(data) && data[0] && 'aliases' in data[0] && 'name' in data[0] && 'executables' in data[0];
}

export const useFetchGameList = createGlobalState(() => {
    const { addLog } = useGlobalState();
    const { apiRefreshPolicy } = useAppSettings();
    const mirrorMeta = ref<MirrorMeta | null>(null);
    const lastCheckedAt = ref<number | null>(null);
    const fetchError = ref<string | null>(null);
    const gameDB = ref<Game[]>([]);
    const allFetchDone = ref(false);
    const isLoadingDiscord = ref(false);
    const isLoadingBundled = ref(false);
    const isReadyDiscord = ref(false);
    const isReadyBundled = ref(false);

    async function fetchJson<T>(url: string): Promise<T> {
        const response = await fetch(url);
        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        return (await response.json()) as T;
    }

    async function fetchMirrorMeta(): Promise<MirrorMeta | null> {
        try {
            return await fetchJson<MirrorMeta>(META_URLS.primary);
        } catch {
            try {
                return await fetchJson<MirrorMeta>(META_URLS.fallback);
            } catch {
                return null;
            }
        }
    }

    async function fetchFullGameList(): Promise<Game[] | null> {
        try {
            return await fetchJson<any>(MIRROR_URLS.primary);
        } catch {
            addLog('warning', 'Espejo principal no disponible, usando fallback CDN...');
            try {
                return await fetchJson<any>(MIRROR_URLS.fallback);
            } catch (e) {
                addLog('error', 'Fallback CDN tampoco disponible');
                throw e;
            }
        }
    }

    async function loadBundled(): Promise<Game[]> {
        isLoadingBundled.value = true;
        try {
            const result = (await import('../assets/gamelist.json')).default as Game[];
            isReadyBundled.value = true;
            return result;
        } finally {
            isLoadingBundled.value = false;
        }
    }

    /**
     * Smart fetch flow:
     *   1. Load cached gamelist from IndexedDB (if any) — instant first paint
     *   2. Fetch meta.json from mirror to get current sha256
     *   3. If sha256 matches cache → done (no network for the heavy file)
     *   4. If sha256 differs (or no cache) → fetch full list, update cache
     *   5. If everything fails → fall back to bundled list
     */
    async function fetchGameList(force = false) {
        allFetchDone.value = false;
        fetchError.value = null;

        // 1) Load cache immediately so the UI has data right away
        const [cached, cachedMeta] = await Promise.all([
            idbGet<Game[]>(CACHE_DB_KEY),
            idbGet<CachedMeta>(CACHE_META_KEY),
        ]);

        const hasValidCache = cached !== null && isValidGameList(cached);
        if (cached && hasValidCache) {
            gameDB.value = cached;
            isReadyDiscord.value = true;
            addLog('info', `Cache cargado: ${cached.length} juegos`);
        }

        if (cachedMeta?.mirror) mirrorMeta.value = cachedMeta.mirror;
        lastCheckedAt.value = cachedMeta?.checked_at ?? cachedMeta?.fetched_at ?? null;

        const policy = apiRefreshPolicy.value;
        const elapsed = Date.now() - (lastCheckedAt.value ?? 0);
        const refreshDue = force
            || !hasValidCache
            || policy === 'launch'
            || (policy !== 'manual' && elapsed >= REFRESH_INTERVALS[policy]);

        if (!refreshDue) {
            addLog('info', policy === 'manual'
                ? 'API en modo manual — usando cache local'
                : 'Cache reciente — comprobación de API omitida');
            allFetchDone.value = true;
            return;
        }

        // 2) Fetch the meta to find out if the mirror has fresher data
        isLoadingDiscord.value = true;
        const meta = await fetchMirrorMeta();
        const checkedAt = Date.now();
        lastCheckedAt.value = checkedAt;
        if (meta) {
            mirrorMeta.value = meta;
            addLog('debug', `Espejo: ${meta.status} | sha256: ${meta.sha256?.slice(0, 12)}…`);
        }

        try {
            const cacheIsFresh =
                !force
                && cached
                && cachedMeta
                && meta
                && cachedMeta.sha256 === meta.sha256;

            if (cacheIsFresh) {
                addLog('info', 'Cache al día — sin descarga');
                await idbSet(CACHE_META_KEY, {
                    ...cachedMeta,
                    sha256: meta.sha256,
                    fetched_at: cachedMeta?.fetched_at ?? checkedAt,
                    checked_at: checkedAt,
                    mirror: meta,
                });
            } else if (meta || !hasValidCache || force) {
                if (cached && meta) {
                    addLog('info', 'Cambios detectados en el espejo, actualizando...');
                } else {
                    addLog('info', 'Descargando lista de juegos...');
                }

                try {
                    const fresh = await fetchFullGameList();
                    if (fresh && isValidGameList(fresh)) {
                        gameDB.value = fresh as Game[];
                        const cacheOk = await idbSet(CACHE_DB_KEY, fresh);
                        await idbSet(CACHE_META_KEY, {
                            sha256: meta?.sha256 ?? cachedMeta?.sha256 ?? '',
                            fetched_at: checkedAt,
                            checked_at: checkedAt,
                            mirror: meta ?? cachedMeta?.mirror,
                        });
                        isReadyDiscord.value = true;
                        addLog('info', `Lista actualizada: ${fresh.length} juegos${cacheOk ? ' (cacheados)' : ''}`);
                    } else {
                        throw new Error('Lista del mirror inválida');
                    }
                } catch (e) {
                    fetchError.value = String(e);
                    addLog('error', `Error al descargar lista: ${e}`);
                    if (!cached) {
                        const bundled = await loadBundled();
                        gameDB.value = bundled;
                        addLog('warning', `Usando lista local (${bundled.length} juegos)`);
                    }
                }
            } else {
                addLog('warning', 'Sin metadatos remotos, usando cache local');
                if (cachedMeta) {
                    await idbSet(CACHE_META_KEY, { ...cachedMeta, checked_at: checkedAt });
                }
            }
        } finally {
            isLoadingDiscord.value = false;
            allFetchDone.value = true;
        }
    }

    tryOnMounted(async () => {
        await fetchGameList();
    });

    return {
        fetchError,
        isReadyDiscord,
        isReadyBundled,
        gameDB,
        fetchGameList,
        isLoadingDiscord,
        isLoadingBundled,
        allFetchDone,
        mirrorMeta,
        lastCheckedAt,
    };
});
