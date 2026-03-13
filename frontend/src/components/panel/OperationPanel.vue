<script setup lang="ts">
import { nextTick, ref, watch } from 'vue'
import type { CommitEdit, GitCommit } from '@/types/gitHistory'

const props = defineProps<{
  rightTab: 'manual' | 'batch' | 'logs'
  currentCommit?: GitCommit
  manualEditCount: number
  hasHistory: boolean
  isLoading: boolean
  isRewriting: boolean
  logs: string[]
  batchForm: {
    startDate: string
    endDate: string
    startTime: string
    endTime: string
    authorName: string
    authorEmail: string
  }
  getOrCreateEdit: (commitId: string) => CommitEdit
  formatDate: (text: string) => string
}>()

const emit = defineEmits<{
  (event: 'update:rightTab', value: 'manual' | 'batch' | 'logs'): void
  (event: 'apply-manual'): void
  (event: 'reset-edits'): void
  (event: 'apply-batch'): void
  (event: 'clear-logs'): void
}>()

const logContentRef = ref<HTMLElement | null>(null)

watch(
  () => props.logs.length,
  async () => {
    await nextTick()
    if (logContentRef.value) {
      logContentRef.value.scrollTop = logContentRef.value.scrollHeight
    }
  },
)
</script>

<template>
  <div class="col-right">
    <div class="card right-panel-card">
      <div class="card-head">
        <h2>操作面板</h2>
        <div class="tab-switch">
          <button
            class="tab-btn"
            :class="{ active: rightTab === 'manual' }"
            @click="emit('update:rightTab', 'manual')"
          >
            信息更改
          </button>
          <button
            class="tab-btn"
            :class="{ active: rightTab === 'batch' }"
            @click="emit('update:rightTab', 'batch')"
          >
            批量更改
          </button>
          <button
            class="tab-btn"
            :class="{ active: rightTab === 'logs' }"
            @click="emit('update:rightTab', 'logs')"
          >
            日志
          </button>
        </div>
      </div>

      <div v-if="rightTab === 'manual' && !currentCommit" class="placeholder small">
        请在左侧时间线选择一条提交记录进行编辑
      </div>
      <div v-else-if="rightTab === 'manual'" class="form-body">
        <div class="panel-meta">当前提交: {{ currentCommit!.id.slice(0, 8) }}</div>
        <label class="field full">
          <span
            >提交说明 (Message)
            <em v-if="getOrCreateEdit(currentCommit!.id).message" class="field-mark">已改</em></span
          >
          <textarea
            v-model="getOrCreateEdit(currentCommit!.id).message"
            :placeholder="currentCommit!.message"
            rows="2"
          />
        </label>
        <div class="form-grid">
          <label class="field">
            <span
              >作者名称
              <em v-if="getOrCreateEdit(currentCommit!.id).authorName" class="field-mark"
                >已改</em
              ></span
            >
            <input
              v-model="getOrCreateEdit(currentCommit!.id).authorName"
              :placeholder="currentCommit!.authorName"
            />
          </label>
          <label class="field">
            <span
              >作者邮箱
              <em v-if="getOrCreateEdit(currentCommit!.id).authorEmail" class="field-mark"
                >已改</em
              ></span
            >
            <input
              v-model="getOrCreateEdit(currentCommit!.id).authorEmail"
              :placeholder="currentCommit!.authorEmail"
            />
          </label>
          <label class="field">
            <span
              >作者时间
              <em v-if="getOrCreateEdit(currentCommit!.id).authorDate" class="field-mark"
                >已改</em
              ></span
            >
            <input
              v-model="getOrCreateEdit(currentCommit!.id).authorDate"
              :placeholder="formatDate(currentCommit!.authorDate)"
            />
          </label>
          <label class="field">
            <span
              >提交时间
              <em v-if="getOrCreateEdit(currentCommit!.id).committerDate" class="field-mark"
                >已改</em
              ></span
            >
            <input
              v-model="getOrCreateEdit(currentCommit!.id).committerDate"
              :placeholder="formatDate(currentCommit!.committerDate)"
            />
          </label>
        </div>
        <div class="actions-row mt-4">
          <button
            class="btn btn-primary"
            :disabled="isLoading || isRewriting || manualEditCount === 0"
            @click="emit('apply-manual')"
          >
            提交更改 ({{ manualEditCount }})
          </button>
          <button
            class="btn btn-text"
            :disabled="manualEditCount === 0"
            @click="emit('reset-edits')"
          >
            撤销修改
          </button>
        </div>
      </div>

      <div v-else-if="rightTab === 'batch'" class="form-body">
        <div class="card-desc">将所有提交均匀分配到时间窗口内</div>
        <div class="form-grid compact">
          <label class="field">
            <span>分配开始日期</span>
            <input v-model="batchForm.startDate" type="date" />
          </label>
          <label class="field">
            <span>分配结束日期</span>
            <input v-model="batchForm.endDate" type="date" />
          </label>
          <label class="field">
            <span>每日活跃起点</span>
            <input v-model="batchForm.startTime" type="time" />
          </label>
          <label class="field">
            <span>每日活跃终点</span>
            <input v-model="batchForm.endTime" type="time" />
          </label>
          <label class="field">
            <span>批量作者名称 <span class="opt">(可选)</span></span>
            <input v-model="batchForm.authorName" placeholder="不填则保持原样" />
          </label>
          <label class="field">
            <span>批量作者邮箱 <span class="opt">(可选)</span></span>
            <input v-model="batchForm.authorEmail" placeholder="不填则保持原样" />
          </label>
        </div>
        <div class="actions-row mt-4">
          <button
            class="btn btn-primary bg-indigo"
            :disabled="isLoading || isRewriting || !hasHistory"
            @click="emit('apply-batch')"
          >
            提交批量更改
          </button>
        </div>
      </div>

      <div v-else class="log-box" ref="logContentRef">
        <div class="log-toolbar">
          <button class="btn btn-text btn-xs" @click="emit('clear-logs')">清空</button>
        </div>
        <div v-if="logs.length === 0" class="log-empty">日志模块休眠中...</div>
        <div v-for="(item, index) in logs" :key="`${index}-${item}`" class="log-item">
          <span class="log-icon">›</span> <span class="log-text">{{ item }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.col-right {
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  margin-left: 12px;
}

.card {
  background: #ffffff;
  border-radius: 16px;
  border: 1px solid #e2e8f0;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.04);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.right-panel-card {
  min-height: 0;
  height: 100%;
}

.card-head {
  padding: 16px 24px;
  border-bottom: 1px solid #f1f5f9;
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: #ffffff;
}

.card-head h2 {
  font-size: 16px;
  font-weight: 600;
  color: #0f172a;
  margin: 0;
}

.form-body {
  padding: 20px 24px;
  overflow: hidden;
}

.form-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.form-grid.compact {
  gap: 14px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 12px;
}

.field.full {
  margin-bottom: 16px;
}

.field span {
  font-size: 13px;
  font-weight: 600;
  color: #475569;
}

.field-mark {
  display: inline-block;
  margin-left: 6px;
  font-style: normal;
  font-size: 11px;
  color: #1d4ed8;
  background: #dbeafe;
  border-radius: 999px;
  padding: 1px 6px;
}

.opt {
  font-weight: 400;
  color: #94a3b8;
}

input,
textarea {
  padding: 12px 14px;
  border-radius: 10px;
  border: 1px solid #cbd5e1;
  background: #f8fafc;
  font-size: 13px;
  color: #1e293b;
  font-family: inherit;
  transition: all 0.2s ease;
}

input::placeholder,
textarea::placeholder {
  color: #94a3b8;
}

input:focus,
textarea:focus {
  outline: none;
  background: #ffffff;
  border-color: #2563eb;
  box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.15);
}

