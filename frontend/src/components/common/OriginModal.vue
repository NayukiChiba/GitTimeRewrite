<script setup lang="ts">
const props = withDefaults(
  defineProps<{
    visible: boolean
    currentOrigin?: string
    modelValue: string
  }>(),
  {
    currentOrigin: '',
  },
)

const emit = defineEmits<{
  (event: 'cancel'): void
  (event: 'confirm'): void
  (event: 'update:modelValue', value: string): void
}>()
</script>

<template>
  <div v-if="visible" class="modal-mask">
    <div class="modal-card">
      <div class="modal-icon">↗</div>
      <div class="modal-title">设置 Origin</div>
      <div class="modal-text" v-if="currentOrigin">当前 Origin: {{ currentOrigin }}</div>
      <div class="modal-text" v-else>当前未设置 Origin，请输入远程仓库地址</div>

      <label class="field">
        <span>Origin URL</span>
        <input
          :value="modelValue"
          placeholder="https://github.com/user/repo.git"
          @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
        />
      </label>

      <div class="actions-row modal-actions">
        <button class="btn btn-text" @click="emit('cancel')">取消</button>
        <button class="btn btn-primary" @click="emit('confirm')">保存Origin</button>
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
  width: min(560px, calc(100vw - 40px));
  background: #ffffff;
  border-radius: 16px;
  border: 1px solid #e2e8f0;
  box-shadow: 0 24px 56px rgba(15, 23, 42, 0.26);
  padding: 24px;
}

.modal-icon {
  width: 40px;
  height: 40px;
  margin: 0 auto 10px;
  border-radius: 999px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: #eff6ff;
  color: #1d4ed8;
  font-size: 20px;
  font-weight: 700;
}

.modal-title {
  font-size: 18px;
  font-weight: 700;
  color: #0f172a;
  text-align: center;
}

.modal-text {
  margin-top: 8px;
  font-size: 13px;
  color: #475569;
  text-align: center;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 16px;
}

.field span {
  font-size: 13px;
  font-weight: 600;
  color: #475569;
}

input {
  padding: 12px 14px;
  border-radius: 10px;
  border: 1px solid #cbd5e1;
  background: #f8fafc;
  font-size: 13px;
  color: #1e293b;
  font-family: inherit;
  transition: all 0.2s ease;
}

input:focus {
  outline: none;
  background: #ffffff;
  border-color: #2563eb;
  box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.15);
}

.actions-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.modal-actions {
  margin-top: 18px;
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
