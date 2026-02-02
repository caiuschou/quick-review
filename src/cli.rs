//! CLI parsing and entry: accepts PR/MR URL, builds pipeline, runs review.
//!
//! Called from `main.rs`. Uses `PrUrl::parse` and `ReviewPipeline::run`.

use crate::pr_url::PrUrl;
use crate::review_pipeline::ReviewPipeline;

/// Parses CLI args (e.g. single positional PR/MR URL) and returns `PrUrl` if valid.
pub fn parse_pr_url_from_args(args: &[String]) -> Option<PrUrl> {
    let url = args.get(1)?;
    PrUrl::parse(url)
}

/// Runs the pipeline with the given reviewer. Prints result summary to stdout.
pub fn run_pipeline<A>(pipeline: &ReviewPipeline<A>, pr: &PrUrl) -> Result<(), Box<dyn std::error::Error>>
where
    A: crate::agent_reviewer::AgentReviewer,
{
    let result = pipeline.run(pr)?;
    println!("{}", result.summary);
    for c in &result.line_comments {
        println!("  {}:{} - {}", c.path, c.line, c.body);
    }
    Ok(())
}
