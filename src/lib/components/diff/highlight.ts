import type { HighlighterCore } from 'shiki/core';
import { escapeHtml, languageFor, type Lang } from '../../util/lang';

// Only these grammars and one theme are bundled (dynamic imports keep each in its own small chunk).
const LOADERS: Record<Lang, () => Promise<{ default: unknown }>> = {
  hcl: () => import('shiki/langs/hcl.mjs'),
  terraform: () => import('shiki/langs/terraform.mjs'),
  python: () => import('shiki/langs/python.mjs'),
  typescript: () => import('shiki/langs/typescript.mjs'),
  javascript: () => import('shiki/langs/javascript.mjs'),
  json: () => import('shiki/langs/json.mjs'),
  yaml: () => import('shiki/langs/yaml.mjs'),
  markdown: () => import('shiki/langs/markdown.mjs'),
  go: () => import('shiki/langs/go.mjs'),
  rust: () => import('shiki/langs/rust.mjs'),
  shellscript: () => import('shiki/langs/shellscript.mjs'),
  svelte: () => import('shiki/langs/svelte.mjs'),
  css: () => import('shiki/langs/css.mjs'),
  html: () => import('shiki/langs/html.mjs'),
  toml: () => import('shiki/langs/toml.mjs'),
  sql: () => import('shiki/langs/sql.mjs'),
  dockerfile: () => import('shiki/langs/dockerfile.mjs'),
};

const THEME = 'github-dark';

let highlighter: Promise<HighlighterCore> | null = null;
const loaded = new Set<Lang>();

function getHighlighter(): Promise<HighlighterCore> {
  highlighter ??= (async () => {
    const [{ createHighlighterCore }, { createJavaScriptRegexEngine }, theme] = await Promise.all([
      import('shiki/core'),
      import('shiki/engine/javascript'),
      import('shiki/themes/github-dark.mjs'),
    ]);
    return createHighlighterCore({ themes: [theme.default], langs: [], engine: createJavaScriptRegexEngine() });
  })();
  highlighter.catch(() => (highlighter = null));
  return highlighter;
}

const plain = (lines: string[]) => lines.map(escapeHtml);

/**
 * Per-line HTML for `lines` (one side of a file, highlighted as one joined block so multi-line tokens
 * colour correctly). Unknown file types, or any highlighter failure, fall back to escaped plain text.
 */
export async function highlightLines(lines: string[], path: string): Promise<string[]> {
  const lang = languageFor(path);
  if (!lang || lines.length === 0) return plain(lines);
  try {
    const h = await getHighlighter();
    if (!loaded.has(lang)) {
      await h.loadLanguage((await LOADERS[lang]()).default as never);
      loaded.add(lang);
    }
    const { tokens } = h.codeToTokens(lines.join('\n'), { lang, theme: THEME });
    if (tokens.length !== lines.length) return plain(lines);
    return tokens.map((line) =>
      line
        .map((t) => (t.color ? `<span style="color:${t.color}">${escapeHtml(t.content)}</span>` : escapeHtml(t.content)))
        .join(''),
    );
  } catch {
    return plain(lines);
  }
}
