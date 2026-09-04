import { fetch } from '@tauri-apps/plugin-http';
import type { Game } from '@/types/types';

const STEAM_INFO_URL = 'https://api.steamcmd.net/v1/info';
const REQUEST_TIMEOUT_MS = 12_000;

interface SteamLaunchEntry {
  executable?: string;
  type?: string;
  config?: {
    oslist?: string;
  };
}

interface SteamAppData {
  common?: {
    name?: string;
  };
  config?: {
    installdir?: string;
    launch?: Record<string, SteamLaunchEntry>;
  };
  depots?: Record<string, unknown>;
}

interface SteamInfoResponse {
  data?: Record<string, SteamAppData>;
}

export interface SteamLaunchProfile {
  steamAppId: string;
  gameName: string;
  installDir: string;
  executablePath: string;
  depotId?: string;
  inferredExecutable: boolean;
}

export function getSteamAppId(game: Game): string | null {
  const sku = game.third_party_skus?.find(item =>
    item.distributor.toLowerCase() === 'steam' && /^\d+$/.test(item.id),
  );
  return sku?.id ?? null;
}

function pickWindowsExecutable(launch: Record<string, SteamLaunchEntry> | undefined): string | null {
  if (!launch) return null;
  const entries = Object.entries(launch).sort(([left], [right]) =>
    left.localeCompare(right, undefined, { numeric: true }),
  );
  for (const [, entry] of entries) {
    const executable = entry.executable?.replace(/\\/g, '/').trim();
    const osList = entry.config?.oslist?.toLowerCase() ?? 'windows';
    if (executable?.toLowerCase().endsWith('.exe') && (osList === '' || osList.includes('windows'))) {
      return executable;
    }
  }
  return null;
}

export async function resolveSteamLaunch(game: Game): Promise<SteamLaunchProfile> {
  const steamAppId = getSteamAppId(game);
  if (!steamAppId) {
    throw new Error('El juego no publica un AppID de Steam');
  }

  const controller = new AbortController();
  const timeout = window.setTimeout(() => controller.abort(), REQUEST_TIMEOUT_MS);
  try {
    const response = await fetch(`${STEAM_INFO_URL}/${steamAppId}`, {
      signal: controller.signal,
    });
    if (!response.ok) {
      throw new Error(`Steam respondió HTTP ${response.status}`);
    }
    const payload = await response.json() as SteamInfoResponse;
    const app = payload.data?.[steamAppId];
    if (!app) {
      throw new Error('Steam no devolvió metadatos para este juego');
    }

    const gameName = app.common?.name?.trim() || game.name;
    const installDir = app.config?.installdir?.trim() || gameName;
    const publishedExecutable = pickWindowsExecutable(app.config?.launch);
    const fallbackName = `${installDir.replace(/\\/g, '/').split('/').pop() || `App${steamAppId}`}.exe`;
    const executablePath = publishedExecutable || fallbackName;
    const depotId = Object.keys(app.depots ?? {}).find(id => /^\d+$/.test(id));

    return {
      steamAppId,
      gameName,
      installDir,
      executablePath,
      depotId,
      inferredExecutable: !publishedExecutable,
    };
  } catch (error) {
    if (error instanceof DOMException && error.name === 'AbortError') {
      throw new Error('La consulta de Steam agotó el tiempo de espera');
    }
    throw error;
  } finally {
    window.clearTimeout(timeout);
  }
}
