/** Where the app's own update stands. Only 'ready' shows outside Settings. */
export type UpdateStatus =
  | { kind: 'idle' }
  | { kind: 'checking' }
  | { kind: 'upToDate' }
  | { kind: 'downloading'; version: string }
  | { kind: 'ready'; version: string }
  | { kind: 'error'; message: string };

/** Plain words for the updater's errors; the common one means GitHub had no release for us, or we're offline. */
export function updateErrorMessage(e: unknown): string {
  const text = e instanceof Error ? e.message : String(e);
  if (/valid release JSON|error sending request|connect/i.test(text)) return "GitHub couldn't be reached, or has no release yet.";
  return text;
}

/** The line shown under "Check now" in Settings. */
export function updateStatusText(s: UpdateStatus, current: string): string {
  switch (s.kind) {
    case 'idle':
      return current ? `Version ${current}` : '';
    case 'checking':
      return 'Checking for updates…';
    case 'upToDate':
      return `You're up to date (${current}).`;
    case 'downloading':
      return `Downloading version ${s.version}…`;
    case 'ready':
      return `Version ${s.version} is ready. Restart Gitsaurus to use it.`;
    case 'error':
      return `Couldn't check for updates: ${s.message}`;
  }
}
