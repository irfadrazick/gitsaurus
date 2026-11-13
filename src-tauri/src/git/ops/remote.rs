use super::branch::safe_target;
use super::integrate::conflict_outcome;
use crate::git::refs::head;
use crate::git::{Git, GitError, GitResult, OpOutcome, PullMode};

fn failed(message: impl Into<String>) -> GitError {
    GitError::Failed {
        message: message.into(),
        stderr: String::new(),
    }
}

fn remotes(git: &Git) -> GitResult<Vec<String>> {
    Ok(git
        .run_read(&["remote"])?
        .lines()
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect())
}

/// Upstream of a local branch as (remote, full remote ref, e.g. `refs/heads/x`).
fn upstream_of(git: &Git, branch: &str) -> GitResult<Option<(String, String)>> {
    let refname = format!("refs/heads/{branch}");
    let out = git.run_read(&[
        "for-each-ref",
        "--format=%(upstream:remotename)%00%(upstream:remoteref)",
        &refname,
    ])?;
    let line = out.trim_end_matches('\n');
    let mut parts = line.split('\0');
    match (parts.next(), parts.next()) {
        (Some(r), Some(rr)) if !r.is_empty() && !rr.is_empty() => {
            Ok(Some((r.to_string(), rr.to_string())))
        }
        _ => Ok(None),
    }
}

const FETCH_ALL: &[&str] = &["fetch", "--all", "--prune", "--tags"];

pub fn fetch_all(git: &Git) -> GitResult<()> {
    git.run_network(FETCH_ALL).map(|_| ())
}

/// The periodic fetch: skipped when another network command is running on the repo.
pub fn background_fetch(git: &Git) -> GitResult<()> {
    git.run_network_if_idle(FETCH_ALL).map(|_| ())
}

fn config_is_set(git: &Git, key: &str) -> GitResult<bool> {
    let out = git.run_read_ok_codes(&["config", "--get", key], &[0, 1])?;
    Ok(!out.trim().is_empty())
}

pub fn pull(git: &Git, mode: PullMode) -> GitResult<OpOutcome> {
    let args: Vec<&str> = match mode {
        PullMode::FfOnly => vec!["pull", "--prune", "--ff-only"],
        PullMode::Rebase => vec!["pull", "--prune", "--rebase"],
        PullMode::Default => {
            if config_is_set(git, "pull.rebase")? || config_is_set(git, "pull.ff")? {
                vec!["pull", "--prune"]
            } else {
                vec!["pull", "--prune", "--no-rebase"]
            }
        }
    };
    // --prune: like Fetch, drop remote-tracking branches deleted on the remote.
    let res = git.run_network(&args);
    conflict_outcome(git, res)
}

pub fn pull_branch(git: &Git, local: &str) -> GitResult<()> {
    safe_target(local)?;
    // git refuses to fetch into the checked-out branch, so fast-forward it via pull.
    if head(git)?.branch.as_deref() == Some(local) {
        if upstream_of(git, local)?.is_none() {
            return Err(failed("No upstream"));
        }
        return pull(git, PullMode::FfOnly).map(|_| ());
    }
    let Some((remote, remote_ref)) = upstream_of(git, local)? else {
        return Err(failed("No upstream"));
    };
    // No leading `+`: a non-fast-forward update is rejected, never forced.
    let spec = format!("{remote_ref}:refs/heads/{local}");
    git.run_network(&["fetch", &remote, &spec]).map(|_| ())
}

fn map_push_error(e: GitError) -> GitError {
    match e {
        GitError::Failed { message, stderr }
            if ["[rejected]", "non-fast-forward", "fetch first"]
                .iter()
                .any(|m| stderr.contains(m)) =>
        {
            GitError::PushRejected { message, stderr }
        }
        other => other,
    }
}

pub fn push(git: &Git, branch: &str, force: bool) -> GitResult<()> {
    safe_target(branch)?;
    // --force-if-includes: the lease alone is satisfied by any fetch (including the
    // background one) that moved the remote-tracking ref; this also requires the
    // remote tip to be in the local branch's reflog, i.e. actually seen locally.
    let lease: &[&str] = if force {
        &["--force-with-lease", "--force-if-includes"]
    } else {
        &[]
    };
    let mut args: Vec<String> = vec!["push".into()];
    if let Some((remote, remote_ref)) = upstream_of(git, branch)? {
        args.extend(lease.iter().map(|s| s.to_string()));
        let short = remote_ref
            .strip_prefix("refs/heads/")
            .unwrap_or(&remote_ref);
        args.push(remote.clone());
        args.push(format!("{branch}:{short}"));
    } else {
        let all = remotes(git)?;
        let remote = if all.is_empty() {
            return Err(failed("No remote configured"));
        } else if all.iter().any(|r| r == "origin") {
            "origin".to_string()
        } else if all.len() == 1 {
            all[0].clone()
        } else {
            return Err(failed("Choose a remote: no upstream and no origin"));
        };
        args.push("-u".into());
        args.extend(lease.iter().map(|s| s.to_string()));
        args.push(remote);
        args.push(branch.to_string());
    }
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    git.run_network(&refs).map(|_| ()).map_err(map_push_error)
}

pub fn set_upstream(git: &Git, local: &str, remote_branch: &str) -> GitResult<()> {
    safe_target(local)?;
    safe_target(remote_branch)?;
    let flag = format!("--set-upstream-to={remote_branch}");
    git.run(&["branch", &flag, "--", local]).map(|_| ())
}

pub fn push_tag(git: &Git, tag: &str, remote: Option<&str>) -> GitResult<()> {
    safe_target(tag)?;
    let remote = remote.unwrap_or("origin");
    safe_target(remote)?;
    let spec = format!("refs/tags/{tag}");
    git.run_network_bounded(&["push", remote, &spec])
        .map(|_| ())
}
