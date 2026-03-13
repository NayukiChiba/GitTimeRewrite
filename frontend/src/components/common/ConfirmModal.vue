<script setup lang="ts">
defineProps<{
  visible: boolean
  title: string
  text: string
}>()

const emit = defineEmits<{
  (event: 'cancel'): void
  (event: 'confirm'): void
}>()
</script>

<template>
  <div v-if="visible" class="modal-mask">
    <div class="modal-card confirm-card">
      <div class="modal-title">{{ title }}</div>
      <div class="modal-text">{{ text }}</div>
      <div class="actions-row modal-actions">
        <button class="btn btn-text" @click="emit('cancel')">取消</button>
        <button class="btn btn-primary" @click="emit('confirm')">确认执行</button>
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
  z-index: 9999;
}

.modal-card {
  width: min(460px, calc(100vw - 40px));
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

.actions-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.modal-actions {
  margin-top: 18px;
  justify-content: flex-end;
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

.btn-primary {
  background: #2563eb;
  color: #ffffff;
}

.btn-primary:hover {
  background: #1d4ed8;
}

.btn-text {
  background: transparent;
  color: #64748b;
}

.btn-text:hover {
  background: #f1f5f9;
  color: #334155;
}
</style>
