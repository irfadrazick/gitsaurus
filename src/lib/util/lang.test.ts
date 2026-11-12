import { describe, expect, it } from 'vitest';
import { escapeHtml, languageFor } from './lang';

describe('languageFor', () => {
  it.each([
    ['infra/main.tf', 'terraform'],
    ['vars.tfvars', 'terraform'],
    ['a/b.hcl', 'hcl'],
    ['x.py', 'python'],
    ['x.ts', 'typescript'],
    ['x.tsx', 'typescript'],
    ['x.mjs', 'javascript'],
    ['x.json', 'json'],
    ['ci.yml', 'yaml'],
    ['README.md', 'markdown'],
    ['main.go', 'go'],
    ['lib.rs', 'rust'],
    ['run.sh', 'shellscript'],
    ['App.svelte', 'svelte'],
    ['a.css', 'css'],
    ['a.html', 'html'],
    ['Cargo.toml', 'toml'],
    ['q.sql', 'sql'],
    ['Dockerfile', 'dockerfile'],
    ['docker/Dockerfile.prod', 'dockerfile'],
    ['X.PY', 'python'],
  ])('%s -> %s', (path, lang) => expect(languageFor(path)).toBe(lang));

  it('returns null for unknown types', () => {
    expect(languageFor('image.png')).toBeNull();
    expect(languageFor('LICENSE')).toBeNull();
  });
});

describe('escapeHtml', () => {
  it('escapes & < > and quotes', () => {
    expect(escapeHtml('<a href="x">&\'</a>')).toBe('&lt;a href=&quot;x&quot;&gt;&amp;&#39;&lt;/a&gt;');
  });
});
