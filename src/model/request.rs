use serde::{Deserialize, Serialize};

/// The kind of reading the caller wants.
//
// For the first MVP, we only support reading the current line.
// Later, this enum can grow to include:
//
// ◯ = Done; X = Not Done
// - ◯ CurrentLine -- What does this exact line say?
// - X CurrentScope -- What local block currently controls this line?
// - ◯ FunctionSummary -- What function am I in, including signature details?
// - ◯ FunctionParameters -- What are this function's parameters?
// - ◯ CurrentContext -- Where am I in the file's larger structure?
// - X DiagnosticsNearCursor -- What problem near the cursor should I be aware of?
// 
// Clarifying Descriptions:
// CurrentLine:
// Reads or describes the physical line where the cursor is.
//
// FunctionSummary:
// Finds the function containing the cursor and summarizes that function.
// 
// FunctionParameters:
// Finds the function containing the cursor and reads only its parameters.
//
// CurrentContext:
// Describes where the cursor is, such as the current class and/or function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadRequest {
    CurrentLine,
    FunctionSummary,
    FunctionParameters,
    CurrentContext,
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