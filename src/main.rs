//! Binary entry: parse PR/MR URL from args, run review pipeline (placeholder impls).
//!
//! In the skeleton, pipeline uses mock provider/reviewer; replace with real MCP + opencode-sdk later.

use quick_review::cli::{parse_pr_url_from_args, run_pipeline};
use quick_review::pr_url::PrUrl;
use quick_review::review_input::ReviewInput;
use quick_review::review_result::ReviewResult;
use quick_review::{AgentReviewer, McpProvider, ReviewPipeline};

/// Placeholder MCP provider: returns fixed input, no-op post.
struct MockMcpProvider;
impl McpProvider for MockMcpProvider {
    fn fetch(&self, _pr: &PrUrl) -> Result<ReviewInput, quick_review::mcp_provider::McpError> {
        Ok(ReviewInput::new()
            .with_title("Mock PR")
            .with_description("Placeholder")
            .with_diff(""))
    }
    fn post_review(
        &self,
        _pr: &PrUrl,
        _result: &ReviewResult,
    ) -> Result<(), quick_review::mcp_provider::McpError> {
        Ok(())
    }
}

/// Placeholder agent reviewer: returns fixed result.
struct MockAgentReviewer;
impl AgentReviewer for MockAgentReviewer {
    fn review(
        &self,
        _project_path: Option<&std::path::Path>,
        _input: &ReviewInput,
    ) -> Result<ReviewResult, quick_review::agent_reviewer::ReviewError> {
        Ok(ReviewResult::new().with_summary("Mock review summary (skeleton)."))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let pr = match parse_pr_url_from_args(&args) {
        Some(p) => p,
        None => {
            eprintln!("Usage: quick-review <PR_OR_MR_URL>");
            eprintln!("Example: quick-review https://github.com/owner/repo/pull/123");
            std::process::exit(1);
        }
    };
    let pipeline: ReviewPipeline<MockMcpProvider, MockAgentReviewer> =
        ReviewPipeline::new(MockMcpProvider, MockAgentReviewer);
    run_pipeline(&pipeline, &pr)?;
    Ok(())
}
