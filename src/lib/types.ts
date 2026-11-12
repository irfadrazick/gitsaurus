// Mirrors src-tauri/src/git/{types,error}.rs (serde camelCase; tagged enums use `kind`).

export interface RepoInfo { path: string; name: string }
export interface Head { branch: string | null; oid: string | null }
export interface Branch {
  name: string; fullRef: string; oid: string; upstream: string | null;
  ahead: number; behind: number; gone: boolean; isHead: boolean;
}
export interface RemoteBranch { remote: string; name: string; fullName: string; oid: string }
export interface Tag { name: string; oid: string; annotated: boolean }
export interface Stash { index: number; message: string; oid: string }
export interface Refs {
  head: Head; local: Branch[]; remote: RemoteBranch[]; remotes: string[]; tags: Tag[]; stashes: Stash[];
}

export type RefKind = 'local' | 'remote' | 'tag';
export interface RefLabel { kind: RefKind; name: string; isHead: boolean }
export interface Line { lane: number; color: number }
export interface GraphCommit {
  index: number; oid: string; shortOid: string; parents: string[];
  authorName: string; authorEmail: string; authorTime: number;
  summary: string; bodyPreview: string;
  lane: number; color: number; into: Line[]; out: Line[]; pass: Line[]; width: number;
  refs: RefLabel[];
}
export interface GraphSummary { total: number; maxLanes: number; headIndex: number | null }

export interface FileStatus {
  path: string; origPath: string | null; index: string; worktree: string;
  conflicted: boolean; untracked: boolean;
}
export type OpKind = 'merge' | 'rebase' | 'cherryPick' | 'revert';
export interface Operation {
  kind: OpKind; step: number | null; total: number | null; oursLabel: string; theirsLabel: string;
}
export interface RepoState { head: Head; operation: Operation | null; files: FileStatus[] }

export interface Person { name: string; email: string; time: number }
export type Change = 'added' | 'modified' | 'deleted' | 'renamed' | 'copied' | 'typeChange';
export interface ChangedFile { path: string; origPath: string | null; change: Change }
export interface CommitDetails {
  oid: string; parents: string[]; author: Person; committer: Person;
  summary: string; body: string; coAuthors: Person[]; files: ChangedFile[];
}

export type DiffSource =
  | { kind: 'commit'; oid: string }
  | { kind: 'unstaged' }
  | { kind: 'staged' }
  | { kind: 'untracked' }
  | { kind: 'workingFile' };
export type LineKind = 'context' | 'add' | 'del' | 'noNewline';
export interface DiffLine { kind: LineKind; oldNo: number | null; newNo: number | null; text: string }
export interface Hunk {
  header: string; oldStart: number; oldLines: number; newStart: number; newLines: number; lines: DiffLine[];
}
export interface FileDiff {
  path: string; origPath: string | null; binary: boolean; tooLarge: boolean; hunks: Hunk[];
}

export type OpOutcome = 'done' | 'conflicts';
export type PullMode = 'default' | 'ffOnly' | 'rebase';
export type ResetMode = 'soft' | 'mixed' | 'hard';
export type Side = 'ours' | 'theirs';

export type GitError =
  | { kind: 'notARepo'; message: string }
  | { kind: 'gitMissing'; message: string }
  | { kind: 'pushRejected'; message: string; stderr: string }
  | { kind: 'failed'; message: string; stderr: string }
  | { kind: 'cancelled'; message: string };

/** Event payload of "repo-changed". Kinds are lowercase on the wire. */
export interface RepoChanged { repo: string; kinds: ('git' | 'worktree')[] }
/** Frontend-side kind used by repo.refresh(). */
export type ChangeKind = 'Git' | 'Worktree';

export interface Pill { name: string; local: boolean; remote: boolean; tag: boolean; isHead: boolean }
export interface TreeNode {
  name: string; path: string; isDir: boolean; file?: ChangedFile; children: TreeNode[];
}
export interface SplitRow { left?: DiffLine; right?: DiffLine }
