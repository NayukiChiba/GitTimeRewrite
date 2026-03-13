import type { CommitEdit, GitCommit, TimelineBatchInput } from '@/types/gitHistory'

const ONE_DAY_MINUTES = 24 * 60

function parseTimeToMinutes(timeText: string): number {
  const [hourText, minuteText] = timeText.split(':')
  const hour = Number(hourText)
  const minute = Number(minuteText)

  if (
    Number.isNaN(hour) ||
    Number.isNaN(minute) ||
    hour < 0 ||
    hour > 23 ||
    minute < 0 ||
    minute > 59
  ) {
    throw new Error(`非法时间格式: ${timeText}`)
  }

  return hour * 60 + minute
}

function toIsoLocal(date: Date): string {
  const timeOffsetMs = date.getTimezoneOffset() * 60 * 1000
  return new Date(date.getTime() - timeOffsetMs).toISOString().replace('Z', '')
}

function randomInt(min: number, max: number): number {
  return Math.floor(Math.random() * (max - min + 1)) + min
}

export function buildTimelineBatchEdits(
  commits: GitCommit[],
  input: TimelineBatchInput,
): CommitEdit[] {
  if (commits.length === 0) {
    return []
  }

  const startDate = new Date(`${input.startDate}T00:00:00`)
  const endDate = new Date(`${input.endDate}T00:00:00`)
  if (Number.isNaN(startDate.valueOf()) || Number.isNaN(endDate.valueOf())) {
    throw new Error('开始日期或结束日期无效')
  }
  if (endDate < startDate) {
    throw new Error('结束日期不能早于开始日期')
  }

  const startMinutes = parseTimeToMinutes(input.startTime)
  const endMinutes = parseTimeToMinutes(input.endTime)
  const duration =
    endMinutes >= startMinutes
      ? endMinutes - startMinutes
      : endMinutes + ONE_DAY_MINUTES - startMinutes

  const totalDays = Math.floor((endDate.getTime() - startDate.getTime()) / (24 * 60 * 60 * 1000))
  const totalCommits = commits.length

  let previousTimestamp = 0
  return commits.map((commit, index) => {
    const dayOffset = totalCommits === 1 ? 0 : Math.floor((index * totalDays) / (totalCommits - 1))
    const targetDate = new Date(startDate)
    targetDate.setDate(startDate.getDate() + dayOffset)

    const minuteOffset = randomInt(0, Math.max(1, duration))
    const targetDateTime = new Date(targetDate)
    targetDateTime.setHours(0, 0, 0, 0)
    targetDateTime.setMinutes(startMinutes + minuteOffset)

    // 确保时间线绝对递增，避免后续改写时出现逆序。
    if (targetDateTime.getTime() <= previousTimestamp) {
      targetDateTime.setTime(previousTimestamp + randomInt(60 * 1000, 5 * 60 * 1000))
    }
    previousTimestamp = targetDateTime.getTime()

    const isoValue = toIsoLocal(targetDateTime)
    return {
      id: commit.id,
      authorDate: isoValue,
      committerDate: isoValue,
    }
  })
}
