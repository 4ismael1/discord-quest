import type { Game, GameExecutable } from '@/types/types';
import { getSteamAppId } from '@/composables/steam-quest';

const ILLEGAL_PATH_CHARS = ['>', '<', ':', '"', '|', '?', '*'];

/**
 * Returns true if the executable has a valid Windows path that can be
 * used as a process name for Discord's quest detection.
 */
export function isExecutableQuestCompatible(exe: GameExecutable): boolean {
  if (exe.os !== 'win32') return false;
  return !ILLEGAL_PATH_CHARS.some(c => exe.name.includes(c));
}

/**
 * A game is "quest compatible" when at least one of its executables is a
 * valid Windows executable. These can be detected by Discord's process
 * scanner (RunningGameStore), which is the system that drives quest
 * progress for PLAY_ON_DESKTOP quests.
 *
 * Games that fail this check can only be launched in RPC-only mode, which
 * displays the game on the user's status but does NOT contribute to
 * quest completion.
 */
export function isGameQuestCompatible(game: Game): boolean {
  if (!game.executables || game.executables.length === 0) return false;
  return game.executables.some(isExecutableQuestCompatible);
}

/**
 * Returns the compatibility level as a label / status pair for display.
 */
export function getGameCompatibility(game: Game): {
  level: 'compatible' | 'steam' | 'rpc-only';
  label: string;
  hint: string;
} {
  if (isGameQuestCompatible(game)) {
    return {
      level: 'compatible',
      label: 'Compatible',
      hint: 'Tiene ejecutables Windows válidos. Las quests progresarán normalmente.',
    };
  }
  if (getSteamAppId(game)) {
    return {
      level: 'steam',
      label: 'Steam',
      hint: 'Discord no publica un ejecutable Win32. Se usará el AppID y la ruta publicados por Steam.',
    };
  }
  return {
    level: 'rpc-only',
    label: 'Solo RPC',
    hint: 'Sin ejecutables Windows registrados. Solo se puede mostrar como estado activo (Rich Presence). Las quests NO progresarán.',
  };
}
