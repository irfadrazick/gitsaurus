# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

Gitsaurus is a macOS git client: Tauri 2, a Rust backend that runs the `git` command line, and a Svelte 5 + TypeScript frontend. It is macOS-only for now (the runner uses `std::os::unix`, and cargo-deny only checks the macOS targets).

## Commands

```sh
pnpm install
pnpm tauri dev                  # run with hot reload (debug builds skip the single-instance lock)
pnpm tauri build --bundles app  # build src-tauri/target/release/bundle/macos/Gitsaurus.app

pnpm check                      # svelte-check (types)
pnpm test                       # vitest, all frontend tests
pnpm vitest run src/lib/util/graphGeometry.test.ts   # one file
pnpm vitest run -t "segmentPath"                     # by test name

cd src-tauri
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings   # CI treats warnings as errors
cargo test --locked                                  # everything
cargo test --test ops_branch                         # one integration test file (src-tauri/tests/)
cargo test --test graph some_test_name               # one test
GITSAURUS_UPDATE_FIXTURES=1 cargo test --test contract   # regenerate the frontend's repo_state fixture
GITSAURUS_PERF_REPO=/path/to/repo cargo test --release --test perf -- --ignored --nocapture
```

CI (`.github/workflows/ci.yml`) runs `pnpm check`, `pnpm test`, `pnpm build`, `cargo fmt --check`, clippy and `cargo test --locked`, plus `pnpm audit --audit-level moderate` and `cargo deny` (`src-tauri/deny.toml`). CodeQL runs in its own workflow.

## Architecture

**Everything is the git CLI.** `git::runner::Git` (`src-tauri/src/git/runner.rs`) spawns `git` with the PATH and SSH_AUTH_SOCK of the user's login shell (`git/env.rs`, resolved once on a background thread at startup). Network commands run one at a time per repository, can be cancelled (`cancel_network`), and are stopped only when they produce no output for `NETWORK_STALL_LIMIT`. Errors cross IPC as `GitError`, serialised as `{ kind, message, stderr? }` with kinds `notARepo`, `gitMissing`, `pushRejected`, `failed` and `cancelled`.

**IPC layer.** `src-tauri/src/commands.rs` is the whole Tauri surface; every command takes `repo` first. Read-only work goes through `read()` (`spawn_blocking`), and mutating work through `write()`, which also holds the per-repo lock in `AppState`. Adding a command means: a function in `commands.rs`, an entry in `generate_handler!` in `lib.rs`, a typed wrapper in `src/lib/api.ts`, and matching types in `src/lib/types.ts`. Rust types serialise as camelCase.

**Commit graph.** `load_graph` reads the log, lays out every row in Rust (`git/graph.rs`, `RowLayout`: lane, colour, and the lines into, out of and through the node) and caches the result in `AppState.graphs`. The frontend then fetches rows in pages of `PAGE_SIZE` (200) with `graph_page`, and virtualises the list (`GraphView.svelte`). `src/lib/util/graphGeometry.ts` turns a layout into SVG paths. `ROW_H` there must match the `.row` height in `GraphRow.svelte`.

**Live updates.** `watcher.rs` watches each open repo's worktree and git dir and emits `repo-changed` with kinds `worktree`/`git`. `App.svelte` refreshes the selected repo (`repo.refresh(kinds)`) or updates the change indicator of another open repo. Window focus triggers the same checks to catch missed events.

**Frontend state** lives in `src/lib/state/`, as Svelte 5 rune classes in `*.svelte.ts`:
- `repo.svelte.ts` is the selected repository: refs, working state, graph pages, selection, and what the centre panel shows.
- `session.svelte.ts` and `settings.svelte.ts` persist to `session.json` and `settings.json` through `persist.ts` (plugin-store; a no-op outside Tauri, so tests don't need it).
- Mutating actions go through `runOp(label, fn)` in `ops.ts`. It tracks busy state, shows the success or error toast and refreshes afterwards; an `OpOutcome` of `"conflicts"` hands over to the conflict banner instead.
- Context-menu actions map (menu kind, item id) to an action in `util/actionTable.ts` and run in `state/actions.ts`.
- Keyboard shortcuts are defined in `util/shortcuts.ts`. Toolbar actions reach the toolbar through the small event bus in `state/commands.ts`.

**Where tests go.** Components have no tests. Logic that needs testing lives as pure functions in `src/lib/util/` next to its `*.test.ts`. Rust integration tests in `src-tauri/tests/` run real git against hermetic temporary repositories (`tests/common::TestRepo` ignores the developer's global git config). `tests/contract.rs` writes `src/lib/util/__fixtures__/repo_state.json` from a real `repo_state`, so the frontend tests catch backend shape changes; it fails until the fixture is regenerated.

**Styling.** Use the tokens in `src/app.css` (colours, the `--fs-*` type scale, `--hover`/`--active`/`--sel` states, near-sharp `--radius*`) rather than literal values. Diff styles are shared in `src/lib/components/diff/diff.css`.

## Repository rules

- `main` is protected: every change goes through a pull request with passing CI and CodeQL checks. Commit messages use conventional prefixes (`feat:`, `fix:`, `style:`, `docs:`, `chore:`, `ci:`).
- GitHub Actions are pinned to full commit SHAs with a version comment; keep that when adding or updating one.
- Releases: `pnpm set-version X.Y.Z` on a branch, merge it, then tag `vX.Y.Z` on main; the tag runs `release.yml`, which builds a draft release. `v*` tags can't be moved or deleted, so a failed release is followed by the next patch version. See "Releasing" in README.md.
- `docs/superpowers/`, `.superpowers/` and `.impeccable/` are local working files and are gitignored; don't commit them.
