use serde::{Deserialize, Serialize};

/// The kind of reading the caller wants.
///
/// For the first MVP, we only support reading the current line.
/// Later, this enum can grow to include:
///
/// - CurrentScope
/// - CurrentFunctionSummary
/// - FunctionParameters
/// - CurrentContext
/// - DiagnosticsNearCursor
/// 
/// CurrentLine:
/// Reads or describes the physical line where the cursor is.
///
/// FunctionSummary:
/// Finds the function containing the cursor and summarizes that function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadRequest {
    CurrentLine,
    FunctionSummary,
}

/// The input sent into the code reader core.
///
/// This is deliberately not tied to VS Code.
/// The VS Code extension can translate editor state into this structure later.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReaderInput {
    pub language: String,
    pub source: String,
    pub cursor_line: usize,
    pub request: ReadRequest,
}