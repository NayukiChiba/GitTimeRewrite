<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref } from 'vue'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { open } from '@tauri-apps/plugin-dialog'
import {
  forcePushOrigin,
  getGitOrigin,
  getGitHistory,
  rewriteGitHistory,
  setGitOrigin,
} from '@/api/gitHistoryApi'
import { buildTimelineBatchEdits } from '@/utils/timelineScheduler'
import type { CommitEdit, GitCommit } from '@/types/gitHistory'
import BusyModal from '@/components/common/BusyModal.vue'
import ConfirmModal from '@/components/common/ConfirmModal.vue'
import ErrorModal from '@/components/common/ErrorModal.vue'
import OriginModal from '@/components/common/OriginModal.vue'
import GitTimelinePanel from '@/components/git/GitTimelinePanel.vue'
import RepoTopBar from '@/components/layout/RepoTopBar.vue'
import WindowBar from '@/components/layout/WindowBar.vue'
import OperationPanel from '@/components/panel/OperationPanel.vue'

type TimelineRow = {
  commit: GitCommit
  lane: number
  laneCount: number
  incomingLanes: number[]
  outgoingLanes: number[]
  edges: Array<{ from: number; to: number }>
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
const originUrl = ref('')
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
const busyLogStartIndex = ref<number | null>(null)
const isConfirmModalVisible = ref(false)
const confirmTitle = ref('')
const confirmText = ref('')
const confirmButtonText = ref('确认执行')
const confirmVariant = ref<'default' | 'danger'>('default')
const confirmMode = ref<'rewrite' | 'forcePush'>('rewrite')
const pendingEdits = ref<CommitEdit[]>([])
const isErrorModalVisible = ref(false)
const errorModalTitle = ref('操作失败')
const errorModalText = ref('')
const isOriginModalVisible = ref(false)
const originInput = ref('')

let unlistenLog: (() => void) | null = null
let resizeStartX = 0
let resizeStartWidth = 0
let isResizingColumns = false

const hasHistory = computed(() => commits.value.length > 0)
const busyLogs = computed(() => {
  if (busyLogStartIndex.value === null) {
    return []
  }
  return logs.value.slice(busyLogStartIndex.value)
})
const manualEditCount = computed(
  () => Object.values(editMap).filter((edit) => hasPendingChanges(edit)).length,
)

const currentCommit = computed(() =>
  commits.value.find((item) => item.id === selectedCommitId.value),
)

const timelineRows = computed<TimelineRow[]>(() => {
  const latestFirst = [...commits.value].reverse()
  const lanes: Array<string | null> = []
  const rows: TimelineRow[] = []

  const findLaneByCommit = (commitId: string): number =>
    lanes.findIndex((value) => value === commitId)

  const allocateLane = (): number => {
    const freeLane = lanes.findIndex((value) => value === null)
    if (freeLane >= 0) {
      return freeLane
    }
    lanes.push(null)
    return lanes.length - 1
  }

  const collapseMergedLanes = () => {
    const seen = new Set<string>()
    for (let index = 0; index < lanes.length; index += 1) {
      const laneCommit = lanes[index]
      if (!laneCommit) {
        continue
      }
      if (seen.has(laneCommit)) {
        lanes[index] = null
        continue
      }
      seen.add(laneCommit)
    }
  }

  const trimTrailingEmptyLanes = () => {
    while (lanes.length > 0 && lanes[lanes.length - 1] === null) {
      lanes.pop()
    }
  }

  const getActiveLanes = (): number[] =>
    lanes.map((value, index) => (value ? index : -1)).filter((index) => index >= 0)

  for (const commit of latestFirst) {
    let lane = findLaneByCommit(commit.id)
    if (lane < 0) {
      lane = allocateLane()
      lanes[lane] = commit.id
    }

    const incomingLanes = getActiveLanes()

    const edges: Array<{ from: number; to: number }> = []

    const firstParent = commit.parentIds[0]
    if (firstParent) {
      const firstParentLane = findLaneByCommit(firstParent)
      edges.push({
        from: lane,
        to: firstParentLane >= 0 ? firstParentLane : lane,
      })
      lanes[lane] = firstParent
    } else {
      lanes[lane] = null
    }

    for (const parent of commit.parentIds.slice(1)) {
      let parentLane = findLaneByCommit(parent)
      if (parentLane < 0) {
        parentLane = allocateLane()
        lanes[parentLane] = parent
      }
      edges.push({ from: lane, to: parentLane })
    }

    collapseMergedLanes()
    trimTrailingEmptyLanes()

    const outgoingLanes = getActiveLanes()

    const edgeMaxLane = edges.reduce((maxLane, edge) => Math.max(maxLane, edge.from, edge.to), lane)
    const incomingMaxLane = incomingLanes.length > 0 ? Math.max(...incomingLanes) : 0
    const outgoingMaxLane = outgoingLanes.length > 0 ? Math.max(...outgoingLanes) : 0
    rows.push({
      commit,
      lane,
      laneCount: Math.max(lanes.length, edgeMaxLane + 1, incomingMaxLane + 1, outgoingMaxLane + 1),
      incomingLanes,
      outgoingLanes,
      edges,
      isMerge: commit.parentIds.length > 1,
    })
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
  busyLogStartIndex.value = null
}

function openConfirmModal(
  edits: CommitEdit[],
  title: string,
  text: string,
  options?: {
    mode?: 'rewrite' | 'forcePush'
    buttonText?: string
    variant?: 'default' | 'danger'
  },
) {
  pendingEdits.value = edits
  confirmTitle.value = title
  confirmText.value = text
  confirmMode.value = options?.mode ?? 'rewrite'
  confirmButtonText.value = options?.buttonText ?? '确认执行'
  confirmVariant.value = options?.variant ?? 'default'
  isConfirmModalVisible.value = true
}

function closeConfirmModal() {
  isConfirmModalVisible.value = false
  pendingEdits.value = []
  confirmMode.value = 'rewrite'
  confirmButtonText.value = '确认执行'
  confirmVariant.value = 'default'
}

function resetFeedback() {
  errorText.value = ''
}

function extractReadableError(rawError: unknown): string {
  const rawText = String(rawError ?? '').trim()
  if (!rawText) {
    return '发生未知错误'
  }

  if (rawText.includes('git filter-branch')) {
    return '批量改写失败：请检查提交历史与时间参数，详细原因见日志'
  }

  if (rawText.includes('系统未安装 Git')) {
    return '系统未安装 Git，请先安装 Git 并重启应用'
  }

  if (rawText.includes('路径不是有效 Git 仓库')) {
    return '当前路径不是有效的 Git 仓库'
  }

  const stderrMarker = 'stderr:\n'
  const stderrIndex = rawText.indexOf(stderrMarker)
  const stderrText = stderrIndex >= 0 ? rawText.slice(stderrIndex + stderrMarker.length) : rawText

  const lines = stderrText
    .split('\n')
    .map((line) => line.trim())
    .filter(Boolean)

  const preferredLine = lines.find((line) =>
    /fatal:|error:|not found|failed|失败|无效|denied/i.test(line),
  )
  if (preferredLine) {
    return preferredLine
  }

  const compact = lines.slice(0, 2).join(' | ')
  if (compact.length > 180) {
    return `${compact.slice(0, 180)}...`
  }

  return compact || '操作失败，请查看日志详情'
}

function closeErrorModal() {
  isErrorModalVisible.value = false
}

function showErrorPopup(error: unknown, prefix: string) {
  const rawMessage = String(error)
  const readableMessage = `${prefix}: ${extractReadableError(error)}`
  errorText.value = readableMessage
  addLog(`${prefix}: ${rawMessage}`)
  errorModalTitle.value = prefix
  errorModalText.value = extractReadableError(error)
  isErrorModalVisible.value = true
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
    showErrorPopup('请先选择项目目录', '加载失败')
    return
  }

  try {
    isLoading.value = true
    resetFeedback()
    showBusyModal('正在加载 Git 历史', '正在读取提交记录，请稍候...')
    addLog(`开始加载仓库历史: ${repoPath.value}`)
    commits.value = await getGitHistory(repoPath.value)
    originUrl.value = await getGitOrigin(repoPath.value)
    statusText.value = `已加载 ${commits.value.length} 条提交历史`

    if (commits.value.length > 0) {
      const latestCommit = commits.value[commits.value.length - 1]
      if (latestCommit) {
        selectCommit(latestCommit.id)
      }
    }
  } catch (error) {
    showErrorPopup(error, '加载失败')
  } finally {
    isLoading.value = false
    hideBusyModal()
  }
}

async function applyManualEdits() {
  const edits = Object.values(editMap).filter((edit) => hasPendingChanges(edit))

  if (edits.length === 0) {
    showErrorPopup('没有可提交的手动修改', '提交失败')
    return
  }

  openConfirmModal(edits, '确认提交信息更改', `即将改写 ${edits.length} 条提交，是否继续？`)
}

async function applyBatchTimelineEdits() {
  if (!hasHistory.value) {
    showErrorPopup('当前没有提交历史可编辑', '批量编辑失败')
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
    showErrorPopup(error, '批量编辑失败')
  }
}

async function confirmAction() {
  if (confirmMode.value === 'forcePush') {
    closeConfirmModal()
    await executeForcePush()
    return
  }

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
    showErrorPopup('请先选择项目目录', '改写失败')
    return
  }

  try {
    isRewriting.value = true
    resetFeedback()
    busyLogStartIndex.value = logs.value.length
    showBusyModal('正在改写 Git 提交', `正在处理 ${edits.length} 条提交，请不要关闭应用...`)
    addLog(`开始改写 ${edits.length} 条提交...`)
    const result = await rewriteGitHistory({ repoPath: repoPath.value, edits })
    statusText.value = `改写完成，共处理 ${result.rewrittenCount} 条提交`
    addLog(`改写完成，共处理 ${result.rewrittenCount} 条提交`)
    resetEdits()
    await refreshHistory()
  } catch (error) {
    showErrorPopup(error, '改写失败')
  } finally {
    isRewriting.value = false
    hideBusyModal()
  }
}

async function setupOrigin() {
  if (!repoPath.value) {
    showErrorPopup('请先选择项目目录', '读取 Origin 失败')
    return
  }

  try {
    const currentOrigin = originUrl.value || (await getGitOrigin(repoPath.value))
    originUrl.value = currentOrigin
    originInput.value = currentOrigin
    isOriginModalVisible.value = true
  } catch (error) {
    showErrorPopup(error, '读取 Origin 失败')
  }
}

function cancelOriginModal() {
  isOriginModalVisible.value = false
}

async function saveOriginFromModal() {
  if (!repoPath.value) {
    showErrorPopup('请先选择项目目录', 'Origin 设置失败')
    return
  }

  const input = originInput.value.trim()
  if (!input) {
    showErrorPopup('Origin 地址不能为空', 'Origin 设置失败')
    return
  }

  try {
    isLoading.value = true
    isOriginModalVisible.value = false
    showBusyModal('正在设置 Origin', '正在写入远程仓库地址，请稍候...')
    const result = await setGitOrigin(repoPath.value, input)
    originUrl.value = result
    statusText.value = `Origin 已设置: ${result}`
    addLog(`Origin 设置完成: ${result}`)
  } catch (error) {
    showErrorPopup(error, 'Origin 设置失败')
  } finally {
    isLoading.value = false
    hideBusyModal()
  }
}

async function runForcePush() {
  if (!repoPath.value) {
    showErrorPopup('请先选择项目目录', 'Force push 失败')
    return
  }

  openConfirmModal(
    [],
    '确认 Force Push',
    '即将强制推送到 origin，可能覆盖远端提交历史，请确认操作。',
    {
      mode: 'forcePush',
      buttonText: '确认 Force Push',
      variant: 'danger',
    },
  )
}

async function executeForcePush() {
  try {
    isRewriting.value = true
    showBusyModal('正在执行 Force Push', '正在推送到 origin，请不要关闭应用...')
    const output = await forcePushOrigin(repoPath.value)
    statusText.value = 'Force push 完成'
    addLog(`Force push 完成: ${output}`)
  } catch (error) {
    showErrorPopup(error, 'Force push 失败')
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
        :origin-url="originUrl"
        :status-text="statusText"
        :is-loading="isLoading"
        :is-rewriting="isRewriting"
        @pick-repo="pickRepoFolder"
        @set-origin="setupOrigin"
        @force-push="runForcePush"
      />

      <p v-if="errorText" class="error-banner">{{ errorText }}</p>

      <div
        ref="workspaceRef"
        class="workspace-grid"
        :style="{ '--left-pane-width': `${leftPaneWidth}px` }"
      >
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

    <BusyModal :visible="isBusyModalVisible" :title="busyTitle" :text="busyText" :logs="busyLogs" />
    <ConfirmModal
      :visible="isConfirmModalVisible"
      :title="confirmTitle"
      :text="confirmText"
      :confirm-label="confirmButtonText"
      :variant="confirmVariant"
      @cancel="closeConfirmModal"
      @confirm="confirmAction"
    />
    <OriginModal
      v-model="originInput"
      :visible="isOriginModalVisible"
      :current-origin="originUrl"
      @cancel="cancelOriginModal"
      @confirm="saveOriginFromModal"
    />
    <ErrorModal
      :visible="isErrorModalVisible"
      :title="errorModalTitle"
      :text="errorModalText"
      @close="closeErrorModal"
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
