<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, reactive, ref, watch } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { open } from '@tauri-apps/plugin-dialog'
import { getGitHistory, rewriteGitHistory } from '@/api/gitHistoryApi'
import { buildTimelineBatchEdits } from '@/utils/timelineScheduler'
import type { CommitEdit, GitCommit } from '@/types/gitHistory'

type TimelineRow = {
  commit: GitCommit
  lane: number
  laneCount: number
  isMerge: boolean
}

const EDITABLE_FIELDS: Array<keyof CommitEdit> = [
  'message',
  'authorName',
  'authorEmail',
  'authorDate',
  'committerDate',
  'committerName',
  'committerEmail',
]

const repoPath = ref('')
const commits = ref<GitCommit[]>([])
const selectedCommitId = ref('')
const isLoading = ref(false)
const isRewriting = ref(false)
const statusText = ref('请选择一个 Git 仓库')
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
const logs = ref<string[]>([])
const logContentRef = ref<HTMLElement | null>(null)
const workspaceRef = ref<HTMLElement | null>(null)
const leftPaneWidth = ref(460)
const rightTab = ref<'manual' | 'batch' | 'logs'>('manual')
const isBusyModalVisible = ref(false)
const busyTitle = ref('')
const busyText = ref('')
const isConfirmModalVisible = ref(false)
const confirmTitle = ref('')
const confirmText = ref('')
const pendingEdits = ref<CommitEdit[]>([])
let unlistenLog: (() => void) | null = null

type ResizeMode = 'none' | 'columns'

let resizeMode: ResizeMode = 'none'
let resizeStartX = 0
let resizeStartWidth = 0

const hasHistory = computed(() => commits.value.length > 0)
const manualEditCount = computed(
  () => Object.values(editMap).filter((edit) => hasPendingChanges(edit)).length,
)

const currentCommit = computed(() =>
  commits.value.find((item) => item.id === selectedCommitId.value),
)

const timelineRows = computed<TimelineRow[]>(() => {
  const latestFirst = [...commits.value].reverse()
  const lanes: string[] = []
  const rows: TimelineRow[] = []

  for (const commit of latestFirst) {
    let lane = lanes.indexOf(commit.id)
    if (lane < 0) {
      lane = lanes.length
      lanes.push(commit.id)
    }

    rows.push({
      commit,
      lane,
      laneCount: Math.min(Math.max(lanes.length, lane + 1), 8),
      isMerge: commit.parentIds.length > 1,
    })

    const firstParent = commit.parentIds[0]
    if (firstParent) {
      lanes[lane] = firstParent
    } else {
      lanes.splice(lane, 1)
    }

    for (const parent of commit.parentIds.slice(1)) {
      if (!lanes.includes(parent)) {
        lanes.push(parent)
      }
    }
  }

  return rows
})

function laneColor(index: number): string {
  const colors = [
    '#2f80ed',
    '#27ae60',
    '#f2994a',
    '#eb5757',
    '#9b51e0',
    '#56ccf2',
    '#219653',
    '#f2c94c',
  ]
  return colors[index % colors.length] ?? '#2f80ed'
}

function addLog(message: string) {
  const now = new Date().toLocaleTimeString('zh-CN', { hour12: false })
  logs.value.push(`[${now}] ${message}`)
}

function showBusyModal(title: string, text: string) {
  busyTitle.value = title
  busyText.value = text
  isBusyModalVisible.value = true
}

function hideBusyModal() {
  isBusyModalVisible.value = false
}

function openConfirmModal(edits: CommitEdit[], title: string, text: string) {
  pendingEdits.value = edits
  confirmTitle.value = title
  confirmText.value = text
  isConfirmModalVisible.value = true
}

function closeConfirmModal() {
  isConfirmModalVisible.value = false
  pendingEdits.value = []
}

function resetFeedback() {
  errorText.value = ''
}

