# Gitsaurus

A minimal, fast git client for macOS. It shows your history as a lane graph and keeps everyday work (staging, committing, branching, pulling, pushing) one click away. It shells out to the `git` CLI, so it behaves exactly like git on your machine and uses your existing credentials, config and hooks.

## Features

- Several repositories open at once, each with a live dirty indicator
- Commit graph that stays smooth on large histories (tens of thousands of commits), with search
- Branches, remotes, tags and stashes in a side column, each with a context menu
- Worktrees listed beside them: open one as its own repo, show it in Finder, or remove it
- Working changes: stage, unstage, discard a file or all unstaged changes at once, commit (with amend)
- Unified and split diffs with syntax highlighting
- Merge, rebase, cherry-pick, revert and reset, with a conflict resolver
- Toolbar: Pull (fetch, fast-forward, fast-forward only, rebase), Push, Branch, Stash, Pop, Search
- Force push only ever uses `--force-with-lease`, and only after you confirm
- Background fetch on an interval you choose. Fetch and Pull both drop branches deleted on the remote, and a remote branch can be deleted from its menu
- Dark theme, no telemetry, no accounts. Network use is limited to your git remotes and optional avatar lookups (can be turned off in Settings, Cmd+,)

## Install

1. Download the latest `.dmg` from [Releases](https://github.com/irfadrazick/gitsaurus/releases). It runs on Apple Silicon and Intel Macs with macOS 13 or later.
2. Open it and drag Gitsaurus into Applications.
3. The app is not notarised by Apple yet, so the first launch is blocked. Click Done, then open System Settings > Privacy & Security, scroll to "Gitsaurus was blocked" and click Open Anyway. After that it opens normally.

   Or, from a terminal: `xattr -dr com.apple.quarantine /Applications/Gitsaurus.app`

Git must be installed (`xcode-select --install` provides it).

## Build from source

Requires macOS, Node.js, [pnpm](https://pnpm.io), Rust and the Xcode command line tools.

```sh
pnpm install && pnpm tauri build
```

The app is produced at `src-tauri/target/release/bundle/macos/Gitsaurus.app`, with a `.dmg` beside it in `bundle/dmg/`.

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

## Release

```sh
pnpm set-version 0.2.0
git commit -am "chore: release v0.2.0"
git tag v0.2.0
git push origin main v0.2.0
```

The tag starts the Release workflow, which builds a universal `.dmg` and `.zip` and attaches them to a draft release. Check the draft on GitHub, edit the notes and publish it.

## License

[MIT](LICENSE)
