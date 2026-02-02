//! Agent reviewer trait: runs code review for a PR/MR and returns `ReviewResult`.
//!
//! The agent decides which MCP tools to call (e.g. fetch PR content, post review).
//! Implemented by LangGraph ReAct agent. Used by `ReviewPipeline`.

use crate::pr_url::PrUrl;
use crate::review_result::ReviewResult;

/// Runs agent code review for the given PR/MR. The agent decides when to call MCP (fetch/post).
pub trait AgentReviewer: Send + Sync {
    /// Performs review; `project_path` is optional and may be used for repo context.
    /// The agent fetches PR content and posts the result via its tools (MCP).
    fn review(
        &self,
        project_path: Option<&std::path::Path>,
        pr: &PrUrl,
    ) -> Result<ReviewResult, ReviewError>;
}

/// Errors from the agent review step (e.g. opencode-sdk session failure).
#[derive(Debug)]
pub struct ReviewError {
    pub message: String,
}

impl std::fmt::Display for ReviewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ReviewError {}
