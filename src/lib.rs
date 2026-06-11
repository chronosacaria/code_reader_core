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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReadRequest {
    CurrentLine,
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

/// Main entry point into the reusable code reader core.
///
/// This function receives editor-like input and returns speech-friendly output.
/// It does not know anything about VS Code, terminals, or speech engines.
pub fn read_code(input: ReaderInput) -> ReaderOutput {
    match input.request {
        ReadRequest::CurrentLine => read_current_line(&input),
    }
}

/// Reads the current line from the source text and converts it into a simple
/// speech-friendly sentence.
///
/// This is intentionally simple for the first version.
/// Later, this function will be replaced by language-aware parsing.
fn read_current_line(input: &ReaderInput) -> ReaderOutput {
    let line_text = get_line(&input.source, input.cursor_line)
        .unwrap_or("")
        .trim();

    if line_text.is_empty() {
        return ReaderOutput {
            speech: "Blank line.".to_string(),
        };
    }

    let spoken_line = make_simple_speech_text(line_text);

    ReaderOutput {
        speech: format!("Current line: {spoken_line}"),
    }
}

/// Gets one line from a source string using a zero-based line number.
///
/// Zero-based means:
///
/// - line 0 is the first line
/// - line 1 is the second line
/// - line 2 is the third line
fn get_line(source: &str, cursor_line: usize) -> Option<&str> {
    source.lines().nth(cursor_line)
}

/// Converts a punctuation-heavy programming line into a very rough speech form.
///
/// This is not the final speech system.
/// It is just enough to prove that the CLI and tests work.
fn make_simple_speech_text(line: &str) -> String {
    line.replace("(", " ")
        .replace(")", " ")
        .replace(":", " ")
        .replace(",", " ")
        .replace("_", " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_current_python_function_line() {
        let input = ReaderInput {
            language: "python".to_string(),
            source: "def calculate_total(price, tax_rate):\n    return price * tax_rate".to_string(),
            cursor_line: 0,
            request: ReadRequest::CurrentLine,
        };

        let output = read_code(input);

        assert_eq!(
            output.speech,
            "Current line: def calculate total price tax rate"
        );
    }

    #[test]
    fn reads_blank_line() {
        let input = ReaderInput {
            language: "python".to_string(),
            source: "def example():\n\n    return 1".to_string(),
            cursor_line: 1,
            request: ReadRequest::CurrentLine,
        };

        let output = read_code(input);

        assert_eq!(output.speech, "Blank line.");
    }

    #[test]
    fn handles_cursor_line_past_end_of_file() {
        let input = ReaderInput {
            language: "python".to_string(),
            source: "print('hello')".to_string(),
            cursor_line: 20,
            request: ReadRequest::CurrentLine,
        };

        let output = read_code(input);

        assert_eq!(output.speech, "Blank line.");
    }

    #[test]
    fn deserializes_json_input() {
        let json = r#"
        {
            "language": "python",
            "source": "def calculate_total(price, tax_rate):",
            "cursor_line": 0,
            "request": "current_line"
        }
        "#;

        let input: ReaderInput = serde_json::from_str(json).unwrap();

        assert_eq!(input.language, "python");
        assert_eq!(input.cursor_line, 0);
        assert_eq!(input.request, ReadRequest::CurrentLine);
    }

    #[test]
    fn serializes_json_output() {
        let output = ReaderOutput {
            speech: "Current line: def example".to_string(),
        };

        let json = serde_json::to_string(&output).unwrap();

        assert_eq!(json, r#"{"speech":"Current line: def example"}"#);
    }
}