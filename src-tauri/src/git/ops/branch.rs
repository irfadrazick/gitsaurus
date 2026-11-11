use crate::git::refs::split_remote;
use crate::git::{Git, GitError, GitResult, RefKind, ResetMode};

fn failed(message: impl Into<String>) -> GitError {
    GitError::Failed {
        message: message.into(),
        stderr: String::new(),
    }
}

/// Rejects anything git could read as an option.
fn safe_target(s: &str) -> GitResult<()> {
    if s.is_empty() || s.starts_with('-') {
        return Err(failed("Invalid ref"));
    }
    Ok(())
}

pub fn validate_ref_name(git: &Git, name: &str) -> GitResult<()> {
    let bad = || failed(format!("Invalid branch name: {name}"));
    if name.is_empty() || name.starts_with('-') {
        return Err(bad());
    }
    git.run_read(&["check-ref-format", "--branch", name])
        .map(|_| ())
        .map_err(|_| bad())
}

pub fn validate_tag_name(git: &Git, name: &str) -> GitResult<()> {
    let bad = || failed(format!("Invalid tag name: {name}"));
    if name.is_empty() || name.starts_with('-') {
        return Err(bad());
    }
    git.run_read(&["check-ref-format", &format!("refs/tags/{name}")])
        .map(|_| ())
        .map_err(|_| bad())
}

fn local_exists(git: &Git, name: &str) -> bool {
    git.run_read(&[
        "show-ref",
        "--verify",
        "--quiet",
        &format!("refs/heads/{name}"),
    ])
    .is_ok()
}

pub fn checkout(git: &Git, target: &str, kind: RefKind) -> GitResult<()> {
    safe_target(target)?;
    match kind {
        RefKind::Local => git
            .run(&["checkout", "--end-of-options", target])
            .map(|_| ()),
        RefKind::Tag => git
            .run(&["checkout", "--detach", "--end-of-options", target])
            .map(|_| ()),
        RefKind::Remote => {
            let remotes: Vec<String> = git
                .run_read(&["remote"])?
                .lines()
                .map(str::to_string)
                .filter(|l| !l.is_empty())
                .collect();
            let (_, name) = split_remote(&remotes, target).ok_or_else(|| failed("Invalid ref"))?;
            if local_exists(git, name) {
                git.run(&["checkout", "--end-of-options", name]).map(|_| ())
            } else {
                git.run(&[
                    "checkout",
                    "-b",
                    name,
                    "--track",
                    "--end-of-options",
                    target,
                ])
                .map(|_| ())
            }
        }
    }
}

pub fn checkout_commit(git: &Git, oid: &str) -> GitResult<()> {
    safe_target(oid)?;
    git.run(&["checkout", "--detach", "--end-of-options", oid])
        .map(|_| ())
}

pub fn create_branch(git: &Git, name: &str, at: &str, checkout: bool) -> GitResult<()> {
    validate_ref_name(git, name)?;
    safe_target(at)?;
    if checkout {
        git.run(&["checkout", "-b", name, "--end-of-options", at])
    } else {
        git.run(&["branch", "--end-of-options", name, at])
    }
    .map(|_| ())
}

pub fn rename_branch(git: &Git, old: &str, new: &str) -> GitResult<()> {
    safe_target(old)?;
    validate_ref_name(git, new)?;
    git.run(&["branch", "-m", "--end-of-options", old, new])
        .map(|_| ())
}

pub fn delete_branch(git: &Git, name: &str, force: bool, remote: Option<&str>) -> GitResult<()> {
    safe_target(name)?;
    if let Some(r) = remote {
        safe_target(r)?;
    }
    git.run(&[
        "branch",
        if force { "-D" } else { "-d" },
        "--end-of-options",
        name,
    ])?;
    if let Some(r) = remote {
        git.run(&["push", r, "--delete", "--end-of-options", name])?;
    }
    Ok(())
}

pub fn create_tag(git: &Git, name: &str, at: &str, message: Option<&str>) -> GitResult<()> {
    validate_tag_name(git, name)?;
    safe_target(at)?;
    match message.map(str::trim).filter(|m| !m.is_empty()) {
        Some(m) => git.run_stdin(&["tag", "-a", "-F", "-", "--end-of-options", name, at], m),
        None => git.run(&["tag", "--end-of-options", name, at]),
    }
    .map(|_| ())
}

pub fn delete_tag(git: &Git, name: &str, remote: Option<&str>) -> GitResult<()> {
    validate_tag_name(git, name)?;
    if let Some(r) = remote {
        safe_target(r)?;
    }
    git.run(&["tag", "-d", "--end-of-options", name])?;
    if let Some(r) = remote {
        git.run(&[
            "push",
            r,
            "--delete",
            "--end-of-options",
            &format!("refs/tags/{name}"),
        ])?;
    }
    Ok(())
}

pub fn reset(git: &Git, oid: &str, mode: ResetMode) -> GitResult<()> {
    safe_target(oid)?;
    let flag = match mode {
        ResetMode::Soft => "--soft",
        ResetMode::Mixed => "--mixed",
        ResetMode::Hard => "--hard",
    };
    git.run(&["reset", flag, "--end-of-options", oid])
        .map(|_| ())
}

pub fn stash_push(git: &Git, message: Option<&str>) -> GitResult<()> {
    let mut args = vec!["stash", "push", "--include-untracked"];
    if let Some(m) = message.map(str::trim).filter(|m| !m.is_empty()) {
        args.extend(["-m", m]);
    }
    // With GIT_LITERAL_PATHSPECS=1, `stash -u` stashes untracked files but fails to remove them
    // from the worktree (git 2.49), so literal mode is switched off; no pathspecs are passed.
    git.run_env(&args, &[("GIT_LITERAL_PATHSPECS", "0")])
        .map(|_| ())
}

fn stash_op(git: &Git, op: &str, index: u32) -> GitResult<()> {
    git.run(&["stash", op, &format!("stash@{{{index}}}")])
        .map(|_| ())
}

pub fn stash_apply(git: &Git, index: u32) -> GitResult<()> {
    stash_op(git, "apply", index)
}

pub fn stash_pop(git: &Git, index: u32) -> GitResult<()> {
    stash_op(git, "pop", index)
}

pub fn stash_drop(git: &Git, index: u32) -> GitResult<()> {
    stash_op(git, "drop", index)
}
