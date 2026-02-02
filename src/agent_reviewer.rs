//! Agent reviewer trait: runs code review over `ReviewInput` and returns `ReviewResult`.
//!
//! Implemented by opencode-sdk integration (session + prompt → assistant_reply). Used by `ReviewPipeline`.
//! Tests can use a mock that returns a fixed `ReviewResult`.

use crate::review_input::ReviewInput;
use crate::review_result::ReviewResult;

/// Runs agent code review on the given input. May use a local project path for context.
pub trait AgentReviewer: Send + Sync {
    /// Performs review; `project_path` is optional and may be used for repo context.
    fn review(
        &self,
        project_path: Option<&std::path::Path>,
        input: &ReviewInput,
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
