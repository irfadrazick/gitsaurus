use super::error::GitResult;
use super::runner::Git;
use super::types::{Branch, Head, Refs, RemoteBranch, Stash, Tag};

/// Parses `%(upstream:track)` output into (ahead, behind, gone).
pub fn parse_track(s: &str) -> (u32, u32, bool) {
    let inner = s.trim().trim_start_matches('[').trim_end_matches(']');
    let (mut ahead, mut behind, mut gone) = (0, 0, false);
    for part in inner.split(',') {
        let part = part.trim();
        if part == "gone" {
            gone = true;
        } else if let Some(n) = part.strip_prefix("ahead ") {
            ahead = n.trim().parse().unwrap_or(0);
        } else if let Some(n) = part.strip_prefix("behind ") {
            behind = n.trim().parse().unwrap_or(0);
        }
    }
    (ahead, behind, gone)
}

pub fn head(git: &Git) -> GitResult<Head> {
    let branch = git.run_read_ok_codes(&["symbolic-ref", "-q", "--short", "HEAD"], &[0, 1])?;
    let oid = git.run_read_ok_codes(&["rev-parse", "-q", "--verify", "HEAD"], &[0, 1])?;
    let non_empty = |s: String| {
        let t = s.trim();
        (!t.is_empty()).then(|| t.to_string())
    };
    Ok(Head {
        branch: non_empty(branch),
        oid: non_empty(oid),
    })
}

/// Splits `origin/feature/x` into (`origin`, `feature/x`) using the longest matching remote name.
pub(crate) fn split_remote<'a>(remotes: &'a [String], full: &'a str) -> Option<(&'a str, &'a str)> {
    let r = remotes
        .iter()
        .filter(|r| {
            full.strip_prefix(r.as_str())
                .is_some_and(|s| s.starts_with('/'))
        })
        .max_by_key(|r| r.len())?;
    Some((r.as_str(), &full[r.len() + 1..]))
}

pub fn list_refs(git: &Git) -> GitResult<Refs> {
    let head = head(git)?;
    let remotes: Vec<String> = git
        .run_read(&["remote"])?
        .lines()
        .map(str::to_string)
        .filter(|l| !l.is_empty())
        .collect();

    let out = git.run_read(&[
        "for-each-ref",
        "--format=%(refname)%00%(objectname)%00%(*objectname)%00%(upstream:short)%00%(upstream:track)%00%(HEAD)",
        "refs/heads",
        "refs/remotes",
        "refs/tags",
    ])?;

    let mut local = Vec::new();
    let mut remote = Vec::new();
    let mut tags = Vec::new();
    for line in out.lines() {
        let f: Vec<&str> = line.split('\0').collect();
        if f.len() < 6 {
            continue;
        }
        let (refname, oid, peeled, upstream, track, is_head) = (f[0], f[1], f[2], f[3], f[4], f[5]);
        if let Some(name) = refname.strip_prefix("refs/heads/") {
            let (ahead, behind, gone) = parse_track(track);
            local.push(Branch {
                name: name.to_string(),
                full_ref: refname.to_string(),
                oid: oid.to_string(),
                upstream: (!upstream.is_empty()).then(|| upstream.to_string()),
                ahead,
                behind,
                gone,
                is_head: is_head == "*",
            });
        } else if let Some(rest) = refname.strip_prefix("refs/remotes/") {
            let Some((r, name)) = split_remote(&remotes, rest) else {
                continue;
            };
            if name == "HEAD" {
                continue;
            }
            remote.push(RemoteBranch {
                remote: r.to_string(),
                name: name.to_string(),
                full_name: rest.to_string(),
                oid: oid.to_string(),
            });
        } else if let Some(name) = refname.strip_prefix("refs/tags/") {
            let annotated = !peeled.is_empty();
            tags.push(Tag {
                name: name.to_string(),
                oid: if annotated { peeled } else { oid }.to_string(),
                annotated,
            });
        }
    }
    local.sort_by(|a, b| b.is_head.cmp(&a.is_head).then_with(|| a.name.cmp(&b.name)));
    remote.sort_by(|a, b| a.full_name.cmp(&b.full_name));
    tags.sort_by(|a, b| a.name.cmp(&b.name));

    let mut stashes = Vec::new();
    if head.oid.is_some() {
        let out = git.run_read(&["stash", "list", "--format=%gd%x1f%H%x1f%gs"])?;
        for line in out.lines() {
            let mut p = line.splitn(3, '\u{1f}');
            let (Some(gd), Some(oid), Some(msg)) = (p.next(), p.next(), p.next()) else {
                continue;
            };
            let index = gd
                .trim_start_matches("stash@{")
                .trim_end_matches('}')
                .parse()
                .unwrap_or(0);
            stashes.push(Stash {
                index,
                message: msg.to_string(),
                oid: oid.to_string(),
            });
        }
        stashes.sort_by_key(|s| s.index);
    }

    Ok(Refs {
        head,
        local,
        remote,
        remotes,
        tags,
        stashes,
    })
}
