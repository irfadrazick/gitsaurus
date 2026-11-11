use super::error::{GitError, GitResult};
use super::runner::Git;
use super::types::{Change, ChangedFile, CommitDetails, Person};

pub fn commit_details(git: &Git, oid: &str) -> GitResult<CommitDetails> {
    let meta = git.run_read(&[
        "show",
        "-s",
        "--format=%H%x1f%P%x1f%an%x1f%ae%x1f%at%x1f%cn%x1f%ce%x1f%ct%x1f%s%x1f%b",
        oid,
    ])?;
    let f: Vec<&str> = meta.splitn(10, '\x1f').collect();
    if f.len() < 10 {
        return Err(GitError::Failed {
            message: format!("unexpected output for commit {oid}"),
            stderr: String::new(),
        });
    }
    let parents: Vec<String> = f[1].split_whitespace().map(str::to_string).collect();
    let person = |n: &str, e: &str, t: &str| Person {
        name: n.to_string(),
        email: e.to_string(),
        time: t.parse().unwrap_or(0),
    };
    let body = f[9].trim_end().to_string();
    let full = f[0].to_string();

    let mut args = vec![
        "diff-tree",
        "-r",
        "-M",
        "--no-commit-id",
        "--name-status",
        "-z",
    ];
    match parents.first() {
        Some(p) => args.push(p),
        None => args.push("--root"),
    }
    args.push(&full);
    let raw = git.run_read(&args)?;

    Ok(CommitDetails {
        oid: full.clone(),
        parents: parents.clone(),
        author: person(f[2], f[3], f[4]),
        committer: person(f[5], f[6], f[7]),
        summary: f[8].to_string(),
        co_authors: parse_co_authors(&body),
        body,
        files: parse_name_status(&raw),
    })
}

fn parse_co_authors(body: &str) -> Vec<Person> {
    const PREFIX: &str = "co-authored-by:";
    body.lines()
        .filter_map(|line| {
            let line = line.trim_end();
            if line.len() < PREFIX.len() || !line.is_char_boundary(PREFIX.len()) {
                return None;
            }
            if !line[..PREFIX.len()].eq_ignore_ascii_case(PREFIX) {
                return None;
            }
            let rest = line[PREFIX.len()..].trim().strip_suffix('>')?;
            let lt = rest.rfind('<')?;
            let email = &rest[lt + 1..];
            let name = rest[..lt].trim();
            if name.is_empty() || email.is_empty() || email.contains('>') {
                return None;
            }
            Some(Person {
                name: name.to_string(),
                email: email.to_string(),
                time: 0,
            })
        })
        .collect()
}

fn parse_name_status(raw: &str) -> Vec<ChangedFile> {
    let mut out = Vec::new();
    let mut it = raw.split('\0').filter(|s| !s.is_empty());
    while let Some(status) = it.next() {
        let change = match status.as_bytes()[0] {
            b'A' => Change::Added,
            b'M' => Change::Modified,
            b'D' => Change::Deleted,
            b'R' => Change::Renamed,
            b'C' => Change::Copied,
            b'T' => Change::TypeChange,
            _ => {
                it.next();
                continue;
            }
        };
        let (orig_path, path) = if matches!(change, Change::Renamed | Change::Copied) {
            let (Some(o), Some(p)) = (it.next(), it.next()) else {
                break;
            };
            (Some(o.to_string()), p.to_string())
        } else {
            let Some(p) = it.next() else { break };
            (None, p.to_string())
        };
        out.push(ChangedFile {
            path,
            orig_path,
            change,
        });
    }
    out
}
