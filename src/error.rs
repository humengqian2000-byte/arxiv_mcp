use thiserror::Error;

#[derive(Error, Debug)]
pub enum ArxivError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Paper not found: {0}")]
    PaperNotFound(String),
}
