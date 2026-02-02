//! quick-review library: types and pipeline for agent-powered PR/MR review.
//!
//! Main flow: parse PR/MR URL → fetch content via MCP → run agent review (opencode-sdk) → post back.
//! See `review_pipeline::ReviewPipeline` for the orchestration type.

pub mod agent_reviewer;
pub mod cli;
pub mod mcp_provider;
pub mod pr_url;
pub mod review_input;
pub mod review_pipeline;
pub mod review_result;

pub use agent_reviewer::AgentReviewer;
pub use mcp_provider::McpProvider;
pub use pr_url::PrUrl;
pub use review_input::ReviewInput;
pub use review_pipeline::ReviewPipeline;
pub use review_result::ReviewResult;
