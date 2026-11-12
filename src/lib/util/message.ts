/** Git trailers shown elsewhere (co-authors) or of no use when reading a message. */
const TRAILER = /^(co-authored-by|signed-off-by|reviewed-by|acked-by|tested-by|reported-by|helped-by|change-id):/i;

/** A line that must keep its own line: list items, indented or code-like text. */
const STRUCTURAL = /^(\s|[-*+•]\s|\d+[.)]\s|>|```|\|)/;

/**
 * A commit body for reading in a panel narrower than the 72 columns it was wrapped at:
 * hard-wrapped prose lines rejoin into paragraphs, structure (lists, indented code) stays,
 * and trailers such as Co-Authored-By are dropped.
 */
export function displayBody(body: string): string {
  const paragraphs = body
    .replace(/\r\n/g, '\n')
    .split(/\n\s*\n/)
    .map((p) => p.split('\n').filter((l) => !TRAILER.test(l.trim())))
    .filter((lines) => lines.some((l) => l.trim() !== ''));
  return paragraphs
    .map((lines) => {
      let out = '';
      for (const line of lines) {
        if (out === '') out = line.trimEnd();
        else if (STRUCTURAL.test(line)) out += '\n' + line.trimEnd();
        else {
          const prev = out.slice(out.lastIndexOf('\n') + 1);
          out += (STRUCTURAL.test(prev) || prev.endsWith(':') ? '\n' : ' ') + line.trim();
        }
      }
      return out;
    })
    .join('\n\n');
}
