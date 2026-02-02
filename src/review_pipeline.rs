//! Pipeline that orchestrates: fetch (MCP) → review (agent) → post (MCP).
//!
//! Depends only on `McpProvider` and `AgentReviewer` traits. See `design.md` for the flow.

use crate::agent_reviewer::AgentReviewer;
use crate::mcp_provider::{McpError, McpProvider};
use crate::pr_url::PrUrl;
use crate::review_result::ReviewResult;

/// Runs the full review flow for one PR/MR.
pub struct ReviewPipeline<M, A> {
    pub mcp: M,
    pub agent: A,
    pub project_path: Option<std::path::PathBuf>,
}

impl<M: McpProvider, A: AgentReviewer> ReviewPipeline<M, A> {
    /// Creates a pipeline with the given MCP provider and agent reviewer.
    pub fn new(mcp: M, agent: A) -> Self {
        Self {
            mcp,
            agent,
            project_path: None,
        }
    }

    /// Sets optional project path for the agent reviewer (e.g. cloned repo).
    pub fn with_project_path(mut self, path: std::path::PathBuf) -> Self {
        self.project_path = Some(path);
        self
    }

    /// Fetches PR/MR content, runs agent review, and posts the result. Returns the `ReviewResult` on success.
    pub fn run(&self, pr: &PrUrl) -> Result<ReviewResult, PipelineError> {
        let input = self.mcp.fetch(pr).map_err(PipelineError::Fetch)?;
        let result = self
            .agent
            .review(self.project_path.as_deref(), &input)
            .map_err(PipelineError::Review)?;
        self.mcp.post_review(pr, &result).map_err(PipelineError::Post)?;
        Ok(result)
    }
}

/// Aggregated error for the pipeline (fetch / review / post).
#[derive(Debug)]
pub enum PipelineError {
    Fetch(McpError),
    Review(crate::agent_reviewer::ReviewError),
    Post(McpError),
}

impl std::fmt::Display for PipelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PipelineError::Fetch(e) => write!(f, "fetch: {}", e),
            PipelineError::Review(e) => write!(f, "review: {}", e),
            PipelineError::Post(e) => write!(f, "post: {}", e),
        }
    }
}

impl std::error::Error for PipelineError {}
