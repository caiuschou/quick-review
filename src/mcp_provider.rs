//! MCP provider trait: fetch PR/MR content and (optionally) post review back.
//!
//! Implemented by MCP client wrappers (e.g. github-mcp, gitlab-mcp). Used by `ReviewPipeline`.
//! Tests can use a mock that returns fixed `ReviewInput` and records `post_review` calls.

use crate::pr_url::PrUrl;
use crate::review_input::ReviewInput;
use crate::review_result::ReviewResult;

/// Fetches PR/MR content from GitHub or GitLab via MCP; may post review back.
pub trait McpProvider: Send + Sync {
    /// Fetches review input for the given PR/MR. Caller uses this before invoking `AgentReviewer`.
    fn fetch(&self, pr: &PrUrl) -> Result<ReviewInput, McpError>;

    /// Posts the review result to the PR/MR. Optional in v1 (e.g. output to stdout only).
    fn post_review(&self, pr: &PrUrl, result: &ReviewResult) -> Result<(), McpError>;
}

/// Errors from MCP operations (network, auth, parse).
#[derive(Debug)]
pub struct McpError {
    pub message: String,
}

impl std::fmt::Display for McpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for McpError {}
