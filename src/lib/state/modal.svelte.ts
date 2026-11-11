export interface ConfirmOpts { title: string; body: string; danger?: boolean; confirmLabel: string }
export interface PromptOpts {
  title: string; label: string; initial?: string;
  /** Resolves to an error message, or null when valid. */
  validate?: (v: string) => Promise<string | null>;
}
export interface ChooseOpts { title: string; options: { value: string; label: string }[] | string[] }

export type ModalRequest =
  | ({ type: 'confirm'; resolve: (v: boolean) => void } & ConfirmOpts)
  | ({ type: 'prompt'; resolve: (v: string | null) => void } & PromptOpts)
  | ({ type: 'choose'; resolve: (v: string | null) => void } & ChooseOpts);

/** Currently shown dialog; Modal.svelte renders it. Requests queue behind each other. */
export const modalState = $state<{ current: ModalRequest | null }>({ current: null });
const queue: ModalRequest[] = [];

function show(req: ModalRequest) {
  if (modalState.current) queue.push(req);
  else modalState.current = req;
}

/** Called by Modal.svelte once the current dialog has resolved. */
export function closeModal() {
  modalState.current = queue.shift() ?? null;
}

export const confirm = (o: ConfirmOpts) =>
  new Promise<boolean>((resolve) => show({ type: 'confirm', resolve, ...o }));
export const prompt = (o: PromptOpts) =>
  new Promise<string | null>((resolve) => show({ type: 'prompt', resolve, ...o }));
export const choose = (o: ChooseOpts) =>
  new Promise<string | null>((resolve) => show({ type: 'choose', resolve, ...o }));
