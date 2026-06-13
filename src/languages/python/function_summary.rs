// Questions Answered:
// What is the function that my cursor is in?
// What function am I in, including its signature details?

use super::syntax::{
    describe_function_definition,
    find_function_definition_containing_line,
    parse_python,
};

/// Summarises the Python function containing the cursor.
/// 
/// Unlike `describe_current_line`, this does not require the cursor to be on the
/// `def` line. The cursor can be anywhere within the function body.
pub fn describe_function_summary(source: &str, cursor_line: usize) -> Option<String> {
    let tree = parse_python(source)?;
    let root = tree.root_node();

    let function_node = find_function_definition_containing_line(root, cursor_line)?;

    Some(describe_function_definition(function_node, source))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarizes_function_when_cursor_is_on_def_line() {
        let source = "def calculate_total(price, tax_rate):\n    return price * tax_rate";

        let speech = describe_function_summary(source, 0);

        assert_eq!(
            speech,
            Some("Function calculate total. Parameters: price; tax rate.".to_string())
        );
    }

    #[test]
    fn summarizes_function_when_cursor_is_inside_body() {
        let source = "def calculate_total(price, tax_rate):\n    return price * tax_rate";

        let speech = describe_function_summary(source, 1);

        assert_eq!(
            speech,
            Some("Function calculate total. Parameters: price; tax rate.".to_string())
        );
    }

    #[test]
    fn summarizes_typed_function_when_cursor_is_inside_body() {
        let source = "def calculate_total(price: float, tax_rate: float = 0.19) -> float:\n    return price * tax_rate";

        let speech = describe_function_summary(source, 1);

        assert_eq!(
            speech,
            Some(
                "Function calculate total. Parameters: price, float; tax rate, float, default zero point one nine. Returns float."
                    .to_string()
            )
        );
    }

    #[test]
    fn returns_none_when_cursor_is_outside_any_function() {
        let source = "import math\n\nx = 10\n";

        let speech = describe_function_summary(source, 2);

        assert_eq!(speech, None);
    }

    #[test]
    fn prefers_innermost_function() {
        let source = "\
def outer():
    def inner(value):
        return value
    return inner(1)
";

        let speech = describe_function_summary(source, 2);

        assert_eq!(
            speech,
            Some("Function inner. Parameters: value.".to_string())
        );
    }
}