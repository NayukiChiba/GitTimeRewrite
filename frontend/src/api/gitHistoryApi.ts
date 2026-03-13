import { invoke } from '@tauri-apps/api/core'
import type { GitCommit, RewriteRequest, RewriteResult } from '@/types/gitHistory'

export async function getGitHistory(repoPath: string): Promise<GitCommit[]> {
  return invoke<GitCommit[]>('get_git_history', { repoPath })
}

export async function rewriteGitHistory(request: RewriteRequest): Promise<RewriteResult> {
  return invoke<RewriteResult>('rewrite_git_history', { request })
}

export async function setGitOrigin(repoPath: string, originUrl: string): Promise<string> {
  return invoke<string>('set_git_origin', { repoPath, originUrl })
}

export async function forcePushOrigin(repoPath: string): Promise<string> {
  return invoke<string>('force_push_origin', { repoPath })
}
