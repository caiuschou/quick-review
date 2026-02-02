//! Input for the agent reviewer: diff, description, and file list fetched via MCP.
//!
//! Produced by `McpProvider::fetch`; consumed by `AgentReviewer::review`.

/// One file's metadata and content (or diff) for review.
#[derive(Debug, Clone, Default)]
pub struct FileContent {
    pub path: String,
    pub diff: Option<String>,
    pub content: Option<String>,
}

/// Aggregated input for a single PR/MR review.
#[derive(Debug, Clone, Default)]
pub struct ReviewInput {
    pub title: String,
    pub description: String,
    pub diff: String,
    pub files: Vec<FileContent>,
}

impl ReviewInput {
    /// Creates an empty `ReviewInput`. Used by MCP layer to fill and pass to `AgentReviewer`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder-style: set title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Builder-style: set description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Builder-style: set full diff.
    pub fn with_diff(mut self, diff: impl Into<String>) -> Self {
        self.diff = diff.into();
        self
    }

    /// Builder-style: set file list.
    pub fn with_files(mut self, files: Vec<FileContent>) -> Self {
        self.files = files;
        self
    }
}
