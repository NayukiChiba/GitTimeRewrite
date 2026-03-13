<script setup lang="ts">
defineProps<{
  repoPath: string
  statusText: string
  isLoading: boolean
  isRewriting: boolean
}>()

const emit = defineEmits<{
  (event: 'pick-repo'): void
}>()
</script>

<template>
  <div class="top-bar">
    <button
      class="btn btn-primary btn-large shadow-brand"
      :disabled="isLoading || isRewriting"
      @click="emit('pick-repo')"
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="20"
        height="20"
        fill="none"
        viewBox="0 0 24 24"
        stroke="currentColor"
        stroke-width="2"
        style="margin-right: 8px"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"
        />
      </svg>
      选择项目文件夹
    </button>
    <div class="repo-status">
      <div class="status-indicator" :class="{ active: repoPath }"></div>
      <div class="status-texts">
        <span class="repo-path" :title="repoPath">{{ repoPath || '未选择仓库' }}</span>
        <span class="status-msg" v-if="statusText">{{ statusText }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.top-bar {
  padding: 24px 32px;
  display: flex;
  align-items: center;
  gap: 24px;
  flex-shrink: 0;
}

.repo-status {
  display: flex;
  align-items: center;
  gap: 12px;
  background: #ffffff;
  padding: 10px 16px;
  border-radius: 12px;
  border: 1px solid #e2e8f0;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.02);
  flex: 1;
}

.status-indicator {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: #cbd5e1;
  box-shadow:
    inset 0 0 0 2px #fff,
    0 0 0 2px #cbd5e1;
}

.status-indicator.active {
  background: #10b981;
  box-shadow:
    inset 0 0 0 2px #fff,
    0 0 0 2px #10b981;
}

.status-texts {
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.repo-path {
  font-size: 14px;
  font-weight: 600;
  color: #1e293b;
}

.status-msg {
  font-size: 12px;
  color: #64748b;
  margin-top: 2px;
}

.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 10px 18px;
  border-radius: 10px;
  font-size: 13px;
  font-weight: 600;
  border: none;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-large {
  padding: 12px 24px;
  font-size: 14px;
  border-radius: 12px;
}

.btn-primary {
  background: #2563eb;
  color: #ffffff;
}

.btn-primary:not(:disabled):hover {
  background: #1d4ed8;
  box-shadow: 0 4px 12px rgba(37, 99, 235, 0.25);
}

.shadow-brand {
  box-shadow: 0 4px 12px rgba(37, 99, 235, 0.2);
}
</style>
