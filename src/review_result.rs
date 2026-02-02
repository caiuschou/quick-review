//! Result of agent review: summary and optional line-level comments.
//!
//! Produced by `AgentReviewer::review`; consumed by the publish step (e.g. `McpProvider::post_review`).

/// A single comment attached to a line (file path + line number).
#[derive(Debug, Clone)]
pub struct LineComment {
    pub path: String,
    pub line: u32,
    pub body: String,
}

/// Full review result: summary text and optional per-line comments.
#[derive(Debug, Clone, Default)]
pub struct ReviewResult {
    pub summary: String,
    pub line_comments: Vec<LineComment>,
}

impl ReviewResult {
    /// Creates an empty result. Used by `AgentReviewer` implementations to fill and return.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder-style: set summary.
    pub fn with_summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = summary.into();
        self
    }

    /// Builder-style: set line comments.
    pub fn with_line_comments(mut self, line_comments: Vec<LineComment>) -> Self {
        self.line_comments = line_comments;
        self
    }
}
