use super::error::GitResult;
use super::refs::head;
use super::runner::Git;
use super::types::{FileStatus, Head, OpKind, Operation, RepoState};

pub fn repo_state(git: &Git) -> GitResult<RepoState> {
    let raw = git.run_read(&["status", "--porcelain=v2", "-z", "--untracked-files=all"])?;
    let head = head(git)?;
    let files = parse_porcelain_v2(raw.as_bytes());
    let operation = operation(git, &head);
    Ok(RepoState {
        head,
        operation,
        files,
    })
}

/// Parses `git status --porcelain=v2 -z` output.
pub fn parse_porcelain_v2(raw: &[u8]) -> Vec<FileStatus> {
    let mut out = Vec::new();
    let mut records = raw
        .split(|b| *b == 0)
        .filter(|r| !r.is_empty())
        .map(|r| String::from_utf8_lossy(r).into_owned());
    while let Some(rec) = records.next() {
        let (xy, path, orig_path, conflicted, untracked) = match rec.as_bytes()[0] {
            b'1' => {
                let mut p = rec.splitn(9, ' ');
                let xy = p.nth(1).unwrap_or("..");
                let Some(path) = p.nth(6) else { continue };
                (xy.to_string(), path.to_string(), None, false, false)
            }
            b'2' => {
                let mut p = rec.splitn(10, ' ');
                let xy = p.nth(1).unwrap_or("..");
                let Some(path) = p.nth(7) else { continue };
                (
                    xy.to_string(),
                    path.to_string(),
                    records.next(),
                    false,
                    false,
                )
            }
            b'u' => {
                let mut p = rec.splitn(11, ' ');
                let xy = p.nth(1).unwrap_or("UU");
                let Some(path) = p.nth(8) else { continue };
                (xy.to_string(), path.to_string(), None, true, false)
            }
            b'?' => match rec.get(2..) {
                Some(path) => (".?".to_string(), path.to_string(), None, false, true),
                None => continue,
            },
            _ => continue,
        };
        let mut chars = xy.chars();
        let index = chars.next().unwrap_or('.').to_string();
        let worktree = chars.next().unwrap_or('.').to_string();
        out.push(FileStatus {
            path,
            orig_path,
            index,
            worktree,
            conflicted,
            untracked,
        });
    }
    out
}

fn short(oid: &str) -> String {
    oid.chars().take(7).collect()
}

fn read_trimmed(path: &std::path::Path) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_string())
}

fn name_rev(git: &Git, oid: &str, extra: &[&str]) -> String {
    let mut args = vec!["name-rev", "--name-only", "--no-undefined"];
    args.extend_from_slice(extra);
    args.push(oid);
    match git.run_read(&args) {
        Ok(s) if !s.trim().is_empty() => s.trim().to_string(),
        _ => short(oid),
    }
}

pub fn operation(git: &Git, head: &Head) -> Option<Operation> {
    let ours_default = head
        .branch
        .clone()
        .or_else(|| head.oid.as_deref().map(short))
        .unwrap_or_else(|| "HEAD".to_string());

    for (dir, step_file, total_file) in [
        ("rebase-merge", "msgnum", "end"),
        ("rebase-apply", "next", "last"),
    ] {
        let Ok(d) = git.git_path(dir) else { continue };
        if !d.is_dir() {
            continue;
        }
        let num = |name: &str| read_trimmed(&d.join(name)).and_then(|s| s.parse::<u32>().ok());
        let ours_label = read_trimmed(&d.join("onto"))
            .map(|onto| name_rev(git, &onto, &["--refs=refs/heads/*"]))
            .unwrap_or(ours_default.clone());
        let theirs_label = read_trimmed(&d.join("head-name"))
            .map(|h| h.strip_prefix("refs/heads/").unwrap_or(&h).to_string())
            .unwrap_or_default();
        return Some(Operation {
            kind: OpKind::Rebase,
            step: num(step_file),
            total: num(total_file),
            ours_label,
            theirs_label,
        });
    }

    let head_file = |name: &str| {
        let p = git.git_path(name).ok()?;
        if p.is_file() { read_trimmed(&p) } else { None }
    };
    let simple = |kind, theirs_label| {
        Some(Operation {
            kind,
            step: None,
            total: None,
            ours_label: ours_default.clone(),
            theirs_label,
        })
    };
    if let Some(oid) = head_file("MERGE_HEAD") {
        return simple(OpKind::Merge, name_rev(git, &oid, &[]));
    }
    if let Some(oid) = head_file("CHERRY_PICK_HEAD") {
        return simple(OpKind::CherryPick, short(&oid));
    }
    if let Some(oid) = head_file("REVERT_HEAD") {
        return simple(OpKind::Revert, format!("revert {}", short(&oid)));
    }
    None
}
