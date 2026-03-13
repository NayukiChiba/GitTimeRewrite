import { invoke } from '@tauri-apps/api/core'
import type { GitCommit, RewriteRequest, RewriteResult } from '@/types/gitHistory'

export async function getGitHistory(repoPath: string): Promise<GitCommit[]> {
  return invoke<GitCommit[]>('get_git_history', { repoPath })
}

export async function rewriteGitHistory(request: RewriteRequest): Promise<RewriteResult> {
  return invoke<RewriteResult>('rewrite_git_history', { request })
}
