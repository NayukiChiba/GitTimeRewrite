<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, reactive, ref, watch } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import { getGitHistory, rewriteGitHistory } from '@/api/gitHistoryApi'
import { buildTimelineBatchEdits } from '@/utils/timelineScheduler'
import type { CommitEdit, GitCommit } from '@/types/gitHistory'

const repoPath = ref('')
const commits = ref<GitCommit[]>([])
const isLoading = ref(false)
const isRewriting = ref(false)
const statusText = ref('等待选择仓库')
const errorText = ref('')

const batchForm = reactive({
  startDate: '2026-01-01',
  endDate: '2026-01-31',
  startTime: '08:00',
  endTime: '01:00',
  authorName: '',
  authorEmail: '',
})

const editMap = reactive<Record<string, CommitEdit>>({})

// 日志相关状态
const logs = ref<string[]>([])
const logContentRef = ref<HTMLElement | null>(null)
let unlistenLog: (() => void) | null = null

function addLog(message: string) {
  const now = new Date().toLocaleTimeString('zh-CN', { hour12: false })
  logs.value.push(`[${now}] ${message}`)
}

function clearLogs() {
  logs.value = []
}

// 日志内容变化时自动滚动到底部
watch(
  () => logs.value.length,
  async () => {
    await nextTick()
    if (logContentRef.value) {
      logContentRef.value.scrollTop = logContentRef.value.scrollHeight
    }
  },
)

// 监听后端发送的日志事件
onMounted(async () => {
  unlistenLog = await listen<string>('log', (event) => {
    addLog(event.payload)
  })
})

onUnmounted(() => {
  unlistenLog?.()
})

const hasHistory = computed(() => commits.value.length > 0)
const manualEditCount = computed(
  () =>
    Object.values(editMap).filter(
      (edit) =>
        !!edit.message ||
        !!edit.authorName ||
        !!edit.authorEmail ||
        !!edit.authorDate ||
        !!edit.committerDate ||
        !!edit.committerName ||
        !!edit.committerEmail,
    ).length,
)

const timeRangeText = computed(() => `${batchForm.startTime} -> ${batchForm.endTime}`)

function getOrCreateEdit(commitId: string): CommitEdit {
  if (!editMap[commitId]) {
    editMap[commitId] = { id: commitId }
  }
  return editMap[commitId]
}

function resetFeedback() {
  errorText.value = ''
}

function normalizeRepoPath(selectedPath: string) {
  repoPath.value = selectedPath.replace(/\\/g, '/')
}

function formatDate(dateText: string): string {
  const value = new Date(dateText)
  if (Number.isNaN(value.valueOf())) {
    return dateText
  }
  return value.toLocaleString()
}

async function pickRepoFolder() {
  resetFeedback()
  const selected = await open({
    directory: true,
    multiple: false,
    title: '选择 Git 项目目录',
  })

  if (typeof selected === 'string') {
    normalizeRepoPath(selected)
    await refreshHistory()
  }
}

async function refreshHistory() {
  if (!repoPath.value) {
    errorText.value = '请先选择项目目录'
    return
  }

  try {
    isLoading.value = true
    resetFeedback()
    addLog(`开始加载仓库历史: ${repoPath.value}`)
    commits.value = await getGitHistory(repoPath.value)
    Object.keys(editMap).forEach((key) => {
      delete editMap[key]
    })
    statusText.value = `已加载 ${commits.value.length} 条提交历史`
  } catch (error) {
    errorText.value = String(error)
    addLog(`加载失败: ${String(error)}`)
  } finally {
    isLoading.value = false
  }
}

async function applyManualEdits() {
  const edits = Object.values(editMap).filter(
    (edit) =>
      !!edit.message ||
      !!edit.authorName ||
      !!edit.authorEmail ||
      !!edit.authorDate ||
      !!edit.committerDate ||
      !!edit.committerName ||
      !!edit.committerEmail,
  )

  if (edits.length === 0) {
    errorText.value = '没有可提交的手动修改'
    return
  }

  await executeRewrite(edits)
}

