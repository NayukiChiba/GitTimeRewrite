export interface GitCommit {
  id: string
  parentIds: string[]
  message: string
  authorName: string
  authorEmail: string
  authorDate: string
  committerName: string
  committerEmail: string
  committerDate: string
}

export interface CommitEdit {
  id: string
  message?: string
  authorName?: string
  authorEmail?: string
  authorDate?: string
  committerName?: string
  committerEmail?: string
  committerDate?: string
}

export interface RewriteRequest {
  repoPath: string
  edits: CommitEdit[]
}

export interface RewriteResult {
  rewrittenCount: number
  output: string
}

export interface TimelineBatchInput {
  startDate: string
  endDate: string
  startTime: string
  endTime: string
}
