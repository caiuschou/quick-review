//! Integration tests for review_agent tools: submit_review writes to result slot.
//!
//! BDD-style: given a ReviewToolSource with a result slot, when the agent (or we) call
//! submit_review with summary and optional line_comments, then the slot contains the
//! expected ReviewResult.

use langgraph::ToolSource;
use quick_review::review_agent::ReviewToolSource;
use quick_review::review_input::ReviewInput;
use quick_review::review_result::ReviewResult;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Scenario: Calling submit_review with summary only writes ReviewResult with empty line_comments.
#[tokio::test]
async fn submit_review_summary_only_writes_to_slot() {
    let input = ReviewInput::new().with_title("PR");
    let result_slot: Arc<RwLock<Option<ReviewResult>>> = Arc::new(RwLock::new(None));
    let tools = ReviewToolSource::new(input, result_slot.clone());

    let args = serde_json::json!({
        "summary": "Looks good."
    });
    let _ = tools.call_tool("submit_review", args).await.unwrap();

    let guard = result_slot.read().await;
    let result = guard.as_ref().expect("slot should have result");
    assert_eq!(result.summary, "Looks good.");
    assert!(result.line_comments.is_empty());
}

/// Scenario: Calling submit_review with summary and line_comments writes correct LineComments.
#[tokio::test]
async fn submit_review_with_line_comments_writes_to_slot() {
    let input = ReviewInput::new();
    let result_slot: Arc<RwLock<Option<ReviewResult>>> = Arc::new(RwLock::new(None));
    let tools = ReviewToolSource::new(input, result_slot.clone());

    let args = serde_json::json!({
        "summary": "A few nits.",
        "line_comments": [
            { "path": "src/lib.rs", "line": 10, "body": "Use Option here." }
        ]
    });
    let _ = tools.call_tool("submit_review", args).await.unwrap();

    let guard = result_slot.read().await;
    let result = guard.as_ref().expect("slot should have result");
    assert_eq!(result.summary, "A few nits.");
    assert_eq!(result.line_comments.len(), 1);
    assert_eq!(result.line_comments[0].path, "src/lib.rs");
    assert_eq!(result.line_comments[0].line, 10);
    assert_eq!(result.line_comments[0].body, "Use Option here.");
}