function normalizeRepoPath(selectedPath: string) {
  repoPath.value = selectedPath.replace(/\\/g, '/')
}

function getOrCreateEdit(commitId: string): CommitEdit {
  if (!editMap[commitId]) {
    editMap[commitId] = { id: commitId }
  }
  return editMap[commitId]
}

function hasPendingChanges(edit?: CommitEdit): boolean {
  if (!edit) {
    return false
  }

  return EDITABLE_FIELDS.some((field) => {
    const value = edit[field]
    return typeof value === 'string' && value.trim().length > 0
  })
}

function hasPendingEdit(commitId: string): boolean {
  return hasPendingChanges(editMap[commitId])
}

function selectCommit(commitId: string) {
  selectedCommitId.value = commitId
  getOrCreateEdit(commitId)
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
    showBusyModal('正在加载 Git 历史', '正在读取提交记录，请稍候...')
    addLog(`开始加载仓库历史: ${repoPath.value}`)
    commits.value = await getGitHistory(repoPath.value)
    statusText.value = `已加载 ${commits.value.length} 条提交历史`

    if (commits.value.length > 0) {
      const latestCommit = commits.value[commits.value.length - 1]
      if (!latestCommit) {
        return
      }
      selectCommit(latestCommit.id)
    }
  } catch (error) {
    errorText.value = String(error)
    addLog(`加载失败: ${String(error)}`)
  } finally {
    isLoading.value = false
    hideBusyModal()
  }
}

async function applyManualEdits() {
  const edits = Object.values(editMap).filter((edit) => hasPendingChanges(edit))

  if (edits.length === 0) {
    errorText.value = '没有可提交的手动修改'
    return
  }

  openConfirmModal(edits, '确认提交信息更改', `即将改写 ${edits.length} 条提交，是否继续？`)
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

    openConfirmModal(
      batchEdits,
      '确认批量更改',
      `即将批量改写 ${batchEdits.length} 条提交，是否继续？`,
    )
  } catch (error) {
    errorText.value = String(error)
  }
}

async function confirmRewrite() {
  if (pendingEdits.value.length === 0) {
    closeConfirmModal()
    return
  }

  const edits = [...pendingEdits.value]
  closeConfirmModal()
  await executeRewrite(edits)
}

async function executeRewrite(edits: CommitEdit[]) {
  if (!repoPath.value) {
    errorText.value = '请先选择项目目录'
    return
  }

  try {
    isRewriting.value = true
    resetFeedback()
    showBusyModal('正在改写 Git 提交', `正在处理 ${edits.length} 条提交，请不要关闭应用...`)
    addLog(`开始改写 ${edits.length} 条提交...`)
    const result = await rewriteGitHistory({
      repoPath: repoPath.value,
      edits,
    })
    statusText.value = `改写完成，共处理 ${result.rewrittenCount} 条提交`
    addLog(`改写完成，共处理 ${result.rewrittenCount} 条提交`)
    resetEdits()
    await refreshHistory()
  } catch (error) {
    errorText.value = String(error)
    addLog(`改写失败: ${String(error)}`)
  } finally {
    isRewriting.value = false
    hideBusyModal()
  }
}

async function minimizeWindow() {
  await getCurrentWindow().minimize()
}

async function closeWindow() {
  await getCurrentWindow().close()
}

async function startWindowDrag() {
  await getCurrentWindow().startDragging()
}

function startColumnsResize(event: MouseEvent) {
  if (!workspaceRef.value) {
    return
  }
  resizeMode = 'columns'
  resizeStartX = event.clientX
  resizeStartWidth = leftPaneWidth.value
  window.addEventListener('mousemove', handleResizing)
  window.addEventListener('mouseup', stopResizing)
}

function handleResizing(event: MouseEvent) {
  if (resizeMode === 'none') {
    return
  }

  if (resizeMode === 'columns' && workspaceRef.value) {
    const workspaceWidth = workspaceRef.value.clientWidth
    const nextWidth = resizeStartWidth + (event.clientX - resizeStartX)
    leftPaneWidth.value = Math.max(320, Math.min(nextWidth, workspaceWidth - 440))
  }
}

