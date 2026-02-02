//! Binary entry: parse PR/MR URL from args, run review pipeline.
//!
//! Uses MockMcpProvider and LangGraphReviewAgent (ReAct). The agent decides when to call MCP (fetch/post); replace with real MCP and LLM for production.

use std::sync::Arc;

use langgraph::{MockLlm, ToolCall};
use quick_review::cli::{parse_pr_url_from_args, run_pipeline};
use quick_review::pr_url::PrUrl;
use quick_review::review_input::ReviewInput;
use quick_review::review_result::ReviewResult;
use quick_review::{LangGraphReviewAgent, McpProvider, ReviewPipeline};

/// Placeholder MCP provider: returns fixed input on fetch, no-op post.
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

    // Mock LLM: first round get_pr_context, then submit_review (agent decides MCP calls).
    let mock_llm = MockLlm::new(
        "",
        vec![
            ToolCall {
                name: "get_pr_context".to_string(),
                arguments: r#"{"part":"diff"}"#.to_string(),
                id: None,
            },
            ToolCall {
                name: "submit_review".to_string(),
                arguments: r#"{"summary":"Mock review from ReAct agent.","line_comments":[]}"#.to_string(),
                id: None,
            },
        ],
    );
    let mcp = Arc::new(MockMcpProvider);
    let agent = LangGraphReviewAgent::new(Arc::new(mock_llm), mcp)
        .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?;
    let pipeline = ReviewPipeline::new(agent);
    run_pipeline(&pipeline, &pr)?;
    Ok(())
}
