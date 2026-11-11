use super::error::GitResult;
use super::runner::Git;
use super::types::{DiffLine, DiffSource, FileDiff, Hunk, LineKind};

const EMPTY_TREE: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";
const MAX_BYTES: usize = 1_000_000;
const MAX_LINES: usize = 20_000;

fn empty(path: &str, orig_path: Option<&str>) -> FileDiff {
    FileDiff {
        path: path.to_string(),
        orig_path: orig_path.map(str::to_string),
        binary: false,
        too_large: false,
        hunks: vec![],
    }
}

fn parse_range(s: &str) -> Option<(u32, u32)> {
    match s.split_once(',') {
        Some((a, b)) => Some((a.parse().ok()?, b.parse().ok()?)),
        None => Some((s.parse().ok()?, 1)),
    }
}

/// Parses `@@ -a[,b] +c[,d] @@ ...` into (old_start, old_lines, new_start, new_lines).
fn parse_hunk_header(line: &str) -> Option<(u32, u32, u32, u32)> {
    let rest = line.strip_prefix("@@ -")?;
    let (old, rest) = rest.split_once(" +")?;
    let (new, _) = rest.split_once(" @@")?;
    let (os, ol) = parse_range(old)?;
    let (ns, nl) = parse_range(new)?;
    Some((os, ol, ns, nl))
}

pub fn parse_unified(path: &str, orig_path: Option<&str>, text: &str) -> FileDiff {
    let mut d = empty(path, orig_path);
    let mut cur: Option<(Hunk, u32, u32)> = None;
    let mut lines: Vec<&str> = text.split('\n').collect();
    if lines.last() == Some(&"") {
        lines.pop();
    }
    for line in lines {
        if let Some((os, ol, ns, nl)) = parse_hunk_header(line) {
            if let Some((h, _, _)) = cur.take() {
                d.hunks.push(h);
            }
            cur = Some((
                Hunk {
                    header: line.to_string(),
                    old_start: os,
                    old_lines: ol,
                    new_start: ns,
                    new_lines: nl,
                    lines: vec![],
                },
                os,
                ns,
            ));
            continue;
        }
        let Some((h, old_no, new_no)) = cur.as_mut() else {
            if line.starts_with("Binary files ") || line.starts_with("GIT binary patch") {
                d.binary = true;
            }
            continue;
        };
        let (kind, text) = match line.as_bytes().first() {
            Some(b'+') => (LineKind::Add, &line[1..]),
            Some(b'-') => (LineKind::Del, &line[1..]),
            Some(b'\\') => (LineKind::NoNewline, line),
            Some(b' ') => (LineKind::Context, &line[1..]),
            _ => (LineKind::Context, line),
        };
        let (o, n) = match kind {
            LineKind::Context => {
                let r = (Some(*old_no), Some(*new_no));
                *old_no += 1;
                *new_no += 1;
                r
            }
            LineKind::Add => {
                let r = (None, Some(*new_no));
                *new_no += 1;
                r
            }
            LineKind::Del => {
                let r = (Some(*old_no), None);
                *old_no += 1;
                r
            }
            LineKind::NoNewline => (None, None),
        };
        h.lines.push(DiffLine {
            kind,
            old_no: o,
            new_no: n,
            text: text.to_string(),
        });
    }
    if let Some((h, _, _)) = cur {
        d.hunks.push(h);
    }
    d
}

fn too_large(text: &str) -> bool {
    text.len() > MAX_BYTES || text.lines().count() > MAX_LINES
}

pub fn file_diff(
    git: &Git,
    path: &str,
    orig_path: Option<&str>,
    source: &DiffSource,
    context: u32,
) -> GitResult<FileDiff> {
    let u = format!("-U{context}");
    let mut args: Vec<&str> = vec!["diff", "--no-color", "--no-ext-diff", "-M", &u];
    let mut ok = vec![0];
    let base;
    match source {
        DiffSource::Commit { oid } => {
            let listing = git.run_read(&["rev-list", "--parents", "-n", "1", oid])?;
            base = listing
                .split_whitespace()
                .nth(1)
                .unwrap_or(EMPTY_TREE)
                .to_string();
            args.push(&base);
            args.push(oid);
            args.push("--");
            if let Some(o) = orig_path {
                args.push(o);
            }
            args.push(path);
        }
        DiffSource::Staged => args.extend(["--cached", "--", path]),
        DiffSource::Unstaged => args.extend(["--", path]),
        DiffSource::Untracked => {
            ok.push(1);
            args.extend(["--no-index", "--", "/dev/null", path]);
        }
        DiffSource::WorkingFile => return working_file(git, path, orig_path, context),
    }
    let text = git.run_ok_codes(&args, &ok)?;
    if too_large(&text) {
        let mut d = empty(path, orig_path);
        d.too_large = true;
        return Ok(d);
    }
    Ok(parse_unified(path, orig_path, &text))
}

fn working_file(
    git: &Git,
    path: &str,
    orig_path: Option<&str>,
    context: u32,
) -> GitResult<FileDiff> {
    let mut d = empty(path, orig_path);
    let bytes =
        std::fs::read(git.repo().join(path)).map_err(|e| super::error::GitError::Failed {
            message: format!("cannot read {path}: {e}"),
            stderr: String::new(),
        })?;
    if bytes.contains(&0) {
        d.binary = true;
        return Ok(d);
    }
    let text = String::from_utf8_lossy(&bytes);
    if too_large(&text) {
        d.too_large = true;
        return Ok(d);
    }
    let all: Vec<&str> = text.lines().collect();
    let n = all.len().min(context as usize);
    if n == 0 {
        return Ok(d);
    }
    let lines = all[..n]
        .iter()
        .enumerate()
        .map(|(i, l)| DiffLine {
            kind: LineKind::Context,
            old_no: Some(i as u32 + 1),
            new_no: Some(i as u32 + 1),
            text: (*l).to_string(),
        })
        .collect();
    d.hunks.push(Hunk {
        header: format!("@@ -1,{n} +1,{n} @@"),
        old_start: 1,
        old_lines: n as u32,
        new_start: 1,
        new_lines: n as u32,
        lines,
    });
    Ok(d)
}