function stopResizing() {
  resizeMode = 'none'
  window.removeEventListener('mousemove', handleResizing)
  window.removeEventListener('mouseup', stopResizing)
}

watch(
  () => logs.value.length,
  async () => {
    await nextTick()
    if (logContentRef.value) {
      logContentRef.value.scrollTop = logContentRef.value.scrollHeight
    }
  },
)

onMounted(async () => {
  unlistenLog = await listen<string>('log', (event) => {
    addLog(event.payload)
  })
})

onUnmounted(() => {
  unlistenLog?.()
  stopResizing()
})

function clearLogs() {
  logs.value = []
}

function resetEdits() {
  for (const key in editMap) {
    delete editMap[key]
  }
}
</script>

<template>
  <div class="app-shell">
    <header class="window-bar">
      <div class="window-dragger" @mousedown.left.prevent="startWindowDrag">
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
          <span>Git Time Rewrite</span>
        </div>
      </div>
      <div class="window-actions">
        <button class="win-btn" @click="minimizeWindow">
          <svg width="12" height="12" viewBox="0 0 12 12">
            <path d="M1 6h10" stroke="#64748b" stroke-width="1.5" stroke-linecap="round" />
          </svg>
        </button>
        <button class="win-btn win-close" @click="closeWindow">
          <svg width="12" height="12" viewBox="0 0 12 12">
            <path
              d="M2 2l8 8M10 2L2 10"
              stroke="#64748b"
              stroke-width="1.5"
              stroke-linecap="round"
            />
          </svg>
        </button>
      </div>
    </header>

    <main class="main-layout">
      <!-- Top Row: Select Folder -->
      <div class="top-bar">
        <button
          class="btn btn-primary btn-large shadow-brand"
          :disabled="isLoading || isRewriting"
          @click="pickRepoFolder"
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

      <p v-if="errorText" class="error-banner">{{ errorText }}</p>

      <div
        ref="workspaceRef"
        class="workspace-grid"
        :style="{ '--left-pane-width': `${leftPaneWidth}px` }"
      >
        <!-- Left: Timeline Grid (Native-like) -->
        <div class="card col-left">
          <div class="card-head">
            <h2>Git 时间线</h2>
            <span class="chip">{{ commits.length }} 条记录</span>
          </div>
          <div class="card-body scroll-y">
            <div v-if="isLoading" class="placeholder">正在读取 Git 历史...</div>
            <div v-else-if="!hasHistory" class="placeholder">请在上方选择包含 Git 的文件夹</div>
            <div v-else class="timeline-list">
              <button
                v-for="row in timelineRows"
                :key="row.commit.id"
                class="timeline-item"
                :class="{
                  active: row.commit.id === selectedCommitId,
                  pending: hasPendingEdit(row.commit.id),
                }"
                @click="selectCommit(row.commit.id)"
              >
                <div class="graph" :style="{ '--lane-count': `${row.laneCount}` }">
                  <span
                    v-for="lane in row.laneCount"
                    :key="`line-${row.commit.id}-${lane}`"
                    class="graph-line"
                    :style="{ left: `${(lane - 1) * 16 + 12}px` }"
                  />
                  <span
                    class="graph-node"
                    :style="{
                      left: `${row.lane * 16 + 8}px`,
                      backgroundColor: laneColor(row.lane),
                    }"
                  />
                </div>
                <div class="timeline-content">
                  <div class="timeline-top">
                    <span class="message">{{ row.commit.message }}</span>
                    <span v-if="hasPendingEdit(row.commit.id)" class="pending-badge">已修改</span>
                  </div>
                  <div class="timeline-bottom">
                    <span class="commit-id">{{ row.commit.id.slice(0, 8) }}</span>
                    <span class="meta-dot">·</span>
                    <span class="meta">{{ row.commit.authorName }}</span>
                    <span class="meta-dot">·</span>
                    <span class="meta date">{{ formatDate(row.commit.authorDate) }}</span>
                    <span v-if="row.isMerge" class="badge ml-auto">Merge</span>
                  </div>
                </div>
              </button>
            </div>
          </div>
        </div>

        <div class="resize-divider col-divider" @mousedown.prevent="startColumnsResize"></div>

        <!-- Right: Switching Panel -->
        <div class="col-right">
          <div class="card right-panel-card">
            <div class="card-head">
              <h2>操作面板</h2>
              <div class="tab-switch">
                <button
                  class="tab-btn"
                  :class="{ active: rightTab === 'manual' }"
                  @click="rightTab = 'manual'"
                >
                  信息更改
                </button>
                <button
                  class="tab-btn"
                  :class="{ active: rightTab === 'batch' }"
                  @click="rightTab = 'batch'"
                >
                  批量更改
                </button>
                <button
                  class="tab-btn"
                  :class="{ active: rightTab === 'logs' }"
                  @click="rightTab = 'logs'"
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
                  <em v-if="getOrCreateEdit(currentCommit!.id).message" class="field-mark"
                    >已改</em
                  ></span
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
                  @click="applyManualEdits"
                >
                  提交更改 ({{ manualEditCount }})
                </button>
                <button class="btn btn-text" :disabled="manualEditCount === 0" @click="resetEdits">
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
                  @click="applyBatchTimelineEdits"
                >
                  提交批量更改
                </button>
              </div>
            </div>

            <div v-else class="log-box" ref="logContentRef">
              <div class="log-toolbar">
                <button class="btn btn-text btn-xs" @click="clearLogs">清空</button>
              </div>
              <div v-if="logs.length === 0" class="log-empty">日志模块休眠中...</div>
              <div v-for="(item, index) in logs" :key="`${index}-${item}`" class="log-item">
                <span class="log-icon">›</span> <span class="log-text">{{ item }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </main>

    <div v-if="isBusyModalVisible" class="modal-mask">
      <div class="modal-card">
        <div class="modal-title">{{ busyTitle }}</div>
        <div class="modal-text">{{ busyText }}</div>
        <div class="modal-loading"></div>
      </div>
    </div>

    <div v-if="isConfirmModalVisible" class="modal-mask">
      <div class="modal-card confirm-card">
        <div class="modal-title">{{ confirmTitle }}</div>
        <div class="modal-text">{{ confirmText }}</div>
        <div class="actions-row modal-actions">
          <button class="btn btn-text" @click="closeConfirmModal">取消</button>
          <button class="btn btn-primary" @click="confirmRewrite">确认执行</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.app-shell {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background-color: #f7f9fb;
  color: #0f172a;
  overflow: hidden;
  font-family: inherit;
}

/* Top Bar */
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

/* Main Layout */
.main-layout {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

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

.workspace-grid {
  flex: 1;
  display: grid;
  grid-template-columns: var(--left-pane-width, 460px) 8px minmax(420px, 1fr);
  gap: 0;
  padding: 0 32px 24px;
  overflow: hidden;
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

.col-left {
  min-height: 0;
  margin-right: 12px;
}

.col-right {
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  margin-left: 12px;
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

.chip {
  background: #f1f5f9;
  color: #475569;
  padding: 4px 12px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 500;
}

.blue-chip {
  background: #eff6ff;
  color: #2563eb;
}

.card-body {
  flex: 1;
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.form-body {
  padding: 20px 24px;
  overflow: hidden;
}

.scroll-y {
  overflow-y: auto;
}

/* Timeline Left Layout */
.timeline-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.timeline-item {
  display: flex;
  align-items: stretch;
  padding: 12px 14px;
  border-radius: 10px;
  border: 1px solid transparent;
  background: transparent;
  cursor: pointer;
  transition: all 0.2s;
  text-align: left;
}

.timeline-item:hover {
  background: #f8fafc;
  border-color: #f1f5f9;
}

.timeline-item.active {
  background: #eff6ff;
  border-color: #bfdbfe;
}

.graph {
  position: relative;
  width: calc(var(--lane-count) * 16px + 12px);
  min-width: 40px;
  flex-shrink: 0;
}

.graph-line {
  position: absolute;
  top: -12px;
  bottom: -12px;
  width: 2px;
  background: #e2e8f0;
  border-radius: 2px;
}

.graph-node {
  position: absolute;
  top: 14px;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  border: 2px solid #ffffff;
  box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.06);
  z-index: 2;
}

.timeline-content {
  flex: 1;
  min-width: 0;
  padding-top: 8px;
}

.timeline-top {
  display: flex;
  justify-content: flex-start;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.message {
  font-size: 14px;
  font-weight: 500;
  color: #1e293b;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.timeline-item.active .message {
  color: #1d4ed8;
}

.timeline-item.pending {
  border-color: #93c5fd;
}

.pending-badge {
  margin-left: auto;
  background: #dbeafe;
  color: #1d4ed8;
  border-radius: 999px;
  padding: 2px 8px;
  font-size: 11px;
  font-weight: 700;
}

.timeline-bottom {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
}

.commit-id {
  font-family: 'JetBrains Mono', Consolas, monospace;
  color: #2563eb;
  background: #e5f0ff;
  padding: 0 6px;
  border-radius: 4px;
  font-weight: 600;
}

.meta {
  color: #64748b;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 110px;
}

.meta-dot {
  color: #cbd5e1;
}

.ml-auto {
  margin-left: auto;
}

.badge {
  background: #fef3c7;
  color: #b45309;
  padding: 2px 8px;
  border-radius: 6px;
  font-size: 11px;
  font-weight: 600;
}

/* Forms */
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

/* Buttons */
.actions-row {
  display: flex;
  align-items: center;
  gap: 12px;
}
.mt-4 {
  margin-top: 16px;
}
.mt-2 {
  margin-top: 8px;
}
.mr-1 {
  margin-right: 4px;
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

.btn-large {
  padding: 12px 24px;
  font-size: 14px;
  border-radius: 12px;
}

.btn-icon {
  /* Using standard padding */
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
  box-shadow: 0 4px 12px rgba(37, 99, 235, 0.25);
}

.bg-indigo {
  background: #4f46e5;
}
.bg-indigo:not(:disabled):hover {
  background: #4338ca;
  box-shadow: 0 4px 12px rgba(79, 70, 229, 0.25);
}

.shadow-brand {
  box-shadow: 0 4px 12px rgba(37, 99, 235, 0.2);
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

/* Logs */
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

.error-banner {
  margin: 0 32px 16px;
  padding: 12px 16px;
  border-radius: 10px;
  background: #fef2f2;
  color: #ef4444;
  border: 1px solid #fecaca;
  font-weight: 500;
  font-size: 13px;
}

.resize-divider {
  position: relative;
  flex-shrink: 0;
}

.col-divider {
  width: 8px;
  cursor: col-resize;
}

.col-divider::before {
  content: '';
  position: absolute;
  left: 3px;
  top: 8px;
  bottom: 8px;
  width: 2px;
  background: #dbe4ef;
  border-radius: 2px;
}

.col-divider:hover::before {
  background: #93c5fd;
}

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

.confirm-card {
  width: min(460px, calc(100vw - 40px));
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

.modal-actions {
  margin-top: 18px;
  justify-content: flex-end;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

/* Scrollbars */
::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}
::-webkit-scrollbar-track {
  background: transparent;
}
::-webkit-scrollbar-thumb {
  background: #cbd5e1;
  border-radius: 10px;
}
::-webkit-scrollbar-thumb:hover {
  background: #94a3b8;
}
</style>
