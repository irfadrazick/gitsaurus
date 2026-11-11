use super::branch::safe_target;
use crate::git::refs::head;
use crate::git::status::{operation, repo_state};
use crate::git::{Git, GitError, GitResult, OpKind, OpOutcome, Side};

fn failed(message: impl Into<String>) -> GitError {
    GitError::Failed {
        message: message.into(),
        stderr: String::new(),
    }
}

/// Maps a command result to an outcome: a failure that left an operation in
/// progress with conflicted files is `Conflicts`; any other failure propagates.
pub(crate) fn conflict_outcome(git: &Git, res: GitResult<String>) -> GitResult<OpOutcome> {
    match res {
        Ok(_) => Ok(OpOutcome::Done),
        Err(e) => {
            let in_conflict = head(git).ok().is_some_and(|h| {
                operation(git, &h).is_some()
                    && repo_state(git).is_ok_and(|s| s.files.iter().any(|f| f.conflicted))
            });
            if in_conflict {
                Ok(OpOutcome::Conflicts)
            } else {
                Err(e)
            }
        }
    }
}

pub fn merge(git: &Git, target: &str) -> GitResult<OpOutcome> {
    safe_target(target)?;
    let res = git.run(&["merge", "--no-edit", "--end-of-options", target]);
    conflict_outcome(git, res)
}

pub fn rebase(git: &Git, onto: &str) -> GitResult<OpOutcome> {
    safe_target(onto)?;
    let res = git.run(&["rebase", "--end-of-options", onto]);
    conflict_outcome(git, res)
}

fn is_merge_commit(git: &Git, oid: &str) -> GitResult<bool> {
    let out = git.run_read(&["rev-list", "--parents", "-n", "1", "--end-of-options", oid])?;
    Ok(out.split_whitespace().count() > 2)
}

pub fn cherry_pick(git: &Git, oid: &str) -> GitResult<OpOutcome> {
    safe_target(oid)?;
    let mut args = vec!["cherry-pick"];
    if is_merge_commit(git, oid)? {
        args.extend(["-m", "1"]);
    }
    args.extend(["--end-of-options", oid]);
    let res = git.run(&args);
    conflict_outcome(git, res)
}

pub fn revert(git: &Git, oid: &str) -> GitResult<OpOutcome> {
    safe_target(oid)?;
    let mut args = vec!["revert", "--no-edit"];
    if is_merge_commit(git, oid)? {
        args.extend(["-m", "1"]);
    }
    args.extend(["--end-of-options", oid]);
    let res = git.run(&args);
    conflict_outcome(git, res)
}

fn current_kind(git: &Git) -> GitResult<OpKind> {
    let h = head(git)?;
    operation(git, &h)
        .map(|o| o.kind)
        .ok_or_else(|| failed("No operation in progress"))
}

fn subcommand(kind: OpKind) -> &'static str {
    match kind {
        OpKind::Merge => "merge",
        OpKind::Rebase => "rebase",
        OpKind::CherryPick => "cherry-pick",
        OpKind::Revert => "revert",
    }
}

pub fn continue_op(git: &Git) -> GitResult<OpOutcome> {
    let sub = subcommand(current_kind(git)?);
    // GIT_EDITOR=true accepts the prepared message without opening an editor.
    let res = git.run_env(&[sub, "--continue"], &[("GIT_EDITOR", "true")]);
    conflict_outcome(git, res)
}

pub fn skip_op(git: &Git) -> GitResult<OpOutcome> {
    if current_kind(git)? != OpKind::Rebase {
        return Err(failed("Skip is only available during a rebase"));
    }
    let res = git.run_env(&["rebase", "--skip"], &[("GIT_EDITOR", "true")]);
    conflict_outcome(git, res)
}

pub fn abort_op(git: &Git) -> GitResult<()> {
    let sub = subcommand(current_kind(git)?);
    git.run(&[sub, "--abort"]).map(|_| ())
}

pub fn resolve_conflict(git: &Git, path: &str, side: Side) -> GitResult<()> {
    let (flag, stage) = match side {
        Side::Ours => ("--ours", "2"),
        Side::Theirs => ("--theirs", "3"),
    };
    // Only touch files that are actually unmerged; checkout would otherwise
    // silently discard working-tree edits on a normal tracked file.
    let raw = git.run_read(&["ls-files", "-u", "-z", "--", path])?;
    let stages: Vec<&str> = raw
        .split('\0')
        .filter_map(|e| e.split_once('\t'))
        .filter_map(|(meta, _)| meta.split_whitespace().nth(2))
        .collect();
    if stages.is_empty() {
        return Err(failed(format!("Not in conflict: {path}")));
    }
    if stages.contains(&stage) {
        git.run(&["checkout", flag, "--", path])?;
        git.run(&["add", "--", path])?;
    } else {
        // That side deleted the file, so resolving to it means removing it.
        git.run(&["rm", "-q", "--", path])?;
    }
    Ok(())
}

pub fn mark_resolved(git: &Git, path: &str) -> GitResult<()> {
    git.run(&["add", "--", path]).map(|_| ())
}
