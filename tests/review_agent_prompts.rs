//! Integration tests for review_agent prompts: ReviewInput → user message text.
//!
//! BDD-style: given a ReviewInput, when we build the user message, then the output
//! matches the expected format (Title, Description, Diff, Files).

use quick_review::review_agent::review_input_to_user_message;
use quick_review::review_input::{FileContent, ReviewInput};

/// Scenario: Empty ReviewInput produces a message with empty title/description/diff and "(none)" for files.
#[test]
fn review_input_to_user_message_empty_input_produces_none_for_files() {
    let input = ReviewInput::new();
    let msg = review_input_to_user_message(&input);
    assert!(msg.contains("Title: \n\nDescription: "));
    assert!(msg.contains("Diff:\n\n"));
    assert!(msg.contains("Files (0): (none)"));
}

/// Scenario: ReviewInput with title and description only produces the expected sections.
#[test]
fn review_input_to_user_message_title_and_description_only() {
    let input = ReviewInput::new()
        .with_title("Fix bug")
        .with_description("Fixes the null pointer.");
    let msg = review_input_to_user_message(&input);
    assert!(msg.contains("Title: Fix bug"));
    assert!(msg.contains("Description: Fixes the null pointer."));
    assert!(msg.contains("Files (0): (none)"));
}

/// Scenario: ReviewInput with files produces a comma-separated file list with (diff)/(content) hints.
#[test]
fn review_input_to_user_message_with_files_includes_paths_and_hints() {
    let input = ReviewInput::new()
        .with_title("T")
        .with_files(vec![
            FileContent {
                path: "a.rs".to_string(),
                diff: Some("".to_string()),
                content: None,
            },
            FileContent {
                path: "b.rs".to_string(),
                diff: None,
                content: Some("x".to_string()),
            },
        ]);
    let msg = review_input_to_user_message(&input);
    assert!(msg.contains("Files (2): "));
    assert!(msg.contains("a.rs (diff)"));
    assert!(msg.contains("b.rs (content)"));
}
