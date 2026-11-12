use std::collections::HashMap;

use crate::git::error::GitResult;
use crate::git::log::{RawCommit, read_log};
use crate::git::refs::list_refs;
use crate::git::runner::Git;
use crate::git::types::{GraphCommit, Line, RefKind, RefLabel};

/// Per-row drawing instructions; the renderer draws exactly this.
#[derive(Debug, Clone, PartialEq)]
pub struct RowLayout {
    pub lane: u16,
    pub color: u16,
    /// Lines arriving from above into this node.
    pub into: Vec<Line>,
    /// Lines leaving this node toward parents.
    pub out: Vec<Line>,
    /// Lines passing straight through this row.
    pub pass: Vec<Line>,
    pub width: u16,
}

type Slot = Option<(String, u16)>;

fn first_free(lanes: &mut Vec<Slot>) -> usize {
    match lanes.iter().position(Option::is_none) {
        Some(i) => i,
        None => {
            lanes.push(None);
            lanes.len() - 1
        }
    }
}

fn highest_occupied(lanes: &[Slot]) -> Option<usize> {
    lanes.iter().rposition(Option::is_some)
}

/// Assign lanes to commits (children before parents). Returns rows and max lane width.
pub fn layout(commits: &[RawCommit], head: Option<&str>) -> (Vec<RowLayout>, u16) {
    let mut next_color: u16 = 0;
    let mut alloc = || {
        let c = next_color;
        next_color += 1;
        c
    };
    let mut lanes: Vec<Slot> = Vec::new();
    if let Some(h) = head {
        lanes.push(Some((h.to_string(), alloc())));
    }
    let mut rows = Vec::with_capacity(commits.len());
    let mut max_lanes: u16 = 0;

    for c in commits {
        let before = highest_occupied(&lanes);
        let expecting: Vec<usize> = lanes
            .iter()
            .enumerate()
            .filter(|(_, s)| s.as_ref().is_some_and(|(o, _)| *o == c.oid))
            .map(|(i, _)| i)
            .collect();
        let (lane, color) = match expecting.first() {
            None => {
                let i = first_free(&mut lanes);
                (i, alloc())
            }
            Some(&i) => (i, lanes[i].as_ref().map(|s| s.1).unwrap_or(0)),
        };
        let line_at = |lanes: &[Slot], i: usize| Line {
            lane: i as u16,
            color: lanes[i].as_ref().map(|s| s.1).unwrap_or(0),
        };
        let into: Vec<Line> = expecting.iter().map(|&i| line_at(&lanes, i)).collect();
        let pass: Vec<Line> = (0..lanes.len())
            .filter(|i| lanes[*i].is_some() && !expecting.contains(i))
            .map(|i| line_at(&lanes, i))
            .collect();
        for &i in &expecting {
            lanes[i] = None;
        }

        let mut out = Vec::new();
        if let Some(p0) = c.parents.first() {
            // R1: the first parent always continues in the node's own lane.
            lanes[lane] = Some((p0.clone(), color));
            out.push(Line {
                lane: lane as u16,
                color,
            });
        }
        for p in c.parents.iter().skip(1) {
            let existing = lanes
                .iter()
                .position(|s| s.as_ref().is_some_and(|(o, _)| o == p));
            match existing {
                Some(j) => out.push(line_at(&lanes, j)),
                None => {
                    let k = first_free(&mut lanes);
                    let col = alloc();
                    lanes[k] = Some((p.clone(), col));
                    out.push(Line {
                        lane: k as u16,
                        color: col,
                    });
                }
            }
        }

        let after = highest_occupied(&lanes);
        let top = [Some(lane), before, after]
            .into_iter()
            .flatten()
            .max()
            .unwrap_or(lane);
        let width = (top + 1) as u16;
        max_lanes = max_lanes.max(width);
        rows.push(RowLayout {
            lane: lane as u16,
            color,
            into,
            out,
            pass,
            width,
        });
    }
    (rows, max_lanes)
}

/// Non-empty body lines joined by single spaces, cut at 200 chars.
/// Trailers that add nothing to a one-line preview (co-authors show as avatars in details).
const TRAILERS: &[&str] = &[
    "co-authored-by:",
    "signed-off-by:",
    "reviewed-by:",
    "acked-by:",
    "tested-by:",
    "reported-by:",
    "helped-by:",
    "change-id:",
];

fn is_trailer(line: &str) -> bool {
    let l = line.to_ascii_lowercase();
    TRAILERS.iter().any(|t| l.starts_with(t))
}

pub fn body_preview(body: &str) -> String {
    let joined = body
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !is_trailer(l))
        .collect::<Vec<_>>()
        .join(" ");
    joined.chars().take(200).collect()
}

