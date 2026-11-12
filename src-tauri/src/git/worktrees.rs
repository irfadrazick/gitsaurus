//! Linked worktrees: list, remove, prune (`git worktree`).
use std::path::{Path, PathBuf};

use super::error::{GitError, GitResult};
use super::runner::Git;
use super::types::Worktree;

/// Parses `git worktree list --porcelain -z`. Bare entries are skipped; the first
/// non-bare entry of a non-bare repository is the main worktree.
pub fn parse_worktrees(out: &str) -> Vec<Worktree> {
    let mut list = Vec::new();
    let mut first = true;
    for record in out
        .split("\0\0")
        .filter(|r| !r.trim_matches('\0').is_empty())
    {
        let (mut path, mut oid, mut branch) = (None, None, None);
        let (mut bare, mut prunable, mut locked) = (false, false, false);
        for field in record.split('\0') {
            let (key, value) = field.split_once(' ').unwrap_or((field, ""));
            match key {
                "worktree" => path = Some(value.to_string()),
                "HEAD" => oid = Some(value.to_string()).filter(|v| !v.chars().all(|c| c == '0')),
                "branch" => {
                    branch = Some(
                        value
                            .strip_prefix("refs/heads/")
                            .unwrap_or(value)
                            .to_string(),
                    )
                }
                "bare" => bare = true,
                "prunable" => prunable = true,
                "locked" => locked = true,
                _ => {}
            }
        }
        let is_main = std::mem::take(&mut first);
        let Some(path) = path else { continue };
        if bare {
            continue;
        }
        let name = Path::new(&path)
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.clone());
        let missing = prunable || !Path::new(&path).exists();
        list.push(Worktree {
            path,
            name,
            branch,
            oid,
            is_main,
            is_current: false,
            missing,
            locked,
        });
    }
    list
}

fn canon(p: &Path) -> PathBuf {
    std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf())
}

pub fn list_worktrees(git: &Git) -> GitResult<Vec<Worktree>> {
    let out = git.run_read(&["worktree", "list", "--porcelain", "-z"])?;
    let here = canon(git.repo());
    let mut list = parse_worktrees(&out);
    for w in &mut list {
        w.is_current = canon(Path::new(&w.path)) == here;
    }
    Ok(list)
}

fn refuse(message: &str) -> GitError {
    GitError::Failed {
        message: message.into(),
        stderr: String::new(),
    }
}

/// Removes the linked worktree at `path`. Refuses the main worktree and the one `git`
/// runs in (git would delete the folder out from under the open window).
/// `force` also discards its uncommitted and untracked changes.
pub fn remove_worktree(git: &Git, path: &str, force: bool) -> GitResult<()> {
    let target = canon(Path::new(path));
    let list = list_worktrees(git)?;
    let Some(w) = list.iter().find(|w| canon(Path::new(&w.path)) == target) else {
        return Err(refuse("Not a worktree of this repository"));
    };
    if w.is_main {
        return Err(refuse("The main worktree can't be removed"));
    }
    if w.is_current {
        return Err(refuse("Open another worktree to remove this one"));
    }
    let mut args = vec!["worktree", "remove"];
    if force {
        args.push("--force");
    }
    args.push(&w.path);
    git.run(&args).map(|_| ())
}

/// Forgets worktrees whose folders no longer exist.
pub fn prune_worktrees(git: &Git) -> GitResult<()> {
    git.run(&["worktree", "prune"]).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_branch_detached_prunable_locked_and_skips_bare() {
        let out = "worktree /r/main\0HEAD aaaa\0branch refs/heads/main\0\0\
                   worktree /r/wt1\0HEAD bbbb\0branch refs/heads/feat/x\0locked\0\0\
                   worktree /r/wt2\0HEAD cccc\0detached\0\0\
                   worktree /gone/wt3\0HEAD dddd\0branch refs/heads/old\0prunable gitdir file points to non-existent location\0\0";
        let w = parse_worktrees(out);
        assert_eq!(w.len(), 4);
        assert!(w[0].is_main && !w[1].is_main);
        assert_eq!(w[0].name, "main");
        assert_eq!(w[1].branch.as_deref(), Some("feat/x"));
        assert!(w[1].locked);
        assert_eq!(
            (w[2].branch.as_deref(), w[2].oid.as_deref()),
            (None, Some("cccc"))
        );
        assert!(w[3].missing);
    }

    #[test]
    fn a_bare_main_entry_is_skipped() {
        let out =
            "worktree /r/repo.git\0bare\0\0worktree /r/wt\0HEAD aaaa\0branch refs/heads/x\0\0";
        let w = parse_worktrees(out);
        assert_eq!(w.len(), 1);
        assert_eq!(w[0].name, "wt");
        assert!(!w[0].is_main);
    }

    #[test]
    fn an_unborn_head_has_no_oid() {
        let out = format!(
            "worktree /r/m\0HEAD {}\0branch refs/heads/main\0\0",
            "0".repeat(40)
        );
        assert_eq!(parse_worktrees(&out)[0].oid, None);
    }
}
