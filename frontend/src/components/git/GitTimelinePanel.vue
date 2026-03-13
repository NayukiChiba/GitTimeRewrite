<script setup lang="ts">
import type { GitCommit } from '@/types/gitHistory'

type TimelineRow = {
  commit: GitCommit
  lane: number
  laneCount: number
  incomingLanes: number[]
  outgoingLanes: number[]
  edges: Array<{ from: number; to: number }>
  isMerge: boolean
}

const props = defineProps<{
  isLoading: boolean
  hasHistory: boolean
  commitsCount: number
  timelineRows: TimelineRow[]
  selectedCommitId: string
  laneColor: (index: number) => string
  formatDate: (text: string) => string
  hasPendingEdit: (commitId: string) => boolean
}>()

const emit = defineEmits<{
  (event: 'select-commit', commitId: string): void
}>()

function laneX(lane: number): number {
  return lane * 16 + 12
}

function edgePath(fromLane: number, toLane: number): string {
  const fromX = laneX(fromLane)
  const toX = laneX(toLane)
  const startY = 22
  const endY = 22
  const controlY = 34
  return `M ${fromX} ${startY} C ${fromX} ${controlY}, ${toX} ${controlY}, ${toX} ${endY}`
}

function graphWidth(laneCount: number): number {
  return laneCount * 16 + 24
}
</script>

<template>
  <div class="card col-left">
    <div class="card-head">
      <h2>Git 时间线</h2>
      <span class="chip">{{ commitsCount }} 条记录</span>
    </div>
    <div class="card-body scroll-y">
      <div v-if="isLoading" class="placeholder">正在读取 Git 历史...</div>
      <div v-else-if="!hasHistory" class="placeholder">请在上方选择包含 Git 的文件夹</div>
      <div v-else class="timeline-list">
        <button
          v-for="row in props.timelineRows"
          :key="row.commit.id"
          class="timeline-item"
          :class="{
            active: row.commit.id === selectedCommitId,
            pending: hasPendingEdit(row.commit.id),
          }"
          @click="emit('select-commit', row.commit.id)"
        >
          <div class="graph">
            <svg
              class="graph-svg"
              :width="graphWidth(row.laneCount)"
              height="44"
              aria-hidden="true"
            >
              <line
                v-for="lane in row.incomingLanes"
                :key="`line-in-${row.commit.id}-${lane}`"
                class="graph-line"
                :x1="laneX(lane)"
                :x2="laneX(lane)"
                :style="{ stroke: laneColor(lane) }"
                y1="-6"
                y2="22"
              />

              <line
                v-for="lane in row.outgoingLanes"
                :key="`line-out-${row.commit.id}-${lane}`"
                class="graph-line"
                :x1="laneX(lane)"
                :x2="laneX(lane)"
                :style="{ stroke: laneColor(lane) }"
                y1="22"
                y2="58"
              />

              <path
                v-for="(edge, edgeIndex) in row.edges"
                :key="`edge-${row.commit.id}-${edgeIndex}`"
                v-show="edge.from !== edge.to"
                class="graph-edge"
                :d="edgePath(edge.from, edge.to)"
                :style="{ stroke: laneColor(edge.from) }"
              />

              <circle
                class="graph-node"
                :cx="laneX(row.lane)"
                cy="22"
                r="6"
                :fill="laneColor(row.lane)"
              />
            </svg>
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
</template>

<style scoped>
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

.card-body {
  flex: 1;
  padding: 16px 20px;
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.scroll-y {
  overflow-y: auto;
}

.timeline-list {
  display: flex;
  flex-direction: column;
  gap: 0;
}

.timeline-item {
  display: flex;
  align-items: stretch;
  padding: 8px 14px;
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

.timeline-item.pending {
  border-color: #93c5fd;
}

.graph {
  position: relative;
  width: fit-content;
  min-width: 40px;
  flex-shrink: 0;
}

.graph-svg {
  display: block;
  overflow: visible;
}

.graph-line {
  stroke-width: 2;
  stroke-linecap: round;
  opacity: 0.95;
}

.graph-edge {
  stroke-width: 2;
  stroke-linecap: round;
  fill: none;
  opacity: 0.9;
}

.graph-node {
  stroke: rgba(255, 255, 255, 0.75);
  stroke-width: 1.5;
}

.timeline-content {
  flex: 1;
  min-width: 0;
  padding-top: 4px;
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

.placeholder {
  color: #94a3b8;
  text-align: center;
  margin-top: 40px;
  font-size: 14px;
}
</style>
