<script setup lang="ts">
defineProps<{
  visible: boolean
  title: string
  text: string
}>()

const emit = defineEmits<{
  (event: 'close'): void
}>()
</script>

<template>
  <div v-if="visible" class="modal-mask">
    <div class="modal-card">
      <div class="modal-icon">!</div>
      <div class="modal-title">{{ title }}</div>
      <div class="modal-text">{{ text }}</div>
      <div class="modal-actions">
        <button class="btn btn-danger" @click="emit('close')">我知道了</button>
      </div>
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
  z-index: 10000;
}

.modal-card {
  width: min(380px, calc(100vw - 28px));
  max-height: min(360px, calc(100vh - 32px));
  background: #ffffff;
  border-radius: 16px;
  border: 1px solid #fecaca;
  box-shadow: 0 24px 56px rgba(15, 23, 42, 0.26);
  padding: 18px;
  text-align: center;
  display: flex;
  flex-direction: column;
}

.modal-icon {
  width: 40px;
  height: 40px;
  margin: 0 auto 10px;
  border-radius: 999px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: #fef2f2;
  color: #dc2626;
  font-size: 20px;
  font-weight: 700;
}

.modal-title {
  font-size: 16px;
  font-weight: 700;
  color: #7f1d1d;
}

.modal-text {
  margin-top: 8px;
  font-size: 13px;
  color: #991b1b;
  white-space: pre-wrap;
  word-break: break-word;
  overflow-y: auto;
  max-height: 160px;
  text-align: left;
  padding-right: 4px;
}

.modal-actions {
  margin-top: 18px;
  display: flex;
  justify-content: center;
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

.btn-danger {
  background: #dc2626;
  color: #ffffff;
}

.btn-danger:hover {
  background: #b91c1c;
}
</style>
