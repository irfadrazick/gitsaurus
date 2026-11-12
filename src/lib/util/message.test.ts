import { expect, it } from 'vitest';
import { displayBody } from './message';

it('rejoins hard-wrapped prose into paragraphs', () => {
  expect(displayBody('The stash list shifts when stash\nentries are added, so address them\nby oid.\n\nSecond paragraph\nwraps too.')).toBe(
    'The stash list shifts when stash entries are added, so address them by oid.\n\nSecond paragraph wraps too.',
  );
});

it('keeps lists, indented and code-like lines on their own lines', () => {
  const body = 'Changes:\n- one\n- two\n* three\n1. four\n    indented code\n  also code';
  expect(displayBody(body)).toBe(body);
});

it('drops trailers such as Co-Authored-By and Signed-off-by', () => {
  expect(displayBody('Fix it.\n\nCo-Authored-By: Claude <noreply@anthropic.com>\nSigned-off-by: A <a@b.c>')).toBe('Fix it.');
  expect(displayBody('Co-authored-by: X <x@y.z>')).toBe('');
});

it('keeps a "Key: value" line that is part of prose', () => {
  expect(displayBody('Note: this matters\nfor everyone.')).toBe('Note: this matters for everyone.');
});
