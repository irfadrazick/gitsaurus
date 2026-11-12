import { createHash } from 'node:crypto';
import { expect, it } from 'vitest';
import { avatarsAllowed, avatarUrl, initials } from './avatar';

it('github noreply with id', async () => {
  expect(await avatarUrl('123+bob@users.noreply.github.com')).toBe('https://avatars.githubusercontent.com/u/123?s=64&v=4');
});
it('github noreply without id', async () => {
  expect(await avatarUrl('bob@users.noreply.github.com')).toBe('https://github.com/bob.png?size=64');
});
it('gravatar sha256 of trimmed lowercase email', async () => {
  const h = createHash('sha256').update('a@x.com').digest('hex');
  expect(await avatarUrl('A@X.com')).toBe(`https://gravatar.com/avatar/${h}?s=64&d=404`);
});
it('initials', () => {
  expect(initials('Irfad Razick')).toBe('IR');
  expect(initials('bob')).toBe('B');
});

it('allows avatar lookups only once settings have loaded and avatars are on', () => {
  // Before load the default (on) must not leak a request when the saved setting is off.
  expect(avatarsAllowed({ loaded: false, avatarsEnabled: true })).toBe(false);
  expect(avatarsAllowed({ loaded: true, avatarsEnabled: false })).toBe(false);
  expect(avatarsAllowed({ loaded: true, avatarsEnabled: true })).toBe(true);
});
