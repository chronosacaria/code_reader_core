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
        ReadRequest::CurrentScope => read_current_scope(&input),
        ReadRequest::FunctionParameters => read_function_parameters(&input),
        ReadRequest::FunctionSummary => read_function_summary(&input),
        ReadRequest::CurrentContext => read_current_context(&input),
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

/// Reads only the parameters of the function containing the cursor.
///
/// For now, this only works for Python, but will be expanded in the future.
fn read_function_parameters(input: &ReaderInput) -> ReaderOutput {
    if input.language.eq_ignore_ascii_case("python") {
        if let Some(speech) =
            languages::python::describe_function_parameter_list(&input.source, input.cursor_line)
        {
            return ReaderOutput { speech };
        }

        return ReaderOutput {
            speech: "No Python function was found at the cursor.".to_string(),
        };
    }

    ReaderOutput {
        speech: format!(
            "Function parameters are not supported for {} yet.",
            input.language
        ),
    }
}

/// Describes the current code context at the cursor.
///
/// For now, this only works for Python, but will be expanded in the future.
fn read_current_context(input: &ReaderInput) -> ReaderOutput {
    if input.language.eq_ignore_ascii_case("python") {
        if let Some(speech) =
            languages::python::describe_current_context(&input.source, input.cursor_line) {
                return ReaderOutput { speech };
            }

            return ReaderOutput {
                speech: ("No Python context found at cursor.".to_string()),
            };
    }

    ReaderOutput {
        speech: format!(
            "Current context is not available for {} yet.",
            input.language
        ),
    }
}

/// Describes the current code scope at the cursor.
///
/// For now, this only works for Python, but will be expanded in the future.
fn read_current_scope(input: &ReaderInput) -> ReaderOutput {
    if input.language.eq_ignore_ascii_case("python") {
        if let Some(speech) =
            languages::python::describe_current_scope(&input.source, input.cursor_line)
        {
            return ReaderOutput { speech };
        }

        return ReaderOutput {
            speech: ("No Python scope found at cursor.".to_string()),
        };
    }

    ReaderOutput {
        speech: format!("Current Scope is not available for {} yet.", input.language),
    }
}

