import { fetch } from '@tauri-apps/plugin-http';

const CACHE_KEY = 'discordquest_icon_hashes_v1';

interface IconRecord {
  hash: string | null; // null = no icon
  fetched_at: number;
}

type IconCache = Record<string, IconRecord>;

// 7 days — long enough to avoid spam, short enough that updates eventually propagate.
const CACHE_TTL_MS = 7 * 24 * 60 * 60 * 1000;

function readCache(): IconCache {
  try {
    const raw = localStorage.getItem(CACHE_KEY);
    return raw ? (JSON.parse(raw) as IconCache) : {};
  } catch {
    return {};
  }
}

function writeCache(cache: IconCache) {
  try {
    localStorage.setItem(CACHE_KEY, JSON.stringify(cache));
  } catch { /* ignore quota */ }
}

/**
 * Returns the Discord CDN URL for an app icon, or null if the app has no icon.
 * Caches the icon hash in localStorage so we only hit the API once per app.
 */
export async function getAppIconUrl(appId: string): Promise<string | null> {
  const cache = readCache();
  const cached = cache[appId];

  // Cache hit (and not expired)
  if (cached && Date.now() - cached.fetched_at < CACHE_TTL_MS) {
    return cached.hash ? buildIconUrl(appId, cached.hash) : null;
  }

  // Cache miss → fetch
  try {
    const res = await fetch(`https://discord.com/api/v10/applications/${appId}/rpc`);
    if (!res.ok) {
      // Don't poison the cache on transient errors
      return cached?.hash ? buildIconUrl(appId, cached.hash) : null;
    }
    const data = (await res.json()) as { icon?: string };
    const hash = typeof data.icon === 'string' ? data.icon : null;

    cache[appId] = { hash, fetched_at: Date.now() };
    writeCache(cache);

    return hash ? buildIconUrl(appId, hash) : null;
  } catch {
    return cached?.hash ? buildIconUrl(appId, cached.hash) : null;
  }
}

function buildIconUrl(appId: string, hash: string): string {
  return `https://cdn.discordapp.com/app-icons/${appId}/${hash}.png?size=128`;
}
