import { load } from '@tauri-apps/plugin-store';

/** Debounced JSON persistence over plugin-store. Silently no-ops outside Tauri (vitest, plain browser). */
export function createPersister<T extends object>(file: string, delay = 300) {
  let store: Awaited<ReturnType<typeof load>> | null = null;
  let timer: ReturnType<typeof setTimeout> | undefined;
  const open = async () => (store ??= await load(file, { autoSave: false, defaults: {} }));

  return {
    async read(): Promise<Partial<T>> {
      try {
        const s = await open();
        const out: Record<string, unknown> = {};
        for (const [k, v] of await s.entries()) out[k] = v;
        return out as Partial<T>;
      } catch {
        return {};
      }
    },
    write(data: T) {
      clearTimeout(timer);
      timer = setTimeout(async () => {
        try {
          const s = await open();
          for (const [k, v] of Object.entries(data)) await s.set(k, v);
          await s.save();
        } catch {
          /* not running under Tauri */
        }
      }, delay);
    },
  };
}
