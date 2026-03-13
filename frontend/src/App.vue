<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { open } from '@tauri-apps/plugin-dialog'
import { getGitHistory, rewriteGitHistory } from '@/api/gitHistoryApi'
import { buildTimelineBatchEdits } from '@/utils/timelineScheduler'
import type { CommitEdit, GitCommit } from '@/types/gitHistory'
import BusyModal from '@/components/common/BusyModal.vue'
import ConfirmModal from '@/components/common/ConfirmModal.vue'
import GitTimelinePanel from '@/components/git/GitTimelinePanel.vue'
import RepoTopBar from '@/components/layout/RepoTopBar.vue'
import WindowBar from '@/components/layout/WindowBar.vue'
import OperationPanel from '@/components/panel/OperationPanel.vue'

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
const logs = ref<string[]>([])
const workspaceRef = ref<HTMLElement | null>(null)
const leftPaneWidth = ref(460)
const rightTab = ref<'manual' | 'batch' | 'logs'>('manual')

const batchForm = reactive({
  startDate: '2026-01-01',
  endDate: '2026-01-31',
  startTime: '08:00',
  endTime: '01:00',
  authorName: '',
  authorEmail: '',
})

const editMap = reactive<Record<string, CommitEdit>>({})

const isBusyModalVisible = ref(false)
const busyTitle = ref('')
const busyText = ref('')
const isConfirmModalVisible = ref(false)
const confirmTitle = ref('')
const confirmText = ref('')
const pendingEdits = ref<CommitEdit[]>([])

let unlistenLog: (() => void) | null = null
let resizeStartX = 0
let resizeStartWidth = 0
let isResizingColumns = false

const hasHistory = computed(() => commits.value.length > 0)
const manualEditCount = computed(() => Object.values(editMap).filter((edit) => hasPendingChanges(edit)).length)

const currentCommit = computed(() => commits.value.find((item) => item.id === selectedCommitId.value))

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
  const colors = ['#2f80ed', '#27ae60', '#f2994a', '#eb5757', '#9b51e0', '#56ccf2', '#219653', '#f2c94c']
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

function getOrCreateEdit(commitId: string): CommitEdit {
  if (!editMap[commitId]) {
    editMap[commitId] = { id: commitId }
  }
  return editMap[commitId]
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
      if (latestCommit) {
        selectCommit(latestCommit.id)
      }
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

    const edits = timelineEdits.map((timelineEdit) => ({
      ...timelineEdit,
      authorName: batchForm.authorName || undefined,
      authorEmail: batchForm.authorEmail || undefined,
      committerName: batchForm.authorName || undefined,
      committerEmail: batchForm.authorEmail || undefined,
    }))

    openConfirmModal(edits, '确认批量更改', `即将批量改写 ${edits.length} 条提交，是否继续？`)
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
    const result = await rewriteGitHistory({ repoPath: repoPath.value, edits })
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

function clearLogs() {
  logs.value = []
}

function resetEdits() {
  for (const key in editMap) {
    delete editMap[key]
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
  isResizingColumns = true
  resizeStartX = event.clientX
  resizeStartWidth = leftPaneWidth.value
  window.addEventListener('mousemove', handleResizing)
  window.addEventListener('mouseup', stopResizing)
}

function handleResizing(event: MouseEvent) {
  if (!isResizingColumns || !workspaceRef.value) {
    return
  }

  const workspaceWidth = workspaceRef.value.clientWidth
  const nextWidth = resizeStartWidth + (event.clientX - resizeStartX)
  leftPaneWidth.value = Math.max(320, Math.min(nextWidth, workspaceWidth - 440))
}

function stopResizing() {
  isResizingColumns = false
  window.removeEventListener('mousemove', handleResizing)
  window.removeEventListener('mouseup', stopResizing)
}

onMounted(async () => {
  unlistenLog = await listen<string>('log', (event) => {
    addLog(event.payload)
  })
})

onUnmounted(() => {
  unlistenLog?.()
  stopResizing()
})
</script>

<template>
  <div class="app-shell">
    <WindowBar @minimize="minimizeWindow" @close="closeWindow" @drag="startWindowDrag" />

    <main class="main-layout">
      <RepoTopBar
        :repo-path="repoPath"
        :status-text="statusText"
        :is-loading="isLoading"
        :is-rewriting="isRewriting"
        @pick-repo="pickRepoFolder"
      />

      <p v-if="errorText" class="error-banner">{{ errorText }}</p>

      <div ref="workspaceRef" class="workspace-grid" :style="{ '--left-pane-width': `${leftPaneWidth}px` }">
        <GitTimelinePanel
          :is-loading="isLoading"
          :has-history="hasHistory"
          :commits-count="commits.length"
          :timeline-rows="timelineRows"
          :selected-commit-id="selectedCommitId"
          :lane-color="laneColor"
          :format-date="formatDate"
          :has-pending-edit="hasPendingEdit"
          @select-commit="selectCommit"
        />

        <div class="resize-divider col-divider" @mousedown.prevent="startColumnsResize"></div>

        <OperationPanel
          v-model:right-tab="rightTab"
          :current-commit="currentCommit"
          :manual-edit-count="manualEditCount"
          :has-history="hasHistory"
          :is-loading="isLoading"
          :is-rewriting="isRewriting"
          :logs="logs"
          :batch-form="batchForm"
          :get-or-create-edit="getOrCreateEdit"
          :format-date="formatDate"
          @apply-manual="applyManualEdits"
          @reset-edits="resetEdits"
          @apply-batch="applyBatchTimelineEdits"
          @clear-logs="clearLogs"
        />
      </div>
    </main>

    <BusyModal :visible="isBusyModalVisible" :title="busyTitle" :text="busyText" />
    <ConfirmModal
      :visible="isConfirmModalVisible"
      :title="confirmTitle"
      :text="confirmText"
      @cancel="closeConfirmModal"
      @confirm="confirmRewrite"
    />
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

.main-layout {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.workspace-grid {
  flex: 1;
  display: grid;
  grid-template-columns: var(--left-pane-width, 460px) 8px minmax(420px, 1fr);
  gap: 0;
  padding: 0 32px 24px;
  overflow: hidden;
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
</style>
