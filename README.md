# quick-review

**quick-review** is an AI-powered code review tool that reviews pull requests (GitHub) or merge requests (GitLab) and posts review results directly on the PR/MR.

## Overview

- **Input:** A link to a GitHub PR or GitLab MR.
- **Process:** Fetch PR/MR content via MCP, run an AI coding assistant (opencode-sdk) to perform the review, then sync the review back to the PR/MR.
- **Output:** Review comments and summary on the PR/MR.

## Architecture

```
PR/MR URL
    │
    ▼
┌─────────────────────────────────────┐
│  MCP (mcp-rust)                     │
│  Connect to github-mcp or           │
│  gitlab-mcp → fetch PR/MR content   │
│  (diff, files, description, etc.)   │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│  opencode-sdk                       │
│  Open session with project path +  │
│  chat content → AI code review      │
│  → assistant_reply (text, tools)    │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│  MCP / REST API                     │
│  Post review (comments, summary)    │
│  back to the PR/MR                  │
└─────────────────────────────────────┘
```

## Dependencies

| Component | Repository | Role |
|-----------|------------|------|
| **MCP client & servers** | [mcp-rust](https://github.com/caiuschou/mcp-rust) | Query PR/MR data and (where supported) post review via GitHub/GitLab MCP servers. |
| **AI code review** | [opencode-sdk](https://github.com/caiuschou/opencode-sdk) | Run AI coding assistant sessions and consume the agent’s reply for review text. |

- **mcp-rust** provides `mcp-client` to talk to **github-mcp** (PRs, repos, issues, etc.) and **gitlab-mcp** (MRs, projects, etc.). Auth: `GITHUB_TOKEN` for GitHub; config/`.env` for GitLab.
- **opencode-sdk** is used to open a session with a local project path and a chat prompt (e.g. “review this PR”), then read `assistant_reply` for the review content.

## Requirements

- **Rust** — Stable toolchain (see mcp-rust and opencode-sdk for version notes).
- **GitHub:** `GITHUB_TOKEN` for authenticated GitHub API/MCP access.
- **GitLab:** Configuration and `.env` as required by gitlab-mcp.

## Usage (planned)

1. Provide a PR or MR URL (e.g. from CLI or config).
2. quick-review uses MCP to fetch the PR/MR (diff, description, files).
3. Optionally clones or checks out the repo locally for `project_path`.
4. Calls opencode-sdk with a review prompt and gets the AI reply.
5. Posts the review (summary and/or line comments) back via MCP or REST API.

## Project status

This project is in early design/implementation. The README describes the intended architecture and integration points.

## License

TBD.

## References

- [opencode-sdk](https://github.com/caiuschou/opencode-sdk) — Rust SDK for AI coding assistant sessions.
- [mcp-rust](https://github.com/caiuschou/mcp-rust) — Rust MCP implementation (client, server, github-mcp, gitlab-mcp).