async function applyBatchTimelineEdits() {
  if (!hasHistory.value) {
    errorText.value = '当前没有提交历史可编辑'
    return
  }

  try {
    const timelineEdits = buildTimelineBatchEdits(commits.value, {
      startDate: batchForm.startDate,
      endDate: batchForm.endDate,
      startTime: batchForm.startTime,
      endTime: batchForm.endTime,
    })

    const batchEdits = timelineEdits.map((timelineEdit) => ({
      ...timelineEdit,
      authorName: batchForm.authorName || undefined,
      authorEmail: batchForm.authorEmail || undefined,
      committerName: batchForm.authorName || undefined,
      committerEmail: batchForm.authorEmail || undefined,
    }))

    await executeRewrite(batchEdits)
  } catch (error) {
    errorText.value = String(error)
  }
}

async function executeRewrite(edits: CommitEdit[]) {
  if (!repoPath.value) {
    errorText.value = '请先选择项目目录'
    return
  }

  try {
    isRewriting.value = true
    resetFeedback()
    addLog(`开始改写 ${edits.length} 条提交...`)
    const result = await rewriteGitHistory({
      repoPath: repoPath.value,
      edits,
    })
    statusText.value = `改写完成，共处理 ${result.rewrittenCount} 条提交`
    addLog(`改写完成，共处理 ${result.rewrittenCount} 条提交`)
    await refreshHistory()
  } catch (error) {
    errorText.value = String(error)
    addLog(`改写失败: ${String(error)}`)
  } finally {
    isRewriting.value = false
  }
}
</script>

