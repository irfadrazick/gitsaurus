import { expect, it } from 'vitest';
import type { ChangedFile } from '../types';
import { buildTree } from './tree';

const f = (path: string): ChangedFile => ({ path, origPath: null, change: 'modified' });

it('nests, folders first, alphabetical', () => {
  const t = buildTree([f('z.txt'), f('src/b.ts'), f('a.txt'), f('src/lib/c.ts')]);
  expect(t.map((n) => n.name)).toEqual(['src', 'a.txt', 'z.txt']);
  const src = t[0];
  expect(src.isDir).toBe(true);
  expect(src.path).toBe('src');
  expect(src.children.map((n) => n.name)).toEqual(['lib', 'b.ts']);
  expect(src.children[0].children[0].file?.path).toBe('src/lib/c.ts');
});
