<script setup lang="ts">
defineProps<{
  visible: boolean
  title: string
  text: string
}>()
</script>

<template>
  <div v-if="visible" class="modal-mask">
    <div class="modal-card">
      <div class="modal-title">{{ title }}</div>
      <div class="modal-text">{{ text }}</div>
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
  width: min(420px, calc(100vw - 40px));
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