<template>
  <div class="app-shell">
    <header class="hero">
      <p class="tag">Git Time Rewrite</p>
      <h1>Git 历史时间编辑工作台</h1>
      <p class="desc">
        选择仓库后可查看提交、逐条编辑 message/作者/时间，并按连续日期规则批量重写时间线。
      </p>
    </header>

    <section class="toolbar panel" aria-label="仓库操作区">
      <div class="toolbar-grid">
        <button
          class="btn btn-primary"
          :disabled="isLoading || isRewriting"
          @click="pickRepoFolder"
        >
          选择仓库
        </button>
        <label class="field">
          <span>仓库路径</span>
          <input v-model="repoPath" placeholder="例如 D:/work/demo" />
        </label>
        <button
          class="btn btn-secondary"
          :disabled="isLoading || isRewriting"
          @click="refreshHistory"
        >
          刷新历史
        </button>
      </div>
      <p class="status">{{ statusText }}</p>
      <p v-if="errorText" class="error">{{ errorText }}</p>
    </section>

    <section class="ops-grid">
      <article class="panel" aria-label="批量编辑区">
        <div class="panel-head">
          <h2>批量时间线</h2>
          <span class="chip">窗口 {{ timeRangeText }}</span>
        </div>
        <div class="form-grid">
          <label class="field">
            <span>开始日期</span>
            <input v-model="batchForm.startDate" type="date" />
          </label>
          <label class="field">
            <span>结束日期</span>
            <input v-model="batchForm.endDate" type="date" />
          </label>
          <label class="field">
            <span>开始时间</span>
            <input v-model="batchForm.startTime" type="time" />
          </label>
          <label class="field">
            <span>结束时间（可跨天）</span>
            <input v-model="batchForm.endTime" type="time" />
          </label>
          <label class="field">
            <span>批量作者名（可选）</span>
            <input v-model="batchForm.authorName" />
          </label>
          <label class="field">
            <span>批量作者邮箱（可选）</span>
            <input v-model="batchForm.authorEmail" />
          </label>
        </div>
        <button
          class="btn btn-primary"
          :disabled="isLoading || isRewriting || !hasHistory"
          @click="applyBatchTimelineEdits"
        >
          批量改写时间线
        </button>
      </article>

      <article class="panel" aria-label="手动编辑区">
        <div class="panel-head">
          <h2>手动提交编辑</h2>
          <span class="chip">待提交 {{ manualEditCount }}</span>
        </div>
        <p class="hint">在下方表格修改后统一提交，可同时改 message、作者、时间。</p>
        <button
          class="btn btn-secondary"
          :disabled="isLoading || isRewriting || manualEditCount === 0"
          @click="applyManualEdits"
        >
          提交手动修改
        </button>
      </article>
    </section>

    <section class="panel" aria-label="提交历史列表区">
      <div class="panel-head">
        <h2>提交历史</h2>
        <span class="chip">总数 {{ commits.length }}</span>
      </div>

      <div v-if="isLoading" class="placeholder">正在读取 Git 历史...</div>
      <div v-else-if="!hasHistory" class="placeholder">尚未加载提交历史</div>

      <div v-else class="table-wrap">
        <table class="commit-table">
          <thead>
            <tr>
              <th>提交</th>
              <th>原 message</th>
              <th>新 message</th>
              <th>作者</th>
              <th>作者邮箱</th>
              <th>作者时间</th>
              <th>提交时间</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="commit in commits" :key="commit.id">
              <td class="mono">{{ commit.id.slice(0, 8) }}</td>
              <td class="text-col">{{ commit.message }}</td>
              <td>
                <input v-model="getOrCreateEdit(commit.id).message" placeholder="不填则不改" />
              </td>
              <td>
                <input
                  v-model="getOrCreateEdit(commit.id).authorName"
                  :placeholder="commit.authorName"
                />
              </td>
              <td>
                <input
                  v-model="getOrCreateEdit(commit.id).authorEmail"
                  :placeholder="commit.authorEmail"
                />
              </td>
              <td>
                <input
                  v-model="getOrCreateEdit(commit.id).authorDate"
                  :placeholder="formatDate(commit.authorDate)"
                />
              </td>
              <td>
                <input
                  v-model="getOrCreateEdit(commit.id).committerDate"
                  :placeholder="formatDate(commit.committerDate)"
                />
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>

    <section class="panel log-panel" aria-label="运行日志">
      <div class="panel-head">
        <h2>运行日志</h2>
        <button class="btn btn-secondary btn-xs" @click="clearLogs">清空</button>
      </div>
      <div ref="logContentRef" class="log-content">
        <p v-if="logs.length === 0" class="placeholder">暂无日志，操作后将显示运行记录</p>
        <p v-for="(entry, i) in logs" :key="i" class="log-entry">{{ entry }}</p>
      </div>
    </section>
  </div>
</template>

<style scoped>
.app-shell {
  min-height: 100vh;
  background:
    radial-gradient(circle at 10% 0%, rgba(34, 197, 94, 0.16), transparent 30%),
    radial-gradient(circle at 100% 20%, rgba(51, 65, 85, 0.45), transparent 28%),
    linear-gradient(160deg, #0f172a 0%, #101c2e 45%, #15253b 100%);
  color: #f8fafc;
  padding: 24px;
}

.hero {
  margin-bottom: 18px;
}

.tag {
  display: inline-flex;
  padding: 2px 8px;
  border-radius: 999px;
  background: rgba(34, 197, 94, 0.18);
  color: #86efac;
  font-family: 'JetBrains Mono', 'Consolas', monospace;
  font-size: 12px;
}

h1,
h2 {
  margin: 10px 0;
}

h1 {
  font-size: 34px;
  font-weight: 700;
  line-height: 1.2;
}

h2 {
  font-size: 22px;
  font-weight: 600;
}

.desc {
  color: #cbd5e1;
}

.panel {
  border: 1px solid rgba(148, 163, 184, 0.22);
  border-radius: 16px;
  background: rgba(15, 23, 42, 0.78);
  box-shadow: 0 18px 40px rgba(2, 6, 23, 0.35);
  backdrop-filter: blur(8px);
  padding: 16px;
  margin-bottom: 14px;
}

.toolbar-grid {
  display: grid;
  grid-template-columns: 130px 1fr 120px;
  gap: 12px;
}

.panel-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 10px;
}

