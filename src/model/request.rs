use serde::{Deserialize, Serialize};
use crate::model::ReaderDiagnostic;

/// The kind of reading the caller wants.
///
/// Clarifying Descriptions:
/// CurrentLine:
/// Reads or describes the physical line where the cursor is.
///
/// FunctionSummary:
/// Finds the function containing the cursor and summarizes that function.
/// 
/// FunctionParameters:
/// Finds the function containing the cursor and reads only its parameters.
///
/// CurrentContext:
/// Describes where the cursor is, such as the current class and/or function.
///
/// DiagnosticsNearCursor:
/// Reads diagnostics that are on or near the cursor line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadRequest {
    CurrentLine,
    CurrentScope,
    FunctionSummary,
    FunctionParameters,
    CurrentContext,
    DiagnosticsNearCursor,
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

    /// Diagnostics supplied by the caller.
    ///
    /// This field defaults to an empty list.
    #[serde(default)]
    pub diagnostics: Vec<ReaderDiagnostic>,
}