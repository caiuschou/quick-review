//! MCP-backed Review ToolSource: get_pr_context and submit_review call McpProvider.
//!
//! The agent decides when to call which tool: get_pr_context triggers mcp.fetch(pr),
//! submit_review builds the result and calls mcp.post_review(pr, result). Used by
//! `LangGraphReviewAgent` when running with PrUrl; after invoke, the adapter reads from the slot.

use async_trait::async_trait;
use langgraph::{ToolCallContent, ToolSource, ToolSourceError, ToolSpec};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::mcp_provider::{McpError, McpProvider};
use crate::pr_url::PrUrl;
use crate::review_input::ReviewInput;
use crate::review_result::{LineComment, ReviewResult};
use super::review_tools::{TOOL_GET_PR_CONTEXT, TOOL_SUBMIT_REVIEW};

/// MCP-backed tool source: get_pr_context calls mcp.fetch(pr), submit_review calls mcp.post_review.
///
/// Holds the PR URL and a result slot. On first get_pr_context, fetches via MCP and caches
/// ReviewInput; submit_review builds ReviewResult, writes to slot, and posts via MCP.
pub struct McpReviewToolSource {
    mcp: Arc<dyn McpProvider + Send + Sync>,
    pr: PrUrl,
    result_slot: Arc<RwLock<Option<ReviewResult>>>,
    cached: Arc<RwLock<Option<ReviewInput>>>,
}

impl McpReviewToolSource {
    /// Creates a new McpReviewToolSource for the given PR. Fetch happens on first get_pr_context.
    pub fn new(
        mcp: Arc<dyn McpProvider + Send + Sync>,
        pr: PrUrl,
        result_slot: Arc<RwLock<Option<ReviewResult>>>,
    ) -> Self {
        Self {
            mcp,
            pr,
            result_slot,
            cached: Arc::new(RwLock::new(None)),
        }
    }

    /// Returns the same tool specs as ReviewToolSource (get_pr_context, submit_review).
    pub fn tool_specs() -> Vec<ToolSpec> {
        super::review_tools::ReviewToolSource::tool_specs()
    }

    fn get_part_from_input(input: &ReviewInput, part: &str) -> String {
        match part {
            "title" => input.title.clone(),
            "description" => input.description.clone(),
            "diff" => input.diff.clone(),
            "files" => {
                let list: Vec<String> = input.files.iter().map(|f| f.path.clone()).collect();
                list.join(", ")
            }
            _ => format!("Unknown part: {}", part),
        }
    }
}

#[derive(serde::Deserialize)]
struct LineCommentInput {
    path: String,
    line: u32,
    body: String,
}

#[async_trait]
impl ToolSource for McpReviewToolSource {
    async fn list_tools(&self) -> Result<Vec<ToolSpec>, ToolSourceError> {
        Ok(Self::tool_specs())
    }

    async fn call_tool(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<ToolCallContent, ToolSourceError> {
        match name {
            TOOL_GET_PR_CONTEXT => {
                let part = arguments
                    .get("part")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let mut cached = self.cached.write().await;
                if cached.is_none() {
                    let input = self.mcp.fetch(&self.pr).map_err(|e: McpError| {
                        ToolSourceError::InvalidInput(format!("MCP fetch failed: {}", e))
                    })?;
                    *cached = Some(input);
                }
                let input = cached.as_ref().unwrap();
                let text = Self::get_part_from_input(input, part);
                Ok(ToolCallContent { text })
            }
            TOOL_SUBMIT_REVIEW => {
                let summary = arguments
                    .get("summary")
                    .and_then(|v| v.as_str())
                    .map(String::from)
                    .ok_or_else(|| {
                        ToolSourceError::InvalidInput("submit_review: missing summary".to_string())
                    })?;
                let line_comments: Option<Vec<LineCommentInput>> =
                    serde_json::from_value(arguments.get("line_comments").cloned().unwrap_or(json!([])))
                        .ok();
                let comments = line_comments
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|c| {
                        if c.line >= 1 && !c.path.is_empty() && !c.body.is_empty() {
                            Some(LineComment {
                                path: c.path,
                                line: c.line,
                                body: c.body,
                            })
                        } else {
                            None
                        }
                    })
                    .collect();
                let result = ReviewResult {
                    summary,
                    line_comments: comments,
                };
                self.mcp.post_review(&self.pr, &result).map_err(|e: McpError| {
                    ToolSourceError::InvalidInput(format!("MCP post_review failed: {}", e))
                })?;
                let mut slot = self.result_slot.write().await;
                if slot.is_none() {
                    *slot = Some(result);
                }
                Ok(ToolCallContent {
                    text: "Review submitted and posted via MCP.".to_string(),
                })
            }
            _ => Err(ToolSourceError::NotFound(name.to_string())),
        }
    }
}