.chip {
  display: inline-flex;
  align-items: center;
  border: 1px solid rgba(148, 163, 184, 0.35);
  border-radius: 999px;
  padding: 2px 10px;
  font-size: 12px;
  color: #cbd5e1;
}

.form-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
  margin: 12px 0;
}

.ops-grid {
  display: grid;
  grid-template-columns: 1.4fr 1fr;
  gap: 14px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.field span {
  color: #94a3b8;
  font-size: 12px;
}

input {
  width: 100%;
  border: 1px solid rgba(100, 116, 139, 0.45);
  border-radius: 10px;
  background: rgba(30, 41, 59, 0.72);
  color: #f8fafc;
  padding: 8px 10px;
  transition: border-color 0.2s ease;
}

input::placeholder {
  color: #94a3b8;
}

input:focus {
  border-color: #22c55e;
  outline: none;
}

.btn {
  cursor: pointer;
  border: none;
  border-radius: 10px;
  padding: 10px 12px;
  font-weight: 600;
  transition:
    transform 0.2s ease,
    opacity 0.2s ease,
    filter 0.2s ease;
}

.btn:hover:not(:disabled) {
  transform: translateY(-1px);
  filter: brightness(1.06);
}

.btn:disabled {
  opacity: 0.52;
  cursor: not-allowed;
}

.btn-primary {
  color: #052e16;
  background: linear-gradient(120deg, #22c55e 0%, #4ade80 100%);
}

.btn-secondary {
  color: #e2e8f0;
  background: linear-gradient(120deg, #334155 0%, #475569 100%);
}

.status {
  margin-top: 10px;
  color: #bbf7d0;
}

.error {
  margin-top: 6px;
  color: #fca5a5;
}

.hint,
.placeholder {
  color: #cbd5e1;
}

.table-wrap {
  overflow: auto;
  border-radius: 12px;
  border: 1px solid rgba(100, 116, 139, 0.3);
}

.commit-table {
  width: 100%;
  border-collapse: collapse;
  min-width: 1180px;
}

.commit-table th,
.commit-table td {
  border-bottom: 1px solid rgba(100, 116, 139, 0.22);
  padding: 8px;
  vertical-align: top;
  font-size: 13px;
}

.commit-table th {
  text-align: left;
  background: rgba(30, 41, 59, 0.72);
  position: sticky;
  top: 0;
  z-index: 1;
}

.commit-table tr:hover {
  background: rgba(34, 197, 94, 0.1);
}

.text-col {
  min-width: 220px;
}

.mono {
  font-family: 'JetBrains Mono', 'Consolas', monospace;
}

@media (max-width: 1100px) {
  .toolbar-grid {
    grid-template-columns: 1fr;
  }

  .ops-grid {
    grid-template-columns: 1fr;
  }

  .form-grid {
    grid-template-columns: 1fr;
  }

  h1 {
    font-size: 28px;
  }
}

@media (prefers-reduced-motion: reduce) {
  .btn,
  input {
    transition: none;
  }
}

/* 日志面板 */
.log-panel {
  margin-top: 0;
}

.log-content {
  height: 160px;
  overflow-y: auto;
  border-radius: 10px;
  border: 1px solid rgba(100, 116, 139, 0.3);
  background: rgba(2, 6, 23, 0.72);
  padding: 10px 12px;
  margin-top: 10px;
}

.log-entry {
  font-family: 'JetBrains Mono', 'Consolas', monospace;
  font-size: 12px;
  color: #86efac;
  margin: 2px 0;
  white-space: pre-wrap;
  word-break: break-all;
}

.btn-xs {
  padding: 4px 10px;
  font-size: 12px;
}
</style>
