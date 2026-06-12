// Questions Answered:
// Can the current physical Python line be described structurally?

use super::syntax::{
    describe_function_definition,
    find_function_definition_starting_on_line,
    parse_python,
};

/// Tries to describe the current Python line structurally.
///
/// For this first Tree-sitter step, we only detect function definitions
/// that start on the cursor line.
///
/// If the current line is not a function definition, this returns `None`.
/// The caller can then fall back to the simple speech behavior.
pub fn describe_current_line(source: &str, cursor_line: usize) -> Option<String> {
    let tree = parse_python(source)?;
    let root = tree.root_node();

    let function_node = find_function_definition_starting_on_line(root, cursor_line)?;

    Some(describe_function_definition(function_node, source))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describes_function_definition_when_cursor_is_on_def_line() {
        let source = "def calculate_total(price, tax_rate):\n    return price * tax_rate";

        let speech = describe_current_line(source, 0);

        assert_eq!(
            speech,
            Some("Function calculate total. Parameters: price; tax rate.".to_string())
        );
    }

    #[test]
    fn returns_none_when_cursor_is_inside_function_body() {
        let source = "def calculate_total(price, tax_rate):\n    return price * tax_rate";

        let speech = describe_current_line(source, 1);

        assert_eq!(speech, None);
    }
}