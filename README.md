# Gitsaurus

A minimal, fast git client for macOS. It shows your history as a lane graph and keeps everyday work (staging, committing, branching, pulling, pushing) one click away. It shells out to the `git` CLI, so it behaves exactly like git on your machine and uses your existing credentials, config and hooks.

## Features

- Several repositories open at once, each with a live dirty indicator
- Commit graph that stays smooth on large histories (tens of thousands of commits), with search
- Branches, remotes, tags and stashes in a side column, each with a context menu
- Worktrees listed beside them: open one as its own repo, show it in Finder, or remove it
- Working changes: stage, unstage, discard, commit (with amend)
- Unified and split diffs with syntax highlighting
- Merge, rebase, cherry-pick, revert and reset, with a conflict resolver
- Toolbar: Pull (fetch, fast-forward, fast-forward only, rebase), Push, Branch, Stash, Pop, Search
- Force push only ever uses `--force-with-lease`, and only after you confirm
- Background fetch on an interval you choose. Fetch and Pull both drop branches deleted on the remote, and a remote branch can be deleted from its menu
- Dark theme, no telemetry, no accounts. Network use is limited to your git remotes and optional avatar lookups (can be turned off in Settings, Cmd+,)

## Build

Requires macOS, Node.js, [pnpm](https://pnpm.io), Rust and the Xcode command line tools.

```sh
pnpm install && pnpm tauri build
```

The app is produced at `src-tauri/target/release/bundle/macos/Gitsaurus.app`.

## Install on another Mac

Copy `Gitsaurus.app` to the other Mac (for example into `/Applications`). The build is not notarised, so the first time right-click the app and choose Open, then confirm. After that it opens normally. Git must be installed on that Mac.

## Develop

```sh
pnpm install
pnpm tauri dev
```

## Test

```sh
pnpm test
cd src-tauri && cargo test
```

A graph timing test against a large repository is opt-in:

```sh
cd src-tauri
GITSAURUS_PERF_REPO=/path/to/big/repo cargo test --release --test perf -- --ignored --nocapture
```