/// Per-repo graph cache that the frontend pages rows from.
#[derive(Debug, Clone)]
pub struct GraphData {
    pub commits: Vec<GraphCommit>,
    pub by_oid: HashMap<String, u32>,
    pub max_lanes: u16,
    pub head_index: Option<u32>,
}

pub fn build_graph(git: &Git) -> GitResult<GraphData> {
    let raw = read_log(git)?;
    let refs = list_refs(git)?;
    let head_oid = refs.head.oid.as_deref();
    let (rows, max_lanes) = layout(&raw, head_oid);

    let mut labels: HashMap<String, Vec<RefLabel>> = HashMap::new();
    let mut add = |oid: &str, kind, name: &str, is_head| {
        labels.entry(oid.to_string()).or_default().push(RefLabel {
            kind,
            name: name.to_string(),
            is_head,
        });
    };
    // HEAD branch first, then the other local branches.
    for b in refs.local.iter().filter(|b| b.is_head) {
        add(&b.oid, RefKind::Local, &b.name, true);
    }
    for b in refs.local.iter().filter(|b| !b.is_head) {
        add(&b.oid, RefKind::Local, &b.name, false);
    }
    for b in &refs.remote {
        add(&b.oid, RefKind::Remote, &b.full_name, false);
    }
    for t in &refs.tags {
        add(&t.oid, RefKind::Tag, &t.name, false);
    }

    let mut by_oid = HashMap::with_capacity(raw.len());
    let mut head_index = None;
    let commits: Vec<GraphCommit> = raw
        .into_iter()
        .zip(rows)
        .enumerate()
        .map(|(i, (c, r))| {
            let index = i as u32;
            by_oid.insert(c.oid.clone(), index);
            if head_oid == Some(c.oid.as_str()) {
                head_index = Some(index);
            }
            GraphCommit {
                index,
                short_oid: c.oid.chars().take(7).collect(),
                refs: labels.remove(c.oid.as_str()).unwrap_or_default(),
                body_preview: body_preview(&c.body),
                oid: c.oid,
                parents: c.parents,
                author_name: c.author_name,
                author_email: c.author_email,
                author_time: c.author_time,
                summary: c.summary,
                lane: r.lane,
                color: r.color,
                into: r.into,
                out: r.out,
                pass: r.pass,
                width: r.width,
            }
        })
        .collect();
    Ok(GraphData {
        commits,
        by_oid,
        max_lanes,
        head_index,
    })
}

