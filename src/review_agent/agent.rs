//! LangGraphReviewAgent: builds ReAct graph per review, implements AgentReviewer.
//!
//! Holds a tokio Runtime, an LLM, and an McpProvider. In `review(pr)` creates result slot
//! and McpReviewToolSource (tools call MCP: get_pr_context → fetch, submit_review → post),
//! builds StateGraph (think → act → observe), compiles, runs invoke via block_on, then
//! reads the result slot. Which MCP to call is decided by the agent at runtime.

use std::sync::Arc;

use langgraph::{
    ActNode, AgentError, Message, ObserveNode, ReActState, StateGraph, ThinkNode, END, START,
};
use tokio::sync::RwLock;

use crate::agent_reviewer::{AgentReviewer, ReviewError};
use crate::mcp_provider::McpProvider;
use crate::pr_url::PrUrl;
use crate::review_result::ReviewResult;
use crate::review_agent::mcp_review_tools::McpReviewToolSource;
use crate::review_agent::prompts::{pr_url_to_user_message, REVIEW_SYSTEM_PROMPT};

/// Wrapper so we can share an `Arc<dyn LlmClient>` with ThinkNode (which takes Box<dyn LlmClient>).
/// Delegates invoke to the inner client.
struct SharedLlm(pub Arc<dyn langgraph::LlmClient + Send + Sync>);

#[async_trait::async_trait]
impl langgraph::LlmClient for SharedLlm {
    async fn invoke(
        &self,
        messages: &[Message],
    ) -> Result<langgraph::LlmResponse, AgentError> {
        self.0.invoke(messages).await
    }
}

/// Review agent that runs a langgraph ReAct graph (Think → Act → Observe) per review.
///
/// Holds an McpProvider; tools (McpReviewToolSource) call MCP on the agent's behalf.
/// Builds the graph on each `review(pr)` with a fresh result slot and McpReviewToolSource.
/// After invoke, reads the result from the slot; if the agent never called `submit_review`,
/// returns `ReviewError`.
pub struct LangGraphReviewAgent {
    runtime: tokio::runtime::Runtime,
    llm: Arc<dyn langgraph::LlmClient + Send + Sync>,
    mcp: Arc<dyn McpProvider + Send + Sync>,
}

impl LangGraphReviewAgent {
    /// Creates an agent with the given LLM client and MCP provider. The agent decides when to call MCP.
    pub fn new(
        llm: Arc<dyn langgraph::LlmClient + Send + Sync>,
        mcp: Arc<dyn McpProvider + Send + Sync>,
    ) -> Result<Self, ReviewError> {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| ReviewError { message: e.to_string() })?;
        Ok(Self { runtime, llm, mcp })
    }

    /// Runs the ReAct graph for one review: think → act → observe (loop until END).
    /// Tools call MCP (get_pr_context → fetch, submit_review → post). Returns the result from the slot if submit_review was called; otherwise Err.
    fn run_review(&self, pr: &PrUrl) -> Result<ReviewResult, ReviewError> {
        let result_slot: Arc<RwLock<Option<ReviewResult>>> = Arc::new(RwLock::new(None));
        let tool_source = McpReviewToolSource::new(self.mcp.clone(), pr.clone(), result_slot.clone());

        let think = ThinkNode::new(Box::new(SharedLlm(self.llm.clone())));
        let act = ActNode::new(Box::new(tool_source));
        let observe = ObserveNode::new();

        let mut graph = StateGraph::<ReActState>::new();
        graph
            .add_node("think", Arc::new(think))
            .add_node("act", Arc::new(act))
            .add_node("observe", Arc::new(observe))
            .add_edge(START, "think")
            .add_edge("think", "act")
            .add_edge("act", "observe")
            .add_edge("observe", END);

        let compiled = graph
            .compile()
            .map_err(|e| ReviewError { message: e.to_string() })?;

        let user_text = pr_url_to_user_message(pr);
        let state = ReActState {
            messages: vec![
                Message::system(REVIEW_SYSTEM_PROMPT.to_string()),
                Message::user(user_text),
            ],
            tool_calls: vec![],
            tool_results: vec![],
        };

        let slot = result_slot.clone();
        let run = async move {
            compiled.invoke(state, None).await?;
            let guard = slot.read().await;
            Ok(guard.clone())
        };

        let outcome = self
            .runtime
            .block_on(run)
            .map_err(|e: AgentError| ReviewError { message: e.to_string() })?;

        outcome.ok_or_else(|| ReviewError {
            message: "review agent did not call submit_review".to_string(),
        })
    }
}

impl AgentReviewer for LangGraphReviewAgent {
    fn review(
        &self,
        _project_path: Option<&std::path::Path>,
        pr: &PrUrl,
    ) -> Result<ReviewResult, ReviewError> {
        self.run_review(pr)
    }
}
