use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Error, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum GitError {
    #[error("{message}")]
    NotARepo { message: String },
    #[error("{message}")]
    GitMissing { message: String },
    #[error("{message}")]
    PushRejected { message: String, stderr: String },
    #[error("{message}")]
    Failed { message: String, stderr: String },
}

pub type GitResult<T> = Result<T, GitError>;
