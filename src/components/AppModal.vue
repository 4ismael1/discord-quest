<script setup lang="ts">
import { useTemplateRef, watch } from 'vue';

const props = defineProps<{
  open: boolean;
  title?: string;
}>();

const emit = defineEmits<{
  close: [];
}>();

const dialogRef = useTemplateRef<HTMLDialogElement>('dialogRef');

watch(() => props.open, (val) => {
  if (val) {
    dialogRef.value?.showModal();
  } else {
    dialogRef.value?.close();
  }
});

function handleBackdropClick(e: MouseEvent) {
  if (e.target === dialogRef.value) {
    emit('close');
  }
}
</script>

<template>
  <dialog ref="dialogRef" class="modal-dialog" @click="handleBackdropClick" @close="emit('close')">
    <div class="modal-content glass-elevated">
      <div v-if="title" class="modal-header">
        <h3 class="modal-title">{{ title }}</h3>
        <button class="modal-close" @click="emit('close')" aria-label="Close">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="currentColor">
            <path d="M3.5 1.5L7 5L10.5 1.5L12.5 3.5L9 7L12.5 10.5L10.5 12.5L7 9L3.5 12.5L1.5 10.5L5 7L1.5 3.5Z"/>
          </svg>
        </button>
      </div>
      <div class="modal-body">
        <slot />
      </div>
      <div class="modal-footer">
        <slot name="actions" />
      </div>
    </div>
  </dialog>
</template>

<style scoped>
.modal-dialog {
  position: fixed;
  inset: 0;
  width: 100%;
  height: 100%;
  max-width: 100%;
  max-height: 100%;
  border: none;
  background: transparent;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
}

.modal-dialog::backdrop {
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
}

.modal-dialog:not([open]) {
  display: none;
}

.modal-content {
  width: 100%;
  max-width: 420px;
  border-radius: var(--radius-lg);
  overflow: hidden;
  animation: modal-in 0.2s ease;
}

@keyframes modal-in {
  from {
    opacity: 0;
    transform: scale(0.96) translateY(8px);
  }
  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 18px 20px 0;
}

.modal-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary);
}

.modal-close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  border-radius: 6px;
  transition: all 0.15s ease;
}

.modal-close:hover {
  color: var(--text-primary);
  background: rgba(255, 255, 255, 0.06);
}

.modal-body {
  padding: 16px 20px;
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.6;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 0 20px 18px;
}
</style>
