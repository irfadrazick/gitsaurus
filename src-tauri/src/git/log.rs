use crate::git::error::GitResult;
use crate::git::refs;
use crate::git::runner::Git;

#[derive(Debug, Clone, PartialEq)]
pub struct RawCommit {
    pub oid: String,
    pub parents: Vec<String>,
    pub author_name: String,
    pub author_email: String,
    pub author_time: i64,
    pub summary: String,
    pub body: String,
}

/// Parse `-z` log output with fields separated by 0x1f. Lossy on bad UTF-8.
pub fn parse_log(raw: &[u8]) -> Vec<RawCommit> {
    String::from_utf8_lossy(raw)
        .split('\0')
        .filter_map(|rec| {
            let rec = rec.trim_start_matches('\n');
            if rec.is_empty() {
                return None;
            }
            let mut f = rec.splitn(7, '\x1f');
            let oid = f.next()?.to_string();
            let parents = f.next()?.split_whitespace().map(String::from).collect();
            Some(RawCommit {
                oid,
                parents,
                author_name: f.next()?.to_string(),
                author_email: f.next()?.to_string(),
                author_time: f.next()?.parse().unwrap_or(0),
                summary: f.next()?.to_string(),
                body: f.next().unwrap_or("").to_string(),
            })
        })
        .collect()
}

pub fn read_log(git: &Git) -> GitResult<Vec<RawCommit>> {
    let head = refs::head(git)?;
    let mut args = vec!["log", "--branches", "--remotes", "--tags"];
    if head.oid.is_some() {
        args.push("HEAD");
    } else if git
        .run_read(&["for-each-ref", "--count=1", "--format=%(refname)"])?
        .trim()
        .is_empty()
    {
        return Ok(vec![]);
    }
    args.extend([
        "--date-order",
        "-z",
        "--format=%H%x1f%P%x1f%an%x1f%ae%x1f%at%x1f%s%x1f%b",
    ]);
    Ok(parse_log(git.run_read(&args)?.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_log_handles_multiline_crlf_emoji_and_bad_utf8() {
        let raw = b"aaa\x1f\x1fAn\x1fa@x\x1f1700000000\x1ffix \xF0\x9F\x90\xA6 thing\x1fline1\r\nline2\n\0bbb\x1faaa\x1fB\x1fb@x\x1f1700000001\x1fbad \xff byte\x1f\0";
        let v = parse_log(raw);
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].summary, "fix 🐦 thing");
        assert!(v[0].body.contains("line2"));
        assert!(v[0].parents.is_empty());
        assert!(v[1].summary.starts_with("bad "));
        assert_eq!(v[1].parents, vec!["aaa"]);
    }
}
