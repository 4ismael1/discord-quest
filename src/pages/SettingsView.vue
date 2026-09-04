<script setup lang="ts">
import { computed } from 'vue';
import { useAppSettings, type ApiRefreshPolicy } from '@/composables/app-settings';
import { useFetchGameList } from '@/composables/fetch-gamelist';

const { multiQuestEnabled, apiRefreshPolicy } = useAppSettings();
const { fetchGameList, isLoadingDiscord, lastCheckedAt } = useFetchGameList();

const refreshOptions: Array<{ value: ApiRefreshPolicy; label: string }> = [
  { value: 'daily', label: 'Una vez al día (recomendado)' },
  { value: 'launch', label: 'En cada inicio' },
  { value: 'weekly', label: 'Una vez por semana' },
  { value: 'manual', label: 'Solo manualmente' },
];

const lastCheckedLabel = computed(() => {
  if (!lastCheckedAt.value) return 'Aún no comprobada';
  return new Date(lastCheckedAt.value).toLocaleString('es-ES', {
    day: '2-digit',
    month: 'short',
    hour: '2-digit',
    minute: '2-digit',
  });
});
</script>

<template>
  <div class="settings-page">
    <div class="settings-heading">
      <h1>Configuración</h1>
      <p>Preferencias de ejecución y actualización de datos.</p>
    </div>

    <section class="settings-card glass">
      <div class="setting-copy">
        <div class="setting-title-row">
          <h2>Ejecutar varias quests</h2>
          <span class="setting-badge">Experimental</span>
        </div>
        <p>
          Permite mantener varios runners activos al mismo tiempo. Cada juego conserva su propio
          proceso y, en modo Steam, su propio journal de limpieza.
        </p>
        <p class="setting-note">
          RPC solo puede mostrar una actividad a la vez y no completa quests; esta opción resulta
          útil principalmente con runners normales y Steam.
        </p>
      </div>
      <label class="switch" :title="multiQuestEnabled ? 'Desactivar varias quests' : 'Activar varias quests'">
        <input v-model="multiQuestEnabled" type="checkbox">
        <span class="switch-track"><span class="switch-thumb"></span></span>
      </label>
    </section>

    <section class="settings-card glass api-card">
      <div class="setting-copy">
        <h2>Actualización de la lista</h2>
        <p>
          La lista se abre desde la caché local. Solo se descarga completa cuando cambia el SHA del
          espejo o cuando fuerzas una actualización.
        </p>
        <span class="last-check">Última comprobación: {{ lastCheckedLabel }}</span>
      </div>
      <div class="api-controls">
        <select v-model="apiRefreshPolicy" class="settings-select">
          <option v-for="option in refreshOptions" :key="option.value" :value="option.value">
            {{ option.label }}
          </option>
        </select>
        <button class="refresh-button" :disabled="isLoadingDiscord" @click="fetchGameList(true)">
          {{ isLoadingDiscord ? 'Actualizando…' : 'Actualizar ahora' }}
        </button>
      </div>
    </section>

    <section class="settings-card glass safety-card">
      <div class="safety-icon">✓</div>
      <div class="setting-copy">
        <h2>Limpieza Steam siempre activa</h2>
        <p>
          Los runners, manifests, marcadores y respaldos temporales se limpian al detener el proceso.
          Si Windows se cierra inesperadamente, la recuperación continúa al abrir DiscordQuest.
        </p>
      </div>
    </section>
  </div>
</template>

<style scoped>
.settings-page {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 22px 26px 28px;
  max-width: 900px;
  margin: 0 auto;
}

.settings-heading h1 {
  margin: 0;
  color: var(--text-primary);
  font-size: 22px;
}

.settings-heading p,
.setting-copy p {
  margin: 5px 0 0;
  color: var(--text-muted);
  font-size: 12px;
  line-height: 1.55;
}

.settings-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 24px;
  padding: 18px 20px;
  border-radius: var(--radius-lg);
}

.setting-copy { flex: 1; }
.setting-copy h2 { margin: 0; color: var(--text-primary); font-size: 14px; }
.setting-title-row { display: flex; align-items: center; gap: 8px; }
.setting-badge { padding: 2px 7px; border-radius: 8px; color: #f1c40f; background: rgba(241, 196, 15, .12); font-size: 9px; font-weight: 700; text-transform: uppercase; }
.setting-note { color: var(--text-secondary) !important; }

.switch { cursor: pointer; flex-shrink: 0; }
.switch input { position: absolute; opacity: 0; pointer-events: none; }
.switch-track { display: block; width: 46px; height: 25px; padding: 3px; border-radius: 14px; background: rgba(255,255,255,.1); border: 1px solid var(--border-default); transition: .2s ease; }
.switch-thumb { display: block; width: 17px; height: 17px; border-radius: 50%; background: var(--text-muted); transition: .2s ease; }
.switch input:checked + .switch-track { background: var(--accent); border-color: var(--accent); }
.switch input:checked + .switch-track .switch-thumb { transform: translateX(21px); background: white; }
.switch input:focus-visible + .switch-track { box-shadow: 0 0 0 3px var(--accent-soft); }

.api-card { align-items: flex-end; }
.api-controls { display: flex; flex-direction: column; gap: 8px; min-width: 245px; }
.settings-select, .refresh-button { width: 100%; border-radius: 8px; border: 1px solid var(--border-default); font: inherit; font-size: 12px; }
.settings-select { padding: 9px 10px; color: var(--text-primary); background: var(--bg-elevated); }
.settings-select option { background: #15151b; }
.refresh-button { padding: 9px 12px; color: white; background: var(--accent); cursor: pointer; }
.refresh-button:disabled { opacity: .5; cursor: wait; }
.last-check { display: inline-block; margin-top: 9px; color: var(--text-secondary); font-size: 11px; }

.safety-card { justify-content: flex-start; }
.safety-icon { display: grid; place-items: center; width: 32px; height: 32px; border-radius: 50%; background: rgba(46, 204, 113, .12); color: #2ecc71; font-weight: 800; flex-shrink: 0; }

@media (max-width: 700px) {
  .settings-card, .api-card { align-items: flex-start; }
  .api-card { flex-direction: column; }
  .api-controls { width: 100%; min-width: 0; }
}
</style>
