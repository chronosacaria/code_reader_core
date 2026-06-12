pub mod languages;
pub mod model;
pub mod speech;

pub use model::{ReadRequest, ReaderInput, ReaderOutput};

use speech::make_simple_speech_text;

/// Main entry point into the reusable code reader core.
///
/// This function receives editor-like input and returns speech-friendly output.
/// It does not know anything about VS Code, terminals, or speech engines.
pub fn read_code(input: ReaderInput) -> ReaderOutput {
    match input.request {
        ReadRequest::CurrentLine => read_current_line(&input),
        ReadRequest::FunctionSummary => read_function_summary(&input),
    }
}

/// Reads the current line from the source text.

// This first asks the appropriate language module whether it is able
// to produce a structural description. If it is unable to do so, it
// will fall back to simple speech.
fn read_current_line(input: &ReaderInput) -> ReaderOutput {
    let line_text = get_line(&input.source, input.cursor_line)
        .unwrap_or("")
        .trim();

    if line_text.is_empty() {
        return ReaderOutput {
            speech: "Blank line.".to_string(),
        };
    }

    // For now, Python is hard coded, but in the future, the plan is that
    // this will be more dynamic. Some other languages modules will need
    // to be made before this can be tested and implemented.
    if input.language.eq_ignore_ascii_case("python") {
        if let Some(speech) =
            languages::python::describe_current_line(&input.source, input.cursor_line)
        {
            return ReaderOutput { speech };
        }
    }

    let spoken_line = make_simple_speech_text(line_text);

    ReaderOutput {
        speech: format!("Current line: {spoken_line}"),
    }
}

/// Summarises the function containing the cursor.
/// 
/// For now, this only works for Python, but will be expanded in the future.
fn read_function_summary(input: &ReaderInput) -> ReaderOutput {
    if input.language.eq_ignore_ascii_case("python") {
        if let Some(speech) =
            languages::python::describe_function_summary(&input.source, input.cursor_line)
        {
            return ReaderOutput { speech };
        }

        return ReaderOutput {
            speech: "No Python function was found at the cursor.".to_string(),
        };
    }

    ReaderOutput {
        speech: format!(
            "Function summaries are not supported for {} yet.",
            input.language
        ),
    }
}

/// Gets one line from a source string using a zero-based line number.
fn get_line(source: &str, cursor_line: usize) -> Option<&str> {
    source.lines().nth(cursor_line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_current_python_function_line_structurally() {
        let input = ReaderInput {
            language: "python".to_string(),
            source: "def calculate_total(price, tax_rate):\n    return price * tax_rate".to_string(),
            cursor_line: 0,
            request: ReadRequest::CurrentLine,
        };

        let output = read_code(input);

        assert_eq!(
            output.speech,
            "Function calculate total. Parameters: price; tax rate."
        );
    }

    #[test]
    fn reads_current_typed_python_function_line_structurally() {
        let input = ReaderInput {
            language: "python".to_string(),
            source: "def calculate_total(price: float, tax_rate: float = 0.19) -> float:\n    return price * tax_rate".to_string(),
            cursor_line: 0,
            request: ReadRequest::CurrentLine,
        };

        let output = read_code(input);

        assert_eq!(
            output.speech,
            "Function calculate total. Parameters: price, float; tax rate, float, default zero point one nine. Returns float."
        );
    }


    #[test]
    fn falls_back_to_simple_speech_for_python_return_line() {
        let input = ReaderInput {
            language: "python".to_string(),
            source: "def calculate_total(price, tax_rate):\n    return price * tax_rate".to_string(),
            cursor_line: 1,
            request: ReadRequest::CurrentLine,
        };

        let output = read_code(input);

        assert_eq!(output.speech, "Current line: return price * tax rate");
    }

    #[test]
    fn summarizes_python_function_when_cursor_is_on_def_line() {
        let input = ReaderInput {
            language: "python".to_string(),
            source: "def calculate_total(price, tax_rate):\n    return price * tax_rate".to_string(),
            cursor_line: 0,
            request: ReadRequest::FunctionSummary,
        };

        let output = read_code(input);

        assert_eq!(
            output.speech,
            "Function calculate total. Parameters: price; tax rate."
        );
    }

    #[test]
    fn summarizes_python_function_when_cursor_is_inside_body() {
        let input = ReaderInput {
            language: "python".to_string(),
            source: "def calculate_total(price, tax_rate):\n    return price * tax_rate".to_string(),
            cursor_line: 1,
            request: ReadRequest::FunctionSummary,
        };

        let output = read_code(input);

        assert_eq!(
            output.speech,
            "Function calculate total. Parameters: price; tax rate."
        );
    }

    #[test]
    fn summarizes_typed_python_function_when_cursor_is_inside_body() {
        let input = ReaderInput {
            language: "python".to_string(),
            source: "def calculate_total(price: float, tax_rate: float = 0.19) -> float:\n    return price * tax_rate".to_string(),
            cursor_line: 1,
            request: ReadRequest::FunctionSummary,
        };

        let output = read_code(input);

        assert_eq!(
            output.speech,
            "Function calculate total. Parameters: price, float; tax rate, float, default zero point one nine. Returns float."
        );
    }

    #[test]
    fn returns_message_when_no_python_function_found_at_cursor() {
        let input = ReaderInput {
            language: "python".to_string(),
            source: "import math\n\nx = 10\n".to_string(),
            cursor_line: 0,
            request: ReadRequest::FunctionSummary,
        };

        let output = read_code(input);

        assert_eq!(output.speech, "No Python function was found at the cursor.");
    }

    #[test]
    fn returns_message_when_function_summaries_not_supported_for_language() {
        let input = ReaderInput {
            language: "rust".to_string(),
            source: "fn main() {}".to_string(),
            cursor_line: 0,
            request: ReadRequest::FunctionSummary,
        };

        let output = read_code(input);

        assert_eq!(
            output.speech,
            "Function summaries are not supported for rust yet."
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
    fn deserializes_json_input_for_current_line() {
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
    fn deserialize_json_input_for_function_summary() {
        let json = r#"
        {
            "language": "python",
            "source": "def calculate_total(price, tax_rate):",
            "cursor_line": 0,
            "request": "function_summary"
        }
        "#;

        let input: ReaderInput = serde_json::from_str(json).unwrap();

        assert_eq!(input.language, "python");
        assert_eq!(input.cursor_line, 0);
        assert_eq!(input.request, ReadRequest::FunctionSummary);
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