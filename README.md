# Gitsaurus

[![CI](https://github.com/irfadrazick/gitsaurus/actions/workflows/ci.yml/badge.svg)](https://github.com/irfadrazick/gitsaurus/actions/workflows/ci.yml)
[![CodeQL](https://github.com/irfadrazick/gitsaurus/actions/workflows/codeql.yml/badge.svg)](https://github.com/irfadrazick/gitsaurus/actions/workflows/codeql.yml)
[![OpenSSF Scorecard](https://api.scorecard.dev/projects/github.com/irfadrazick/gitsaurus/badge)](https://scorecard.dev/viewer/?uri=github.com/irfadrazick/gitsaurus)

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
- Updates itself: new versions download in the background and take effect when you click Restart (it never restarts on its own)
- Dark theme, no telemetry, no accounts. Network use is limited to your git remotes, optional avatar lookups and a check for new versions on GitHub every few hours (both can be turned off in Settings, Cmd+,)

## Install

1. Download the latest `.dmg` from [Releases](https://github.com/irfadrazick/gitsaurus/releases). It runs on Apple Silicon and Intel Macs with macOS 13 or later.
2. Open it and drag Gitsaurus into Applications.
3. The app is not notarised by Apple yet, so the first launch is blocked. Click Done, then open System Settings > Privacy & Security, scroll to "Gitsaurus was blocked" and click Open Anyway. After that it opens normally.

   Or, from a terminal: `xattr -dr com.apple.quarantine /Applications/Gitsaurus.app`

Git must be installed (`xcode-select --install` provides it).

After that, Gitsaurus keeps itself up to date: when a new version has been downloaded, a "Restart to update" button appears at the bottom of the Repositories column.

## Verify a download

Each release lists SHA-256 checksums in `SHA256SUMS.txt`, and every file carries a build provenance attestation signed by GitHub Actions. To confirm a download was built by this repository's Release workflow from the tagged source:

```sh
gh attestation verify Gitsaurus_0.2.0_universal.dmg --repo irfadrazick/gitsaurus
```

## Security

Every pull request and push to `main` runs these checks:

- **CI**: type checks, tests, `clippy` and `rustfmt`, `pnpm audit`, and [`cargo deny`](src-tauri/deny.toml), which checks RustSec advisories, licenses and crate sources.
- **CodeQL**: scans the TypeScript/Svelte, Rust and workflow code. Findings go to the repository's Security tab.

[OpenSSF Scorecard](https://scorecard.dev/viewer/?uri=github.com/irfadrazick/gitsaurus) rates the project's security practices each week. Dependabot keeps npm, Cargo and GitHub Actions dependencies current, and every action is pinned to a commit SHA.

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

The tag starts the Release workflow, which builds a universal `.dmg` and `.zip` and attaches them to a draft release, along with the signed update and `latest.json` that installed copies check. Check the draft on GitHub, edit the notes and publish it. Installed copies see the update once it is published.

Updates are signed with a key kept in the `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` repository secrets; the matching public key is in `src-tauri/tauri.conf.json`. Keep a backup of the key: without it, installed copies can't be updated.

## License

[MIT](LICENSE)
