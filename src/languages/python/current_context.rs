// Questions Answered:
// Where am I, generally?
// Where am I in the file's larger structure?

use super::syntax::{
    find_class_definition_containing_line,
    find_function_definition_containing_line,
    get_class_name,
    get_function_name,
    parse_python,
};

pub fn describe_current_context(source: &str, cursor_line: usize) -> Option<String> {
    let tree = parse_python(source)?;
    let root = tree.root_node();

    let class_node = find_class_definition_containing_line(root, cursor_line);
    let function_node = find_function_definition_containing_line(root, cursor_line);

    match (class_node, function_node) {
        (Some(class_node), Some(function_node)) => {
            let class_name = get_class_name(class_node, source);
            let function_name = get_function_name(function_node, source);

            Some(format!(
                "Inside class {class_name}, function {function_name}."
            ))
        }
        (Some(class_node), None) => {
            let class_name = get_class_name(class_node, source);

            Some(format!("Inside class {class_name}."))
        }
        (None, Some(function_node)) => {
            let function_name = get_function_name(function_node, source);

            Some(format!("Inside function {function_name}."))
        }
        (None, None) => Some("Top level.".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describes_top_level_context() {
        let source = "import math\n\nx = 10\n";

        let speech = describe_current_context(source, 0);

        assert_eq!(speech, Some("Top level.".to_string()));
    }

    #[test]
    fn describes_function_context() {
        let source = "\
            def calculate_total(price, tax_rate):
                return price * tax_rate
            ";

        let speech = describe_current_context(source, 1);

        assert_eq!(
            speech,
            Some("Inside function calculate total.".to_string())
        );
    }

    #[test]
    fn describes_class_context_when_inside_class_but_outside_method() {
        let source = "\
            class Cart:
                tax_rate = 0.19

                def calculate_total(self):
                    return 0
            ";

        let speech = describe_current_context(source, 1);

        assert_eq!(speech, Some("Inside class Cart.".to_string()));
    }

    #[test]
    fn describes_class_and_function_context_when_inside_method() {
        let source = "\
            class Cart:
                def calculate_total(self, price):
                    return price
            ";

        let speech = describe_current_context(source, 2);

        assert_eq!(
            speech,
            Some("Inside class Cart, function calculate total.".to_string())
        );
    }

    #[test]
    fn prefers_inner_function_for_nested_function_context() {
        let source = "\
            def outer():
                def inner(value):
                    return value
                return inner(1)
            ";

        let speech = describe_current_context(source, 2);

        assert_eq!(speech, Some("Inside function inner.".to_string()));
    }

    #[test]
    fn describes_nested_function_inside_method() {
        let source = "\
            class Cart:
                def calculate_total(self, price):
                    def apply_tax(value):
                        return value * 1.19
                    return apply_tax(price)
            ";

        let speech = describe_current_context(source, 3);

        assert_eq!(
            speech,
            Some("Inside class Cart, function apply tax.".to_string())
        );
    }
}