textarea {
  resize: vertical;
  min-height: 52px;
}

.actions-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.mt-4 {
  margin-top: 16px;
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

.btn-primary {
  background: #2563eb;
  color: #ffffff;
}

.btn-primary:not(:disabled):hover {
  background: #1d4ed8;
}

.bg-indigo {
  background: #4f46e5;
}

.bg-indigo:not(:disabled):hover {
  background: #4338ca;
}

.btn-text {
  background: transparent;
  color: #64748b;
}

.btn-text:hover {
  background: #f1f5f9;
  color: #334155;
}

.btn-xs {
  padding: 4px 12px;
  border-radius: 8px;
}

.card-desc {
  font-size: 13px;
  color: #64748b;
  margin-bottom: 12px;
}

.panel-meta {
  font-size: 12px;
  color: #2563eb;
  background: #eff6ff;
  border: 1px solid #bfdbfe;
  border-radius: 8px;
  padding: 6px 10px;
  margin-bottom: 12px;
  width: fit-content;
}

.tab-switch {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  background: #f1f5f9;
  padding: 4px;
  border-radius: 10px;
}

.tab-btn {
  border: none;
  background: transparent;
  color: #64748b;
  font-size: 12px;
  font-weight: 600;
  border-radius: 8px;
  padding: 6px 10px;
  cursor: pointer;
}

.tab-btn.active {
  background: #ffffff;
  color: #1d4ed8;
  box-shadow: 0 1px 3px rgba(15, 23, 42, 0.1);
}

.log-box {
  background: #1e293b;
  border-radius: 12px;
  padding: 16px;
  margin: 16px 20px 20px;
  flex: 1;
  overflow-y: auto;
  min-height: 100px;
  font-family: 'JetBrains Mono', Consolas, monospace;
  font-size: 12px;
  line-height: 1.6;
}

.log-toolbar {
  display: flex;
  justify-content: flex-end;
  margin-bottom: 8px;
}

.log-empty {
  color: #64748b;
  text-align: center;
  margin-top: 20px;
}

.log-item {
  color: #e2e8f0;
  margin-bottom: 6px;
  display: flex;
  gap: 10px;
}

.log-icon {
  color: #38bdf8;
  font-weight: bold;
}

.log-text {
  word-break: break-all;
}

.placeholder {
  color: #94a3b8;
  text-align: center;
  margin-top: 40px;
  font-size: 14px;
}

.placeholder.small {
  margin-top: 20px;
  margin-bottom: 20px;
}
</style>
