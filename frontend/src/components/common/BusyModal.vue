<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window'
import { nextTick, ref, watch } from 'vue'

const props = defineProps<{
  visible: boolean
  title: string
  text: string
  logs?: string[]
}>()

const logRef = ref<HTMLElement | null>(null)

async function handleMaskMouseDown(event: MouseEvent) {
  if (event.button !== 0) {
    return
  }

  const target = event.target as HTMLElement | null
  if (target?.closest('.modal-card')) {
    return
  }

  await getCurrentWindow().startDragging()
}

async function handleTitleMouseDown(event: MouseEvent) {
  if (event.button !== 0) {
    return
  }
  await getCurrentWindow().startDragging()
}

watch(
  () => props.logs?.length ?? 0,
  async () => {
    await nextTick()
    if (logRef.value) {
      logRef.value.scrollTop = logRef.value.scrollHeight
    }
  },
)
</script>

<template>
  <div v-if="visible" class="modal-mask" @mousedown="handleMaskMouseDown">
    <div class="modal-card">
      <div class="modal-title" @mousedown.stop="handleTitleMouseDown">{{ title }}</div>
      <div class="modal-text">{{ text }}</div>
      <div v-if="logs && logs.length > 0" ref="logRef" class="modal-log-box">
        <div v-for="(item, idx) in logs" :key="`${idx}-${item}`" class="modal-log-item">
          {{ item }}
        </div>
      </div>
      <div class="modal-loading"></div>
    </div>
  </div>
</template>

<style scoped>
.modal-mask {
  position: fixed;
  inset: 0;
  background: rgba(15, 23, 42, 0.3);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
}

.modal-card {
  width: min(520px, calc(100vw - 40px));
  background: #ffffff;
  border-radius: 14px;
  border: 1px solid #e2e8f0;
  box-shadow: 0 18px 48px rgba(15, 23, 42, 0.2);
  padding: 20px;
}

.modal-title {
  font-size: 16px;
  font-weight: 700;
  color: #0f172a;
}

.modal-text {
  margin-top: 8px;
  font-size: 13px;
  color: #475569;
}

.modal-log-box {
  margin-top: 12px;
  max-height: 140px;
  overflow-y: auto;
  border: 1px solid #e2e8f0;
  border-radius: 10px;
  padding: 8px 10px;
  background: #f8fafc;
}

.modal-log-item {
  font-size: 12px;
  color: #334155;
  line-height: 1.5;
  font-family: 'JetBrains Mono', Consolas, monospace;
  word-break: break-word;
}

.modal-log-item + .modal-log-item {
  margin-top: 4px;
}

.modal-loading {
  width: 28px;
  height: 28px;
  border-radius: 50%;
  border: 3px solid #dbeafe;
  border-top-color: #2563eb;
  animation: spin 0.8s linear infinite;
  margin-top: 16px;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
