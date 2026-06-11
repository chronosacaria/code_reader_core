use serde::{Deserialize, Serialize};

/// The output returned by the code reader core.
///
/// For now, we only return a speech string.
/// Later, this can become richer and include structured data such as:
///
/// - the symbol name
/// - the enclosing function
/// - the enclosing class
/// - the exact source range
/// - severity information for diagnostics
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReaderOutput {
    pub speech: String,
}