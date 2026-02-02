//! Review ToolSource: get_pr_context and submit_review tools, with result slot.
//!
//! Implements langgraph `ToolSource`. Holds `ReviewInput` and an
//! `Arc<RwLock<Option<ReviewResult>>>`; `submit_review` writes the result there.
//! Used by `LangGraphReviewAgent`; after invoke, the adapter reads from the slot.

use async_trait::async_trait;
use langgraph::{ToolCallContent, ToolSource, ToolSourceError, ToolSpec};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::review_input::ReviewInput;
use crate::review_result::{LineComment, ReviewResult};

/// Tool name for retrieving a part of the PR context.
pub const TOOL_GET_PR_CONTEXT: &str = "get_pr_context";
/// Tool name for submitting the final review result.
pub const TOOL_SUBMIT_REVIEW: &str = "submit_review";

/// Review-specific ToolSource: get_pr_context and submit_review.
///
/// Holds a clone of `ReviewInput` and a result slot. When the agent calls
/// `submit_review`, we parse args, validate, and write `ReviewResult` into the slot.
/// Only the first successful `submit_review` is stored; later calls are ignored.
pub struct ReviewToolSource {
    input: ReviewInput,
    result_slot: Arc<RwLock<Option<ReviewResult>>>,
}

impl ReviewToolSource {
    /// Creates a new ReviewToolSource with the given input and result slot.
    ///
    /// The same `result_slot` is passed to the adapter so it can read the result
    /// after `invoke` returns.
    pub fn new(input: ReviewInput, result_slot: Arc<RwLock<Option<ReviewResult>>>) -> Self {
        Self { input, result_slot }
    }

    /// Returns the list of tools (get_pr_context, submit_review) with JSON schemas.
    pub fn tool_specs() -> Vec<ToolSpec> {
        vec![
            ToolSpec {
                name: TOOL_GET_PR_CONTEXT.to_string(),
                description: Some("Retrieve a part of the PR: title, description, diff, or files.".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "part": {
                            "type": "string",
                            "enum": ["title", "description", "diff", "files"],
                            "description": "Which part of the PR to retrieve."
                        }
                    },
                    "required": ["part"]
                }),
            },
            ToolSpec {
                name: TOOL_SUBMIT_REVIEW.to_string(),
                description: Some("Submit the final code review. Call exactly once when done. Required: summary; optional: line_comments.".to_string()),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "summary": { "type": "string", "description": "Overall review summary." },
                        "line_comments": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "path": { "type": "string" },
                                    "line": { "type": "integer", "minimum": 1 },
                                    "body": { "type": "string" }
                                },
                                "required": ["path", "line", "body"]
                            },
                            "description": "Optional per-line comments."
                        }
                    },
                    "required": ["summary"]
                }),
            },
        ]
    }

    fn get_pr_context(&self, part: &str) -> String {
        match part {
            "title" => self.input.title.clone(),
            "description" => self.input.description.clone(),
            "diff" => self.input.diff.clone(),
            "files" => {
                let list: Vec<String> = self
                    .input
                    .files
                    .iter()
                    .map(|f| format!("{}", f.path))
                    .collect();
                list.join(", ")
            }
            _ => format!("Unknown part: {}", part),
        }
    }

    fn build_review_result(
        summary: String,
        line_comments: Option<Vec<LineCommentInput>>,
    ) -> ReviewResult {
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
        ReviewResult {
            summary,
            line_comments: comments,
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
impl ToolSource for ReviewToolSource {
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
                let text = self.get_pr_context(part);
                Ok(ToolCallContent { text })
            }
            TOOL_SUBMIT_REVIEW => {
                let summary = arguments
                    .get("summary")
                    .and_then(|v| v.as_str())
                    .map(String::from)
                    .ok_or_else(|| ToolSourceError::InvalidInput("submit_review: missing summary".to_string()))?;
                let line_comments: Option<Vec<LineCommentInput>> =
                    serde_json::from_value(arguments.get("line_comments").cloned().unwrap_or(json!([])))
                        .ok();
                let result = Self::build_review_result(summary, line_comments);
                let mut slot = self.result_slot.write().await;
                if slot.is_none() {
                    *slot = Some(result);
                }
                Ok(ToolCallContent {
                    text: "Review submitted.".to_string(),
                })
            }
            _ => Err(ToolSourceError::NotFound(name.to_string())),
        }
    }
}
