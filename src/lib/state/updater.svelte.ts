import { getVersion } from '@tauri-apps/api/app';
import { check } from '@tauri-apps/plugin-updater';
import { restartApp } from '../api';
import { updateErrorMessage, type UpdateStatus } from '../util/updateStatus';
import { busy } from './ops';

const FIRST_CHECK_MS = 10_000;
const CHECK_EVERY_MS = 6 * 60 * 60 * 1000;

/**
 * Checks GitHub for a newer published release, downloads and installs it in the background,
 * then waits: the new version runs after the user clicks Restart (or next time they open the app).
 * Never restarts on its own.
 */
class Updater {
  status = $state<UpdateStatus>({ kind: 'idle' });
  /** The running version, once known. */
  current = $state('');
  #running = false;

  async init() {
    try {
      this.current = await getVersion();
    } catch {
      /* not running under Tauri */
    }
  }

  /** Background checks: one shortly after launch, then every few hours. Returns a stop function. */
  schedule(): () => void {
    const first = setTimeout(() => this.check(false), FIRST_CHECK_MS);
    const every = setInterval(() => this.check(false), CHECK_EVERY_MS);
    return () => {
      clearTimeout(first);
      clearInterval(every);
    };
  }

  /** `manual` (Settings > Check now) reports errors; background checks fail silently. */
  async check(manual: boolean) {
    if (this.#running || this.status.kind === 'ready') return;
    if (import.meta.env.DEV) {
      if (manual) this.status = { kind: 'error', message: 'updates are off in development builds' };
      return;
    }
    this.#running = true;
    if (manual) this.status = { kind: 'checking' };
    try {
      const update = await check();
      if (!update) {
        this.status = { kind: 'upToDate' };
        return;
      }
      this.status = { kind: 'downloading', version: update.version };
      await update.downloadAndInstall();
      this.status = { kind: 'ready', version: update.version };
    } catch (e) {
      console.warn('Update check failed', e);
      this.status = manual ? { kind: 'error', message: updateErrorMessage(e) } : { kind: 'idle' };
    } finally {
      this.#running = false;
    }
  }

  /** False while a git operation is running, so a restart can't cut one short. */
  get canRestart() {
    return this.status.kind === 'ready' && busy.size === 0;
  }

  async restart() {
    if (this.canRestart) await restartApp();
  }
}

export const updater = new Updater();
