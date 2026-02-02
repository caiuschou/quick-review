//! Parsed PR/MR URL: platform, owner, repo, and PR/MR id.
//!
//! Used by `McpProvider` to know which PR/MR to fetch and where to post review.
//! Parsed from strings like `https://github.com/owner/repo/pull/123` or GitLab MR URLs.

/// Supported platform for pull/merge requests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Platform {
    GitHub,
    GitLab,
}

/// Parsed PR (GitHub) or MR (GitLab) URL.
#[derive(Debug, Clone)]
pub struct PrUrl {
    pub platform: Platform,
    pub owner: String,
    pub repo: String,
    pub id: String,
}

impl PrUrl {
    /// Builds a `PrUrl` from known parts. Callers typically use `parse` from a URL string.
    pub fn new(platform: Platform, owner: String, repo: String, id: String) -> Self {
        Self {
            platform,
            owner,
            repo,
            id,
        }
    }

    /// Parses a GitHub PR or GitLab MR URL into `PrUrl`.
    /// Returns `None` if the URL format is not recognized.
    ///
    /// Example GitHub: `https://github.com/owner/repo/pull/123`
    /// Example GitLab: `https://gitlab.com/owner/repo/-/merge_requests/456`
    pub fn parse(url: &str) -> Option<Self> {
        let url = url.trim();
        if let Some(rest) = url.strip_prefix("https://github.com/") {
            let parts: Vec<&str> = rest.split('/').collect();
            if parts.len() >= 4 && parts[2] == "pull" {
                return Some(Self {
                    platform: Platform::GitHub,
                    owner: parts[0].to_string(),
                    repo: parts[1].to_string(),
                    id: parts[3].to_string(),
                });
            }
        }
        if let Some(rest) = url.strip_prefix("https://gitlab.com/") {
            let parts: Vec<&str> = rest.split('/').collect();
            if let Some(pos) = parts.iter().position(|&p| p == "-") {
                if pos + 2 < parts.len() && parts[pos + 1] == "merge_requests" {
                    let id = parts[pos + 2].to_string();
                    let (owner, repo) = (parts[0].to_string(), parts[1].to_string());
                    return Some(Self {
                        platform: Platform::GitLab,
                        owner,
                        repo,
                        id,
                    });
                }
            }
        }
        None
    }
}
