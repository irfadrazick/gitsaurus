import type { ChangedFile, TreeNode } from '../types';

function sortNodes(nodes: TreeNode[]) {
  nodes.sort((a, b) => Number(b.isDir) - Number(a.isDir) || a.name.localeCompare(b.name));
  for (const n of nodes) sortNodes(n.children);
}

/** Nested folder tree of changed files; folders first, then alphabetical. */
export function buildTree(files: ChangedFile[]): TreeNode[] {
  const root: TreeNode[] = [];
  for (const file of files) {
    const parts = file.path.split('/');
    let level = root;
    parts.forEach((name, i) => {
      const path = parts.slice(0, i + 1).join('/');
      const last = i === parts.length - 1;
      let node = level.find((n) => n.name === name && n.isDir === !last);
      if (!node) {
        node = { name, path, isDir: !last, children: [], ...(last ? { file } : {}) };
        level.push(node);
      }
      level = node.children;
    });
  }
  sortNodes(root);
  return root;
}
