use serde::{Deserialize, Serialize};

/// Search parameters
#[derive(Debug, Deserialize, Clone)]
pub struct SearchArgs {
    pub query: String,
    #[serde(default = "default_search_type")]
    pub searchtype: String,
    pub limit: Option<usize>,
    #[serde(default)]
    pub start: Option<usize>,
}

fn default_search_type() -> String {
    "all".to_string()
}

/// Download paper parameters
#[derive(Debug, Deserialize, Clone)]
pub struct DownloadArgs {
    pub paper_id: String,
}

/// Search and download papers parameters
#[derive(Debug, Deserialize, Clone)]
pub struct SearchAndDownloadArgs {
    pub query: String,
    #[serde(default = "default_search_type")]
    pub searchtype: String,
    pub limit: Option<usize>,
    #[serde(default)]
    pub start: Option<usize>,
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
}

fn default_output_dir() -> String {
    "./papers".to_string()
}

/// Paper metadata
#[derive(Debug, Serialize, Clone)]
pub struct Paper {
    pub id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub abstract_text: String,
    pub categories: Vec<String>,
    pub pdf_url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
}

/// Server configuration
#[derive(Debug, Clone)]
pub struct Config {
    pub user_agent: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string(),
        }
    }
}
