import { SvelteMap, SvelteSet } from 'svelte/reactivity';
import { avatarsAllowed, avatarUrl } from '../util/avatar';
import { settings } from './settings.svelte';

// Module-level avatar cache. URLs are resolved once per email (avatarUrl hashes with SHA-256, async);
// failed loads are remembered so an email never retries. The browser HTTP cache holds the images.

const resolved = new SvelteMap<string, string>();
const failed = new SvelteSet<string>();
const pending = new Set<string>();

const key = (email: string) => email.trim().toLowerCase();

/**
 * Avatar image URL for `email`, or null (settings not loaded yet, avatars off, unresolved yet, or failed before).
 * Reactive: callers re-render once the URL resolves or a failure is recorded.
 */
export function avatarFor(email: string): string | null {
  if (!avatarsAllowed(settings)) return null;
  const k = key(email);
  if (!k || failed.has(k)) return null;
  const url = resolved.get(k);
  if (url) return url;
  if (!pending.has(k)) {
    pending.add(k);
    avatarUrl(k).then(
      (u) => resolved.set(k, u),
      () => failed.add(k),
    );
  }
  return null;
}

/** Record that the image for `email` failed to load (e.g. Gravatar 404); initials are shown from then on. */
export function avatarFailed(email: string) {
  failed.add(key(email));
}
