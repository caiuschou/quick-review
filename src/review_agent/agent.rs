//! LangGraphReviewAgent: builds ReAct graph per review, implements AgentReviewer.
//!
//! Holds a tokio Runtime and an LLM (e.g. MockLlm); in `review()` creates result slot
//! and ReviewToolSource, builds StateGraph (think → act → observe), compiles, runs
//! invoke via block_on, then reads the result slot. Maps AgentError to ReviewError.

use std::sync::Arc;

use langgraph::{
    ActNode, AgentError, Message, ObserveNode, ReActState, StateGraph, ThinkNode, END, START,
};
use tokio::sync::RwLock;

use crate::agent_reviewer::{AgentReviewer, ReviewError};
use crate::review_input::ReviewInput;
use crate::review_result::ReviewResult;
use crate::review_agent::prompts::{review_input_to_user_message, REVIEW_SYSTEM_PROMPT};
use crate::review_agent::review_tools::ReviewToolSource;

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
/// Builds the graph on each `review()` call with a fresh result slot and ReviewToolSource.
/// After invoke, reads the result from the slot; if the agent never called `submit_review`,
/// returns `ReviewError`.
pub struct LangGraphReviewAgent {
    runtime: tokio::runtime::Runtime,
    llm: Arc<dyn langgraph::LlmClient + Send + Sync>,
}

impl LangGraphReviewAgent {
    /// Creates an agent with the given LLM client (e.g. MockLlm or ChatOpenAI).
    pub fn new(llm: Arc<dyn langgraph::LlmClient + Send + Sync>) -> Result<Self, ReviewError> {
        let runtime = tokio::runtime::Runtime::new()
            .map_err(|e| ReviewError { message: e.to_string() })?;
        Ok(Self { runtime, llm })
    }

    /// Runs the ReAct graph for one review: think → act → observe (loop until END).
    /// Returns the result from the slot if submit_review was called; otherwise Err.
    fn run_review(&self, input: &ReviewInput) -> Result<ReviewResult, ReviewError> {
        let result_slot: Arc<RwLock<Option<ReviewResult>>> = Arc::new(RwLock::new(None));
        let tool_source = ReviewToolSource::new(input.clone(), result_slot.clone());

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

        let user_text = review_input_to_user_message(input);
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
        input: &ReviewInput,
    ) -> Result<ReviewResult, ReviewError> {
        self.run_review(input)
    }
}
