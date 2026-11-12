/**
 * Why `name` is not a valid branch or tag name, following `git check-ref-format`, or null
 * when it looks valid. Git stays the authority; this only makes the message specific.
 */
export function explainRefName(name: string): string | null {
  if (name === '') return 'Enter a name.';
  if (/\s/.test(name)) return "Names can't contain spaces.";
  if (name.startsWith('-')) return 'Names can\'t start with "-".';
  if (name === '@') return '"@" on its own isn\'t allowed.';
  if (name.includes('..')) return 'Names can\'t contain "..".';
  if (name.includes('@{')) return 'Names can\'t contain "@{".';
  // eslint-disable-next-line no-control-regex
  if (/[~^:?*[\\\x00-\x1f\x7f]/.test(name)) return "Names can't contain ~ ^ : ? * [ or \\.";
  if (name.endsWith('.') || name.endsWith('/') || name.endsWith('.lock')) return 'Names can\'t end with ".", "/" or ".lock".';
  if (name.split('/').some((part) => part === '' || part.startsWith('.')))
    return 'Each part between slashes must be non-empty and not start with ".".';
  return null;
}
