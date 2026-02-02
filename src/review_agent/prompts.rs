//! System prompt and PR/user message text for the review ReAct agent.
//!
//! Used by `LangGraphReviewAgent` to build initial `ReActState` messages.
//! See `idea/langgraph-review-agent.md` for the prompt design.

use crate::pr_url::{Platform, PrUrl};
use crate::review_input::ReviewInput;

/// System prompt for the code review ReAct agent.
///
/// Tells the agent it is a code reviewer, describes the input (PR title, description, diff, files),
/// the tools (`get_pr_context` to read a part, `submit_review` to submit the final result),
/// and that it **must** call `submit_review` exactly once when done.
pub const REVIEW_SYSTEM_PROMPT: &str = r#"You are a code review agent. Your input is the current PR's title, description, diff, and file list.

RULES:
1. Use get_pr_context(part: "title" | "description" | "diff" | "files") to load PR content (call at least once).
2. When your review is complete, you MUST call submit_review once with:
   - summary: string (overall review summary, required)
   - line_comments: optional array of { path, line, body } for per-line comments (line >= 1).
3. If you do not call submit_review, the review will fail.
4. Be concise and focused; for line comments, cite file path and line number clearly."#;

/// Builds the initial user message from `PrUrl` when the agent fetches via MCP.
pub fn pr_url_to_user_message(pr: &PrUrl) -> String {
    let platform = match pr.platform {
        Platform::GitHub => "GitHub",
        Platform::GitLab => "GitLab",
    };
    format!(
        "Review the {} PR: {} / {} #{}.\nUse get_pr_context(part) to load title, description, diff, or files. When done, call submit_review.",
        platform, pr.owner, pr.repo, pr.id
    )
}

/// Builds the user message text from `ReviewInput` for the ReAct agent.
///
/// Format: Title, Description, Diff, then Files list. Matches the parts returned by
/// `get_pr_context` (title, description, diff, files). Used by `ReviewToolSource::get_pr_context`.
pub fn review_input_to_user_message(input: &ReviewInput) -> String {
    let files_list = input
        .files
        .iter()
        .map(|f| {
            let extra = match (&f.diff, &f.content) {
                (Some(_), Some(_)) => " (diff+content)",
                (Some(_), None) => " (diff)",
                (None, Some(_)) => " (content)",
                (None, None) => "",
            };
            format!("{}{}", f.path, extra)
        })
        .collect::<Vec<_>>()
        .join(", ");
    let n = input.files.len();
    format!(
        "Title: {}\n\nDescription: {}\n\nDiff:\n{}\n\nFiles ({}): {}",
        input.title,
        input.description,
        input.diff,
        n,
        if files_list.is_empty() {
            "(none)".to_string()
        } else {
            files_list
        }
    )
}
