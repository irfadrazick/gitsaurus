import { createPersister } from './persist';

export type DiffMode = 'unified' | 'split';
export type FileListMode = 'path' | 'tree';
export interface Widths { repos: number; refs: number; details: number }

interface SessionData {
  openRepos: string[];
  selected: string | null;
  widths: Widths;
  reposCollapsed: boolean;
  diffMode: DiffMode;
  fileListMode: FileListMode;
}

const persister = createPersister<SessionData>('session.json');

class Session implements SessionData {
  openRepos = $state<string[]>([]);
  selected = $state<string | null>(null);
  widths = $state<Widths>({ repos: 200, refs: 220, details: 360 });
  reposCollapsed = $state(false);
  diffMode = $state<DiffMode>('unified');
  fileListMode = $state<FileListMode>('path');
  loaded = $state(false);

  snapshot(): SessionData {
    return {
      openRepos: [...this.openRepos],
      selected: this.selected,
      widths: { ...this.widths },
      reposCollapsed: this.reposCollapsed,
      diffMode: this.diffMode,
      fileListMode: this.fileListMode,
    };
  }

  /** Load persisted state, then start saving every change (debounced 300 ms). Call once at startup. */
  async init() {
    const d = await persister.read();
    if (d.openRepos) this.openRepos = d.openRepos;
    if (d.selected !== undefined) this.selected = d.selected;
    if (d.widths) this.widths = { ...this.widths, ...d.widths };
    if (d.reposCollapsed !== undefined) this.reposCollapsed = d.reposCollapsed;
    if (d.diffMode) this.diffMode = d.diffMode;
    if (d.fileListMode) this.fileListMode = d.fileListMode;
    if (this.selected && !this.openRepos.includes(this.selected)) this.selected = this.openRepos[0] ?? null;
    this.loaded = true;
    $effect.root(() => {
      $effect(() => {
        persister.write(this.snapshot());
      });
    });
  }

  /** Add (if new) and select a repo. */
  addRepo(path: string) {
    if (!this.openRepos.includes(path)) this.openRepos.push(path);
    this.selected = path;
  }

  closeRepo(path: string) {
    const i = this.openRepos.indexOf(path);
    if (i < 0) return;
    this.openRepos.splice(i, 1);
    if (this.selected === path) this.selected = this.openRepos[Math.min(i, this.openRepos.length - 1)] ?? null;
  }
}

export const session = new Session();
