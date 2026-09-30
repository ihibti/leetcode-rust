use crate::leetcode::{FetchError, ProblemData};
use std::time::Duration;

#[derive(Debug)]
pub enum RemoteError {
    NotFound,
    Other(String),
}

pub trait Remote {
    fn get_text(&self, url: &str) -> Result<String, RemoteError>;
    fn problem(&self, slug: &str) -> Result<ProblemData, FetchError>;
}

pub struct UreqRemote;

impl Remote for UreqRemote {
    fn get_text(&self, url: &str) -> Result<String, RemoteError> {
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(10)))
            .user_agent("leetcode-rust-workspace/0.1")
            .build()
            .new_agent();

        let mut response = agent.get(url).call().map_err(|e| match e {
            ureq::Error::StatusCode(404) => RemoteError::NotFound,
            other => RemoteError::Other(other.to_string()),
        })?;

        response
            .body_mut()
            .read_to_string()
            .map_err(|e| RemoteError::Other(e.to_string()))
    }

    fn problem(&self, slug: &str) -> Result<ProblemData, FetchError> {
        crate::leetcode::fetch_problem(slug)
    }
}
