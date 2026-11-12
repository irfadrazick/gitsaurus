/** Shiki language ids we bundle. */
export type Lang =
  | 'hcl' | 'terraform' | 'python' | 'typescript' | 'javascript' | 'json' | 'yaml' | 'markdown' | 'go'
  | 'rust' | 'shellscript' | 'svelte' | 'css' | 'html' | 'toml' | 'sql' | 'dockerfile';

const EXT: Record<string, Lang> = {
  tf: 'terraform', tfvars: 'terraform', hcl: 'hcl', py: 'python',
  ts: 'typescript', tsx: 'typescript', mts: 'typescript', cts: 'typescript',
  js: 'javascript', jsx: 'javascript', mjs: 'javascript', cjs: 'javascript',
  json: 'json', yml: 'yaml', yaml: 'yaml', md: 'markdown', markdown: 'markdown',
  go: 'go', rs: 'rust', sh: 'shellscript', bash: 'shellscript', zsh: 'shellscript',
  svelte: 'svelte', css: 'css', html: 'html', htm: 'html', toml: 'toml', sql: 'sql',
};

/** Language for a path by extension or file name, or null when unknown. */
export function languageFor(path: string): Lang | null {
  const name = (path.split('/').pop() ?? '').toLowerCase();
  if (name === 'dockerfile' || name.startsWith('dockerfile.')) return 'dockerfile';
  const dot = name.lastIndexOf('.');
  return dot > 0 ? (EXT[name.slice(dot + 1)] ?? null) : null;
}

export function escapeHtml(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;').replace(/'/g, '&#39;');
}
