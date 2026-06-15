// Questions Answered:
// What local block currently controls this line?
// What scope am I in right now?

use tree_sitter::Node;

use super::syntax::{
    describe_local_scope_node, find_class_definition_containing_line,
    find_function_definition_containing_line, find_local_scope_node_containing_line,
    get_class_name, get_function_name, parse_python,
};

/// Describes the current Python scope at the cursor.
///
/// Current scope answers:
/// "What local block currently controls this line?"
///
/// Examples:
///
/// - top level:
///   `Current scope: top level.`
///
/// - inside a function body:
///   `Current scope: function calculate total.`
///
/// - inside an if statement:
///   `Current scope: if statement, inside function calculate total.`
///
/// - inside a method's for loop:
///   `Current scope: for loop, inside class Cart, function calculate total.`
pub fn describe_current_scope(source: &str, cursor_line: usize) -> Option<String> {
    let tree = parse_python(source)?;
    let root = tree.root_node();

    let local_scope_node = find_local_scope_node_containing_line(root, cursor_line);
    let class_node = find_class_definition_containing_line(root, cursor_line);
    let function_node = find_function_definition_containing_line(root, cursor_line);

    match local_scope_node {
        Some(scope_node) => {
            let scope_description = describe_local_scope_node(scope_node);
            let context_description = describe_context_tail(class_node, function_node, source);

            match context_description {
                Some(context_description) => Some(format!(
                    "Current scope: {scope_description}, {context_description}."
                )),
                None => Some(format!("Current scope: {scope_description}.")),
            }
        }
        None => describe_non_local_scope(class_node, function_node, source),
    }
}

/// Describes the current scope when there is no local control-flow block.
fn describe_non_local_scope(
    class_node: Option<Node>,
    function_node: Option<Node>,
    source: &str,
) -> Option<String> {
    match (class_node, function_node) {
        (Some(class_node), Some(function_node)) => {
            let class_name = get_class_name(class_node, source);
            let function_name = get_function_name(function_node, source);

            Some(format!(
                "Current scope: function {function_name}, inside class {class_name}."
            ))
        }
        (Some(class_node), None) => {
            let class_name = get_class_name(class_node, source);

            Some(format!("Current scope: class {class_name}."))
        }
        (None, Some(function_node)) => {
            let function_name = get_function_name(function_node, source);

            Some(format!("Current scope: function {function_name}."))
        }
        (None, None) => Some("Current scope: top level.".to_string()),
    }
}

/// Describes the broader context around a local scope.
///
/// This is used when the cursor is inside something like an if statement or
/// for loop, and we also want to say which function/class contains that block.
fn describe_context_tail(
    class_node: Option<Node>,
    function_node: Option<Node>,
    source: &str,
) -> Option<String> {
    match (class_node, function_node) {
        (Some(class_node), Some(function_node)) => {
            let class_name = get_class_name(class_node, source);
            let function_name = get_function_name(function_node, source);

            Some(format!(
                "inside class {class_name}, function {function_name}"
            ))
        }
        (Some(class_node), None) => {
            let class_name = get_class_name(class_node, source);

            Some(format!("inside class {class_name}"))
        }
        (None, Some(function_node)) => {
            let function_name = get_function_name(function_node, source);

            Some(format!("inside function {function_name}"))
        }
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Describing Tests

    #[test]
    fn describes_top_level_scope() {
        let source = "import math\n\nx = 10\n";

        let speech = describe_current_scope(source, 0);

        assert_eq!(speech, Some("Current scope: top level.".to_string()));
    }

    #[test]
    fn describes_function_scope_when_not_inside_local_block() {
        let source = "\
            def calculate_total(price):
                return price
            ";

        let speech = describe_current_scope(source, 1);

        assert_eq!(
            speech,
            Some("Current scope: function calculate total.".to_string())
        );
    }

    #[test]
    fn describes_class_scope_when_inside_class_but_outside_method() {
        let source = "\
            class Cart:
                tax_rate = 0.19

                def calculate_total(self):
                    return 0
            ";

        let speech = describe_current_scope(source, 1);

        assert_eq!(speech, Some("Current scope: class Cart.".to_string()));
    }

    #[test]
    fn describes_method_scope_when_not_inside_local_block() {
        let source = "\
            class Cart:
                def calculate_total(self, price):
                    return price
            ";

        let speech = describe_current_scope(source, 2);

        assert_eq!(
            speech,
            Some("Current scope: function calculate total, inside class Cart.".to_string())
        );
    }

    #[test]
    fn describes_if_scope_inside_function() {
        let source = "\
            def calculate_total(price):
                if price > 0:
                    return price
            ";

        let speech = describe_current_scope(source, 2);

        assert_eq!(
            speech,
            Some("Current scope: if statement, inside function calculate total.".to_string())
        );
    }

    #[test]
    fn describes_for_loop_scope_inside_function() {
        let source = "\
            def process_items(items):
                for item in items:
                    print(item)
            ";

        let speech = describe_current_scope(source, 2);

        assert_eq!(
            speech,
            Some("Current scope: for loop, inside function process items.".to_string())
        );
    }

    #[test]
    fn describes_for_loop_scope_inside_method() {
        let source = "\
            class Cart:
                def print_items(self, items):
                    for item in items:
                        print(item)
            ";

        let speech = describe_current_scope(source, 3);

        assert_eq!(
            speech,
            Some("Current scope: for loop, inside class Cart, function print items.".to_string())
        );
    }

    #[test]
    fn prefers_inner_local_scope() {
        let source = "\
            def process_items(items):
                for item in items:
                    if item:
                        print(item)
            ";

        let speech = describe_current_scope(source, 3);

        assert_eq!(
            speech,
            Some("Current scope: if statement, inside function process items.".to_string())
        );
    }

    #[test]
    fn describes_while_loop_scope() {
        let source = "\
            def countdown(value):
                while value > 0:
                    value -= 1
            ";

        let speech = describe_current_scope(source, 2);

        assert_eq!(
            speech,
            Some("Current scope: while loop, inside function countdown.".to_string())
        );
    }
}
