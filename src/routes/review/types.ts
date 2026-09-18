// Mirrors the camelCase JSON shape serde produces for src-tauri/src/types.rs.

export type DiffMode = 'unstaged' | 'staged' | 'branch';

export interface Repo {
  id: string;
  name: string;
  path: string;
}

export interface Branch {
  name: string;
  isCurrent: boolean;
  ahead: number;
  behind: number;
}

export interface DiffSpec {
  repoId: string;
  branch: string;
  diffMode: DiffMode;
  baseBranch?: string;
}

export interface Changes {
  add: number;
  del: number;
}

export type TreeNode =
  | { type: 'dir'; name: string; path: string; children: TreeNode[] }
  | { type: 'file'; name: string; path: string; changes?: Changes };

export type LineKind = 'context' | 'add' | 'del';

export interface DiffLine {
  kind: LineKind;
  oldNo: number | null;
  newNo: number | null;
  content: string;
}

export interface DiffHunk {
  header: string;
  lines: DiffLine[];
}

export interface FetchResult {
  success: boolean;
  message: string;
}
