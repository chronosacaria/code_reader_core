// Questions Answered:
// What are this function's parameters?
// What inputs does this function take?

use super::syntax::{
    describe_function_parameters,
    find_function_definition_containing_line,
    parse_python,
};


/// Describes only the parameters of the Python function containing the cursor.
///
/// Unlike `describe_function_summary`, this intentionally does not include:
///
/// - the function name
/// - the return type
/// - the current class/function context
pub fn describe_function_parameter_list(source: &str, cursor_line: usize) -> Option<String> {
    let tree = parse_python(source)?;
    let root = tree.root_node();

    let function_node = find_function_definition_containing_line(root, cursor_line)?;

    Some(describe_function_parameters(function_node, source))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Describing Tests

    #[test]
    fn describes_parameters_when_cursor_is_on_def_line() {
        let source = "def calculate_total(price, tax_rate):\n    return price * tax_rate";

        let speech = describe_function_parameter_list(source, 0);

        assert_eq!(
            speech,
            Some("Parameters: price; tax rate.".to_string())
        );
    }

    #[test]
    fn describes_parameters_when_cursor_is_inside_body() {
        let source = "def calculate_total(price, tax_rate):\n    return price * tax_rate";

        let speech = describe_function_parameter_list(source, 1);

        assert_eq!(
            speech,
            Some("Parameters: price; tax rate.".to_string())
        );
    }

    #[test]
    fn describes_typed_parameters_when_cursor_is_inside_body() {
        let source = "def calculate_total(price: float, tax_rate: float = 0.19) -> float:\n    return price * tax_rate";

        let speech = describe_function_parameter_list(source, 1);

        assert_eq!(
            speech,
            Some(
                "Parameters: price, float; tax rate, float, default zero point one nine."
                    .to_string()
            )
        );
    }

    #[test]
    fn describes_no_parameters() {
        let source = "def main():\n    return 0";

        let speech = describe_function_parameter_list(source, 1);

        assert_eq!(speech, Some("No parameters.".to_string()));
    }

    // Return Tests

    #[test]
    fn returns_none_when_cursor_is_outside_any_function() {
        let source = "import math\n\nx = 10\n";

        let speech = describe_function_parameter_list(source, 0);

        assert_eq!(speech, None);
    }

    #[test]
    fn prefers_innermost_function_parameters() {
        let source = "\
            def outer():
                def inner(value: int):
                    return value
                return inner(1)
            ";

        let speech = describe_function_parameter_list(source, 2);

        assert_eq!(
            speech,
            Some("Parameters: value, int.".to_string())
        );
    }
}