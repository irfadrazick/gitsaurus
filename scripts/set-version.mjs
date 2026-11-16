#!/usr/bin/env node
// Sets the app version everywhere it is declared: package.json, tauri.conf.json, Cargo.toml and Cargo.lock.
// Usage: pnpm set-version 0.2.0
import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const version = process.argv[2]?.replace(/^v/, '');
if (!version || !/^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/.test(version)) {
  console.error('Usage: pnpm set-version <major.minor.patch>, e.g. pnpm set-version 0.2.0');
  process.exit(1);
}

const root = fileURLToPath(new URL('..', import.meta.url));
const edits = [
  ['package.json', /^(  "version": ")[^"]+(")/m],
  ['src-tauri/tauri.conf.json', /^(  "version": ")[^"]+(")/m],
  ['src-tauri/Cargo.toml', /^(\[package\][^[]*?\nversion = ")[^"]+(")/m],
  ['src-tauri/Cargo.lock', /^(name = "gitsaurus"\nversion = ")[^"]+(")/m],
];

for (const [file, pattern] of edits) {
  const path = root + file;
  const text = readFileSync(path, 'utf8');
  if (!pattern.test(text)) {
    console.error(`Could not find the version in ${file}`);
    process.exit(1);
  }
  writeFileSync(path, text.replace(pattern, `$1${version}$2`));
}

console.log(`Version set to ${version}. Next, merge it through a PR (main is protected), then tag the merge:
  git switch -c release/v${version}
  git commit -am "chore: release v${version}"
  git push -u origin HEAD && gh pr create --fill
  gh pr merge --merge --delete-branch   # once CI passes
  git switch main && git pull --ff-only
  git tag v${version} && git push origin v${version}`);
