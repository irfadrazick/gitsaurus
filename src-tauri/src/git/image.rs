//! Both versions of an image file, for showing them side by side in place of a binary diff.
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde::{Deserialize, Serialize};

use super::error::{GitError, GitResult};
use super::ops::branch::safe_target;
use super::runner::Git;
use super::types::DiffSource;

/// Larger versions are reported by size only.
pub const MAX_IMAGE_BYTES: u64 = 20 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ImageVersion {
    pub size: u64,
    /// Base64 of the file's bytes; `None` when it is over `MAX_IMAGE_BYTES`.
    pub data: Option<String>,
}

/// `None` on a side means the file doesn't exist there (added or deleted).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ImageVersions {
    pub old: Option<ImageVersion>,
    pub new: Option<ImageVersion>,
}

/// Where one side of the comparison comes from.
enum Place {
    /// A `rev:path` or `:path` (index) object name.
    Object(String),
    WorkingTree,
    Missing,
}

pub fn image_versions(
    git: &Git,
    path: &str,
    orig_path: Option<&str>,
    source: &DiffSource,
) -> GitResult<ImageVersions> {
    let old_path = orig_path.unwrap_or(path);
    let (old, new) = match source {
        DiffSource::Commit { oid } => {
            safe_target(oid)?;
            let listing =
                git.run_read(&["rev-list", "--parents", "-n", "1", "--end-of-options", oid])?;
            let mut ids = listing.split_whitespace();
            let commit = ids.next().unwrap_or(oid).to_string();
            let old = match ids.next() {
                Some(parent) => Place::Object(format!("{parent}:{old_path}")),
                None => Place::Missing,
            };
            (old, Place::Object(format!("{commit}:{path}")))
        }
        DiffSource::Staged => (
            Place::Object(format!("HEAD:{old_path}")),
            Place::Object(format!(":{path}")),
        ),
        DiffSource::Unstaged => (Place::Object(format!(":{path}")), Place::WorkingTree),
        DiffSource::Untracked | DiffSource::WorkingFile => (Place::Missing, Place::WorkingTree),
    };
    Ok(ImageVersions {
        old: read(git, &old, old_path)?,
        new: read(git, &new, path)?,
    })
}

fn read(git: &Git, place: &Place, path: &str) -> GitResult<Option<ImageVersion>> {
    match place {
        Place::Missing => Ok(None),
        Place::WorkingTree => {
            let full = git.repo().join(path);
            let Ok(meta) = std::fs::metadata(&full) else {
                return Ok(None);
            };
            if !meta.is_file() {
                return Ok(None);
            }
            let size = meta.len();
            if size > MAX_IMAGE_BYTES {
                return Ok(Some(ImageVersion { size, data: None }));
            }
            let bytes = std::fs::read(&full).map_err(|e| GitError::Failed {
                message: format!("cannot read {path}: {e}"),
                stderr: String::new(),
            })?;
            Ok(Some(encode(bytes)))
        }
        Place::Object(name) => {
            // Exit 1 with no output: the file isn't in that commit or the index.
            let found = git.run_read_ok_codes(
                &["rev-parse", "--verify", "--quiet", "--end-of-options", name],
                &[0, 1],
            )?;
            let oid = found.trim();
            if oid.is_empty() {
                return Ok(None);
            }
            let size: u64 = git
                .run_read(&["cat-file", "-s", oid])?
                .trim()
                .parse()
                .unwrap_or(0);
            if size > MAX_IMAGE_BYTES {
                return Ok(Some(ImageVersion { size, data: None }));
            }
            Ok(Some(encode(
                git.run_read_bytes(&["cat-file", "blob", oid])?,
            )))
        }
    }
}

fn encode(bytes: Vec<u8>) -> ImageVersion {
    ImageVersion {
        size: bytes.len() as u64,
        data: Some(STANDARD.encode(bytes)),
    }
}
