//! Pipeline that runs the agent for one PR/MR. The agent decides which MCP to call (fetch/post).
//!
//! Depends only on `AgentReviewer`. MCP is used by the agent via its tools.

use crate::agent_reviewer::AgentReviewer;
use crate::pr_url::PrUrl;
use crate::review_result::ReviewResult;

/// Runs the full review flow for one PR/MR. The agent holds MCP and calls it via tools.
pub struct ReviewPipeline<A> {
    pub agent: A,
    pub project_path: Option<std::path::PathBuf>,
}

impl<A: AgentReviewer> ReviewPipeline<A> {
    /// Creates a pipeline with the given agent reviewer (agent decides MCP calls).
    pub fn new(agent: A) -> Self {
        Self {
            agent,
            project_path: None,
        }
    }

    /// Sets optional project path for the agent reviewer (e.g. cloned repo).
    pub fn with_project_path(mut self, path: std::path::PathBuf) -> Self {
        self.project_path = Some(path);
        self
    }

    /// Runs agent review for the PR/MR. The agent fetches and posts via MCP tools. Returns the `ReviewResult` on success.
    pub fn run(&self, pr: &PrUrl) -> Result<ReviewResult, PipelineError> {
        self.agent
            .review(self.project_path.as_deref(), pr)
            .map_err(PipelineError::Review)
    }
}

/// Aggregated error for the pipeline (review step).
#[derive(Debug)]
pub enum PipelineError {
    Review(crate::agent_reviewer::ReviewError),
}

impl std::fmt::Display for PipelineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PipelineError::Review(e) => write!(f, "review: {}", e),
        }
    }
}

impl std::error::Error for PipelineError {}
