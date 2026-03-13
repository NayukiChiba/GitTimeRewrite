<script setup lang="ts">
defineProps<{
  repoPath: string
  statusText: string
  isLoading: boolean
  isRewriting: boolean
}>()

const emit = defineEmits<{
  (event: 'pick-repo'): void
  (event: 'set-origin'): void
  (event: 'force-push'): void
}>()
</script>

<template>
  <div class="top-bar">
    <div class="action-group">
      <button
        class="btn btn-primary btn-compact shadow-brand"
        :disabled="isLoading || isRewriting"
        @click="emit('pick-repo')"
      >
        选择文件夹
      </button>
      <button
        class="btn btn-outline"
        :disabled="isLoading || isRewriting || !repoPath"
        @click="emit('set-origin')"
      >
        设置Origin
      </button>
      <button
        class="btn btn-danger"
        :disabled="isLoading || isRewriting || !repoPath"
        @click="emit('force-push')"
      >
        ForcePush
      </button>
    </div>
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
  gap: 16px;
  flex-shrink: 0;
}

.action-group {
  display: flex;
  align-items: center;
  gap: 8px;
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

.btn-compact {
  width: 132px;
  padding: 10px 12px;
  font-size: 13px;
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

.btn-outline {
  background: #ffffff;
  color: #2563eb;
  border: 1px solid #93c5fd;
}

.btn-outline:not(:disabled):hover {
  background: #eff6ff;
}

.btn-danger {
  background: #ef4444;
  color: #ffffff;
}

.btn-danger:not(:disabled):hover {
  background: #dc2626;
}
</style>
