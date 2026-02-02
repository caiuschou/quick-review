//! Review agent: LangGraph ReAct agent that implements `AgentReviewer`.
//!
//! Uses tools `get_pr_context` and `submit_review`; result is read from a slot
//! after invoke. See `idea/langgraph-review-agent.md`.

mod agent;
mod mcp_review_tools;
mod prompts;
mod review_tools;

pub use agent::LangGraphReviewAgent;
pub use prompts::{review_input_to_user_message, REVIEW_SYSTEM_PROMPT};
pub use review_tools::{ReviewToolSource, TOOL_GET_PR_CONTEXT, TOOL_SUBMIT_REVIEW};
