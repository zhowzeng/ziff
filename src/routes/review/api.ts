import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type { Branch, DiffHunk, DiffSpec, FetchResult, FileContent, Repo, TreeNode } from './types';

export function listRepos() {
  return invoke<Repo[]>('list_repos');
}

export function addRepo(path: string) {
  return invoke<Repo>('add_repo', { path });
}

/** Native folder picker. Resolves to null when the reviewer cancels. */
export async function pickRepoFolder() {
  const picked = await open({ directory: true, multiple: false, title: 'Add repo' });
  return typeof picked === 'string' ? picked : null;
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
