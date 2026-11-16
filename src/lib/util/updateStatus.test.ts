import { describe, expect, it } from 'vitest';
import { updateErrorMessage, updateStatusText } from './updateStatus';

describe('updateErrorMessage', () => {
  it('turns fetch failures into plain words and keeps other errors', () => {
    const plain = "GitHub couldn't be reached, or has no release yet.";
    expect(updateErrorMessage('Could not fetch a valid release JSON from the remote')).toBe(plain);
    expect(updateErrorMessage(new Error('error sending request for url'))).toBe(plain);
    expect(updateErrorMessage('signature verification failed')).toBe('signature verification failed');
  });
});

describe('updateStatusText', () => {
  it('describes each state', () => {
    expect(updateStatusText({ kind: 'idle' }, '0.1.0')).toBe('Version 0.1.0');
    expect(updateStatusText({ kind: 'idle' }, '')).toBe('');
    expect(updateStatusText({ kind: 'checking' }, '0.1.0')).toBe('Checking for updates…');
    expect(updateStatusText({ kind: 'upToDate' }, '0.1.0')).toBe("You're up to date (0.1.0).");
    expect(updateStatusText({ kind: 'downloading', version: '0.2.0' }, '0.1.0')).toBe('Downloading version 0.2.0…');
    expect(updateStatusText({ kind: 'ready', version: '0.2.0' }, '0.1.0')).toBe(
      'Version 0.2.0 is ready. Restart Gitsaurus to use it.',
    );
    expect(updateStatusText({ kind: 'error', message: 'offline' }, '0.1.0')).toBe("Couldn't check for updates: offline");
  });
});
