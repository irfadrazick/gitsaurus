use crate::git::{Git, GitError, GitResult, head, repo_state};

fn unborn(git: &Git) -> GitResult<bool> {
    Ok(head(git)?.oid.is_none())
}

fn run_with_paths(git: &Git, base: &[&str], paths: &[String]) -> GitResult<()> {
    let mut args: Vec<&str> = base.to_vec();
    args.push("--");
    args.extend(paths.iter().map(String::as_str));
    git.run(&args).map(|_| ())
}

pub fn stage(git: &Git, paths: &[String]) -> GitResult<()> {
    if paths.is_empty() {
        return Ok(());
    }
    run_with_paths(git, &["add", "-A"], paths)
}

pub fn stage_all(git: &Git) -> GitResult<()> {
    git.run(&["add", "-A"]).map(|_| ())
}

pub fn unstage(git: &Git, paths: &[String]) -> GitResult<()> {
    if paths.is_empty() {
        return Ok(());
    }
    if unborn(git)? {
        run_with_paths(git, &["rm", "--cached", "-r", "-q"], paths)
    } else {
        run_with_paths(git, &["restore", "--staged"], paths)
    }
}

pub fn unstage_all(git: &Git) -> GitResult<()> {
    if unborn(git)? {
        git.run(&["rm", "--cached", "-r", "-q", "."]).map(|_| ())
    } else {
        git.run(&["reset", "-q"]).map(|_| ())
    }
}

pub fn discard(git: &Git, paths: &[String]) -> GitResult<()> {
    if paths.is_empty() {
        return Ok(());
    }
    let state = repo_state(git)?;
    let (untracked, tracked): (Vec<String>, Vec<String>) = paths
        .iter()
        .cloned()
        .partition(|p| state.files.iter().any(|f| f.untracked && f.path == *p));
    if !tracked.is_empty() {
        run_with_paths(git, &["restore", "--worktree"], &tracked)?;
    }
    if !untracked.is_empty() {
        run_with_paths(git, &["clean", "-f"], &untracked)?;
    }
    Ok(())
}

pub fn commit(git: &Git, summary: &str, description: &str, amend: bool) -> GitResult<()> {
    let summary = summary.trim();
    if summary.is_empty() {
        return Err(GitError::Failed {
            message: "Commit summary is empty".into(),
            stderr: String::new(),
        });
    }
    let description = description.trim_end();
    let mut msg = summary.to_string();
    if !description.is_empty() {
        msg.push_str("\n\n");
        msg.push_str(description);
    }
    let mut args = vec!["commit", "-F", "-"];
    if amend {
        args.push("--amend");
    }
    git.run_stdin(&args, &msg).map(|_| ())
}

/// (summary, body) of HEAD, used to pre-fill the amend box.
pub fn last_commit_message(git: &Git) -> GitResult<(String, String)> {
    if unborn(git)? {
        return Err(GitError::Failed {
            message: "No commits yet".into(),
            stderr: String::new(),
        });
    }
    let out = git.run_read(&["log", "-1", "--format=%s%x00%b"])?;
    let (summary, body) = out.split_once('\0').unwrap_or((out.as_str(), ""));
    Ok((summary.trim().to_string(), body.trim_end().to_string()))
}