/// Gets one line from a source string using a zero-based line number.
fn get_line(source: &str, cursor_line: usize) -> Option<&str> {
    source.lines().nth(cursor_line)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Description Tests

    // Scope Tests

    #[test]
    fn describes_current_scope_at_top_level() {
        let input = ReaderInput {
            language: "python".to_string(),
            source: "import math\n\nx = 10\n".to_string(),
            cursor_line: 0,
            request: ReadRequest::CurrentScope,
        };

        let output = read_code(input);

        assert_eq!(output.speech, "Current Scope: top level.");
    }

    #[test]
    fn describes_current_scope_inside_function() {
        let input = ReaderInput {
            language: "python".to_string(),
            source: "def calculate_total(price):\n    return price".to_string(),
            cursor_line: 1,
            request: ReadRequest::CurrentScope,
        };

        let output = read_code(input);

        assert_eq!(output.speech, "Current Scope: function calculate total.");
    }

    #[test]
    fn describes_current_scope_inside_if_statement() {
        let input = ReaderInput {
            language: "python".to_string(),
            source: "\
                def calculate_total(price):
                    if price > 0:
                        return price
                "
            .to_string(),
            cursor_line: 2,
            request: ReadRequest::CurrentScope,
        };

        let output = read_code(input);

        assert_eq!(
            output.speech,
            "Current Scope: if statement, inside function calculate total."
        );
    }

    #[test]
    fn describes_current_scope_inside_for_loop() {
        let input = ReaderInput {
            language: "python".to_string(),
            source: "\
                def process_items(items):
                    for item in items:
                        print(item)
                "
            .to_string(),
            cursor_line: 2,
            request: ReadRequest::CurrentScope,
        };

        let output = read_code(input);

        assert_eq!(
            output.speech,
            "Current Scope: for loop, inside function process items."
        );
    }

    #[test]
    fn describes_current_scope_inside_method_for_loop() {
        let input = ReaderInput {
            language: "python".to_string(),
            source: "\
                class Cart:
                    def print_items(self, items):
                        for item in items:
                            print(item)
                "
            .to_string(),
            cursor_line: 3,
            request: ReadRequest::CurrentScope,
        };

        let output = read_code(input);

        assert_eq!(
            output.speech,
            "Current Scope: for loop, inside class Cart, function print items."
        );
    }

    // Context Tests

    #[test]
    fn describes_current_context_at_top_level() {
        let input = ReaderInput {
            language: "python".to_string(),
            source: "import math\n\nx = 10\n".to_string(),
            cursor_line: 0,
            request: ReadRequest::CurrentContext,
        };

        let output = read_code(input);

        assert_eq!(output.speech, "Top level.");
    }

    #[test]
    fn describes_current_context_inside_function() {
        let input = ReaderInput {
            language: "python".to_string(),
            source: "def calculate_total(price, tax_rate):\n    return price * tax_rate".to_string(),
            cursor_line: 1,
            request: ReadRequest::CurrentContext,
        };

        let output = read_code(input);

        assert_eq!(output.speech, "Inside function calculate total.");
    }

    #[test]
    fn describes_current_context_inside_class() {
        let input = ReaderInput {
            language: "python".to_string(),
            source: "\
                class Cart:
                    tax_rate = 0.19

                    def calculate_total(self):
                        return 0
                "
            .to_string(),
            cursor_line: 1,
            request: ReadRequest::CurrentContext,
        };

        let output = read_code(input);

        assert_eq!(output.speech, "Inside class Cart.");
    }

    #[test]
    fn describes_current_context_inside_method() {
        let input = ReaderInput {
            language: "python".to_string(),
            source: "\
                class Cart:
                    def calculate_total(self, price):
                        return price
                "
            .to_string(),
            cursor_line: 2,
            request: ReadRequest::CurrentContext,
        };

        let output = read_code(input);

        assert_eq!(
            output.speech,
            "Inside class Cart, function calculate total."
        );
    }

    // Reading Tests

    // Function Tests

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

    // Parameter Tests

    #[test]
    fn reads_python_function_parameters_when_cursor_is_on_def_line() {
        let input = ReaderInput {
            language: "python".to_string(),
            source: "def calculate_total(price, tax_rate):\n    return price * tax_rate".to_string(),
            cursor_line: 0,
            request: ReadRequest::FunctionParameters,
        };

        let output = read_code(input);

        assert_eq!(output.speech, "Parameters: price; tax rate.");
    }

    #[test]
    fn reads_python_function_parameters_when_cursor_is_inside_body() {
        let input = ReaderInput {
            language: "python".to_string(),
            source: "def calculate_total(price, tax_rate):\n    return price * tax_rate".to_string(),
            cursor_line: 1,
            request: ReadRequest::FunctionParameters,
        };

        let output = read_code(input);

        assert_eq!(output.speech, "Parameters: price; tax rate.");
    }

    #[test]
    fn reads_typed_python_function_parameters_when_cursor_is_inside_body() {
        let input = ReaderInput {
            language: "python".to_string(),
            source: "def calculate_total(price: float, tax_rate: float = 0.19) -> float:\n    return price * tax_rate".to_string(),
            cursor_line: 1,
            request: ReadRequest::FunctionParameters,
        };

        let output = read_code(input);

        assert_eq!(
            output.speech,
            "Parameters: price, float; tax rate, float, default zero point one nine."
        );
    }

    #[test]
    fn reads_no_parameters_for_function_without_parameters() {
        let input = ReaderInput {
            language: "python".to_string(),
            source: "def main():\n    return 0".to_string(),
            cursor_line: 1,
            request: ReadRequest::FunctionParameters,
        };

        let output = read_code(input);

        assert_eq!(output.speech, "No parameters.");
    }

    // Line Tests

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

    // Fallback Tests

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

    // Summarising Tests

    // Summarising Function Tests

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

    // Null or Empty Return Tests

    // Null or Empty Function Return Tests

    #[test]
    fn returns_message_when_no_python_function_found_at_cursor() {
        let input = ReaderInput {
            language: "python".to_string(),
            source: "import math\n\nx = 10\n".to_string(),
            cursor_line: 0,
            request: ReadRequest::FunctionSummary,
        };

        let output = read_code(input);

        assert_eq!(
            output.speech,
            "No Python function was found at the cursor."
        );
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

    // Null or Empty Context Return Tests

    #[test]
    fn returns_message_when_current_scope_not_supported_for_language() {
        let input = ReaderInput {
            language: "rust".to_string(),
            source: "fn main() {}".to_string(),
            cursor_line: 0,
            request: ReadRequest::CurrentScope,
        };

        let output = read_code(input);

        assert_eq!(
            output.speech,
            "Current Scope is not available for rust yet."
        );
    }

    #[test]
    fn returns_message_when_current_context_not_supported_for_language() {
        let input = ReaderInput {
            language: "rust".to_string(),
            source: "fn main() {}".to_string(),
            cursor_line: 0,
            request: ReadRequest::CurrentContext,
        };

        let output = read_code(input);

        assert_eq!(
            output.speech,
            "Current context is not available for rust yet."
        );
    }

    // Null or Empty Parameter Return Tests

    #[test]
    fn returns_message_when_no_python_function_found_for_parameters() {
        let input = ReaderInput {
            language: "python".to_string(),
            source: "import math\n\nx = 10\n".to_string(),
            cursor_line: 0,
            request: ReadRequest::FunctionParameters,
        };

        let output = read_code(input);

        assert_eq!(output.speech, "No Python function was found at the cursor.");
    }

    #[test]
    fn returns_message_when_function_parameters_not_supported_for_language() {
        let input = ReaderInput {
            language: "rust".to_string(),
            source: "fn main() {}".to_string(),
            cursor_line: 0,
            request: ReadRequest::FunctionParameters,
        };

        let output = read_code(input);

        assert_eq!(
            output.speech,
            "Function parameters are not supported for rust yet."
        );
    }

    // Deserialisation Tests

    #[test]
    fn deserializes_json_input_for_current_scope() {
        let json = r#"
        {
            "language": "python",
            "source": "def calculate_total(price):\n    return price",
            "cursor_line": 1,
            "request": "current_scope"
        }
        "#;

        let input: ReaderInput = serde_json::from_str(json).unwrap();

        assert_eq!(input.language, "python");
        assert_eq!(input.cursor_line, 1);
        assert_eq!(input.request, ReadRequest::CurrentScope);
    }

    #[test]
    fn deserializes_json_input_for_current_context() {
        let json = r#"
        {
            "language": "python",
            "source": "def calculate_total(price, tax_rate):",
            "cursor_line": 0,
            "request": "current_context"
        }
        "#;

        let input: ReaderInput = serde_json::from_str(json).unwrap();

        assert_eq!(input.language, "python");
        assert_eq!(input.cursor_line, 0);
        assert_eq!(input.request, ReadRequest::CurrentContext);
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
    fn deserializes_json_input_for_function_parameters() {
        let json = r#"
        {
            "language": "python",
            "source": "def calculate_total(price, tax_rate):",
            "cursor_line": 0,
            "request": "function_parameters"
        }
        "#;

        let input: ReaderInput = serde_json::from_str(json).unwrap();

        assert_eq!(input.language, "python");
        assert_eq!(input.cursor_line, 0);
        assert_eq!(input.request, ReadRequest::FunctionParameters);
    }

    // Serialisation Tests

    #[test]
    fn serializes_json_output() {
        let output = ReaderOutput {
            speech: "Current line: def example".to_string(),
        };

        let json = serde_json::to_string(&output).unwrap();

        assert_eq!(json, r#"{"speech":"Current line: def example"}"#);
    }
}