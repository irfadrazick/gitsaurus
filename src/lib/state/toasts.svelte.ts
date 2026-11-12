export interface ToastAction { label: string; run: () => unknown }
export interface Toast { id: number; kind: 'ok' | 'info' | 'error'; title: string; detail?: string; action?: ToastAction }

class Toasts {
  items = $state<Toast[]>([]);
  #next = 1;

  push(t: Omit<Toast, 'id'>) {
    const id = this.#next++;
    this.items.push({ ...t, id });
    // A toast with a button stays long enough to read and decide.
    setTimeout(() => this.dismiss(id), t.action ? 20000 : t.kind === 'error' ? 8000 : 3500);
    return id;
  }

  dismiss(id: number) {
    const i = this.items.findIndex((t) => t.id === id);
    if (i >= 0) this.items.splice(i, 1);
  }
}

export const toasts = new Toasts();
