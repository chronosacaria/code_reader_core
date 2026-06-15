use serde::{Deserialize, Serialize};

/// The output returned by the code reader core.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReaderOutput {
    pub speech: String,
}