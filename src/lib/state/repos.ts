import { open } from '@tauri-apps/plugin-dialog';
import * as api from '../api';
import { session } from './session.svelte';
import { toasts } from './toasts.svelte';

/** Pick a folder, validate it as a repository, then add and select it. */
export async function openRepoDialog() {
  const picked = await open({ directory: true, multiple: false });
  if (typeof picked !== 'string') return;
  try {
    const info = await api.openRepo(picked);
    session.addRepo(info.path);
  } catch (e) {
    const err = e as { message?: string };
    toasts.push({ kind: 'error', title: 'Not a git repository', detail: err?.message ?? String(e) });
  }
}