/// Case-insensitive match on summary/author, or oid prefix (query >= 4 hex chars).
pub fn search(data: &GraphData, query: &str) -> Vec<u32> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Vec::new();
    }
    let hex = q.len() >= 4 && q.chars().all(|c| c.is_ascii_hexdigit());
    data.commits
        .iter()
        .filter(|c| {
            c.summary.to_lowercase().contains(&q)
                || c.author_name.to_lowercase().contains(&q)
                || c.author_email.to_lowercase().contains(&q)
                || (hex && c.oid.starts_with(&q))
        })
        .map(|c| c.index)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::log::RawCommit;

    /// Shadows `super::layout` so every fixture is also checked for continuity.
    fn layout(commits: &[RawCommit], head: Option<&str>) -> (Vec<RowLayout>, u16) {
        let out = super::layout(commits, head);
        assert_rows_continuous(&out.0);
        out
    }

    fn assert_rows_continuous(rows: &[RowLayout]) {
        use std::collections::BTreeSet;
        for (i, w) in rows.windows(2).enumerate() {
            let a: BTreeSet<(u16, u16)> = w[0]
                .out
                .iter()
                .chain(&w[0].pass)
                .map(|l| (l.lane, l.color))
                .collect();
            let b: BTreeSet<(u16, u16)> = w[1]
                .into
                .iter()
                .chain(&w[1].pass)
                .map(|l| (l.lane, l.color))
                .collect();
            assert_eq!(a, b, "rows {i} -> {} discontinuous", i + 1);
        }
    }

    fn c(oid: &str, parents: &[&str]) -> RawCommit {
        RawCommit {
            oid: oid.into(),
            parents: parents.iter().map(|p| p.to_string()).collect(),
            author_name: String::new(),
            author_email: String::new(),
            author_time: 0,
            summary: String::new(),
            body: String::new(),
        }
    }
    fn lanes(ls: &[crate::git::Line]) -> Vec<u16> {
        ls.iter().map(|l| l.lane).collect()
    }

    #[test]
    fn linear_history_single_lane() {
        let (rows, max) = layout(
            &[c("c3", &["c2"]), c("c2", &["c1"]), c("c1", &[])],
            Some("c3"),
        );
        assert_eq!(max, 1);
        assert!(rows.iter().all(|r| r.lane == 0 && r.color == rows[0].color));
        assert!(rows[2].out.is_empty());
    }

    #[test]
    fn body_preview_joins_and_truncates() {
        assert_eq!(body_preview("\n\nA\n\nB  \n"), "A B");
        assert_eq!(body_preview(&"x".repeat(500)).chars().count(), 200);
    }

    #[test]
    fn body_preview_drops_trailers() {
        assert_eq!(
            body_preview(
                "Explain why.\n\nCo-Authored-By: Claude <noreply@anthropic.com>\nSigned-off-by: A <a@b.c>"
            ),
            "Explain why."
        );
        assert_eq!(body_preview("Co-authored-by: X <x@y.z>"), "");
    }

    #[test]
    fn empty_has_zero_lanes() {
        assert_eq!(layout(&[], None), (vec![], 0));
    }

    fn merge_rows() -> Vec<RowLayout> {
        layout(
            &[
                c("m", &["a", "b"]),
                c("b", &["base"]),
                c("a", &["base"]),
                c("base", &[]),
            ],
            Some("m"),
        )
        .0
    }

    #[test]
    fn merge_opens_second_lane_and_fork_closes_it() {
        let rows = merge_rows();
        assert_eq!(lanes(&rows[0].out), vec![0, 1]);
        assert_eq!(rows[1].lane, 1);
        assert_eq!(rows[2].lane, 0);
        assert_eq!(lanes(&rows[3].into), vec![0, 1]);
        assert_eq!(rows[3].lane, 0);
        assert_ne!(rows[0].color, rows[1].color);
    }

    #[test]
    fn head_chain_stays_lane_0_when_other_tip_is_newer() {
        let rows = layout(
            &[c("f", &["base"]), c("h", &["base"]), c("base", &[])],
            Some("h"),
        )
        .0;
        assert_eq!(rows[1].lane, 0);
        assert_eq!(rows[0].lane, 1);
    }

    #[test]
    fn freed_lane_is_reused() {
        // m1 merges s1; m2 merges s2; both side branches land in lane 1.
        let rows = layout(
            &[
                c("m2", &["m1", "s2"]),
                c("s2", &["m1"]),
                c("m1", &["base", "s1"]),
                c("s1", &["base"]),
                c("base", &[]),
            ],
            Some("m2"),
        )
        .0;
        assert_eq!(rows[1].lane, 1);
        assert_eq!(rows[3].lane, 1);
    }

    #[test]
    fn pass_lines_exclude_node_lanes() {
        let rows = merge_rows();
        assert_eq!(lanes(&rows[1].pass), vec![0]);
        assert!(rows[0].pass.is_empty());
    }

    #[test]
    fn second_parent_joins_lane_that_already_expects_it() {
        // m's parents: a, x. Lane 1 (opened by tip t) already expects x.
        // head chain: m -> a -> x; t -> x on lane 1.
        let rows = layout(
            &[
                c("t", &["x"]),
                c("m", &["a", "x"]),
                c("a", &["x"]),
                c("x", &[]),
            ],
            Some("m"),
        )
        .0;
        assert_eq!(rows[0].lane, 1);
        assert_eq!(rows[1].lane, 0);
        // m's second parent x joins existing lane 1, not a new lane.
        assert_eq!(lanes(&rows[1].out), vec![0, 1]);
        assert!(rows.iter().all(|r| r.width <= 2));
    }

    #[test]
    fn octopus_merge_joins_and_opens() {
        let (rows, max) = layout(
            &[
                c("t", &["p3"]),
                c("o", &["p1", "p2", "p3"]),
                c("p3", &[]),
                c("p2", &[]),
                c("p1", &[]),
            ],
            Some("o"),
        );
        assert_eq!(rows[0].lane, 1);
        // p1 -> lane 0, p2 -> first free slot (2), p3 joins lane 1.
        assert_eq!(lanes(&rows[1].out), vec![0, 2, 1]);
        assert_eq!(max, 3);
    }

    #[test]
    fn first_parent_stays_in_own_lane_and_duplicates_converge() {
        // R1: h and f both have first parent base; h keeps lane 0, f lane 1.
        let rows = layout(
            &[c("f", &["base"]), c("h", &["base"]), c("base", &[])],
            Some("h"),
        )
        .0;
        assert_eq!(lanes(&rows[2].into), vec![0, 1]);
        assert_eq!(rows[2].lane, 0);
    }
}
