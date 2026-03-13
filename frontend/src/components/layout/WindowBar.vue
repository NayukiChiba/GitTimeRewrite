<script setup lang="ts">
const emit = defineEmits<{
  (event: 'minimize'): void
  (event: 'close'): void
  (event: 'drag'): void
}>()
</script>

<template>
  <header class="window-bar">
    <div class="window-dragger" @mousedown.left.prevent="emit('drag')">
      <div class="window-title">
        <svg
          class="app-icon"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
          stroke-width="2.5"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4"
          ></path>
        </svg>
        <span>GitTimeRewrite</span>
      </div>
    </div>
    <div class="window-actions">
      <button class="win-btn" @click="emit('minimize')">
        <svg width="12" height="12" viewBox="0 0 12 12">
          <path d="M1 6h10" stroke="#64748b" stroke-width="1.5" stroke-linecap="round" />
        </svg>
      </button>
      <button class="win-btn win-close" @click="emit('close')">
        <svg width="12" height="12" viewBox="0 0 12 12">
          <path d="M2 2l8 8M10 2L2 10" stroke="#64748b" stroke-width="1.5" stroke-linecap="round" />
        </svg>
      </button>
    </div>
  </header>
</template>

<style scoped>
.window-bar {
  height: 48px;
  background: #ffffff;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  user-select: none;
  border-bottom: 1px solid #e2e8f0;
  flex-shrink: 0;
}

.window-dragger {
  flex: 1;
  height: 100%;
  display: flex;
  align-items: center;
}

.window-title {
  font-size: 14px;
  font-weight: 600;
  color: #334155;
  display: flex;
  align-items: center;
  gap: 8px;
}

.app-icon {
  width: 18px;
  height: 18px;
  color: #2563eb;
}

.window-actions {
  display: flex;
  gap: 8px;
}

.win-btn {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.2s;
}

.win-btn:hover {
  background: #f1f5f9;
}

.win-close:hover {
  background: #fee2e2;
}

.win-close:hover svg path {
  stroke: #ef4444;
}
</style>
