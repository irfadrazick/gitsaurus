use super::error::GitResult;
use super::runner::Git;
use super::types::RepoInfo;
use std::path::Path;

pub fn open_repo(path: &Path) -> GitResult<RepoInfo> {
    let out = Git::new(path).run_read(&["rev-parse", "--show-toplevel"])?;
    let top = out.trim_end_matches('\n');
    let name = Path::new(top)
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| top.to_string());
    Ok(RepoInfo {
        path: top.to_string(),
        name,
    })
}
