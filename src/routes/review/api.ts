import { invoke } from '@tauri-apps/api/core';
import type { Branch, DiffHunk, DiffSpec, FetchResult, FileContent, Repo, TreeNode } from './types';

export function listRepos() {
  return invoke<Repo[]>('list_repos');
}

export function listBranches(repoId: string) {
  return invoke<Branch[]>('list_branches', { repoId });
}

export function getFileTree(spec: DiffSpec) {
  return invoke<TreeNode[]>('get_file_tree', { spec });
}

export function getFileDiff(spec: DiffSpec, path: string) {
  return invoke<DiffHunk[]>('get_file_diff', { spec, path });
}

export function getFileContent(repoId: string, path: string) {
  return invoke<FileContent>('get_file_content', { repoId, path });
}

export function fetchRemote(repoId: string) {
  return invoke<FetchResult>('fetch_remote', { repoId });
}
