use tree_sitter::{Node, Parser, Tree};

use crate::speech::make_simple_speech_text;

/// Parses Python source code using Tree-sitter.
/// 
/// This creates a parser, assigns the Python grammar to it, and parses
/// the source text into a syntax tree.
pub fn parse_python(source: &str) -> Option<Tree> {
    let mut parser = Parser::new();

    let language = tree_sitter_python::LANGUAGE;

    parser
        .set_language(&language.into())
        .expect("Error loading Python grammar");

    parser.parse(source, None)
}

/// Recursively searches the syntax tree for a Python function definition
/// that starts on the requested cursor line.
///
/// This is useful for `current_line`, because `current_line` should only
/// describe a function structurally when the cursor is physically on the
/// `def` line.
pub fn find_function_definition_starting_on_line<'tree>(
    node: Node<'tree>,
    cursor_line: usize,
) -> Option<Node<'tree>> {
    if node.kind() == "function_definition" && node.start_position().row == cursor_line {
        return Some(node);
    }

    let mut cursor = node.walk();

    for child in node.named_children(&mut cursor) {
        if let Some(found) = find_function_definition_starting_on_line(child, cursor_line) {
            return Some(found);
        }
    }

    None
}

/// Recursively searches the syntax tree for the most specific Python function
/// definition that contains the requested cursor line.
///
/// This is useful for `function_summary`, because the cursor may be inside
/// the function body rather than on the `def` line.
pub fn find_function_definition_containing_line<'tree>(
    node: Node<'tree>,
    cursor_line: usize,
) -> Option<Node<'tree>> {
    let mut cursor = node.walk();

    for child in node.named_children(&mut cursor) {
        if node_contains_line(child, cursor_line) {
            if let Some(found) = find_function_definition_containing_line(child, cursor_line) {
                return Some(found);
            }
        }
    }

    if node.kind() == "function_definition" && node_contains_line(node, cursor_line) {
        return Some(node);
    }

    None
}

/// Returns true when a syntax node covers the requested line.
fn node_contains_line(node: Node, line: usize) -> bool {
    let start_row = node.start_position().row;
    let end_row = node.end_position().row;

    start_row <= line && line <= end_row
}

/// Converts a Python function_definition node into speech text.
pub fn describe_function_definition(function_node: Node, source: &str) -> String {
    let source_bytes = source.as_bytes();

    let function_name = function_node
        .child_by_field_name("name")
        .and_then(|name_node| name_node.utf8_text(source_bytes).ok())
        .map(make_simple_speech_text)
        .unwrap_or_else(||"unknown function".to_string());

    let parameters = function_node
        .child_by_field_name("parameters")
        .and_then(|parameter_node| parameter_node.utf8_text(source_bytes).ok())
        .map(extract_parameter_names)
        .unwrap_or_default();

    if parameters.is_empty() {
        format!("Function {function_name}. No parameters.")
    } else {
        format!(
            "Function {function_name}. Parameters: {}.",
            parameters.join("; ")
        )
    }
}

/// Extracts simple parameter names from a Python parameter list.
///
/// This first version intentionally handles common simple cases only.
///
/// Examples:
///
/// - `(price, tax_rate)` becomes `price`, `tax rate`
/// - `(price: float, tax_rate: float = 0.19)` becomes `price`, `tax rate`
/// - `()` becomes an empty list
fn extract_parameter_names(parameters_text: &str) -> Vec<String> {
    let without_parentheses = parameters_text
        .trim()
        .trim_start_matches('(')
        .trim_end_matches(')')
        .trim();

    if without_parentheses.is_empty() {
        return Vec::new();
    }

    without_parentheses
        .split(',')
        .filter_map(extract_single_parameter_name)
        .collect()
}

/// Extracts one parameter name from one parameter fragment.
///
/// Examples:
///
/// - `price: float` becomes `price`
/// - `tax_rate: float = 0.19` becomes `tax_rate`
/// - `*args` becomes `args`
/// - `**kwargs` becomes `kwargs`
fn extract_single_parameter_name(parameter_text: &str) -> Option<String> {
    let before_default = parameter_text.split('=').next()?.trim();
    let before_type = before_default.split(':').next()?.trim();

    let name = before_type.trim_start_matches('*').trim();

    if name.is_empty() {
        None
    } else {
        Some(make_simple_speech_text(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_function_that_starts_on_cursor_line() {
        let source = "def calculate_total(price, tax_rate):\n   return price * tax_rate";
        let tree = parse_python(source).unwrap();
        let root = tree.root_node();

        let function_node = find_function_definition_starting_on_line(root, 0);

        assert!(function_node.is_some());
    }

    #[test]
    fn does_not_find_starting_function_when_cursor_is_in_body() {
        let source = "def calculate_total(price, tax_rate):\n   return price * tax_rate";
        let tree = parse_python(source).unwrap();
        let root = tree.root_node();

        let function_node = find_function_definition_starting_on_line(root, 1);

        assert!(function_node.is_none());
    }

    #[test]
    fn finds_function_containing_cursor_line() {
        let source = "def calculate_total(price, tax_rate):\n    return price * tax_rate";
        let tree = parse_python(source).unwrap();
        let root = tree.root_node();

        let function_node = find_function_definition_starting_on_line(root, 0);

        assert!(function_node.is_some());
    }
    
    #[test]
    fn describes_function_definition_with_parameters() {
        let source = "def calculate_total(price, tax_rate):\n    return price * tax_rate";
        let tree = parse_python(source).unwrap();
        let root = tree.root_node();

        let function_node = 
            find_function_definition_starting_on_line(root, 0)
                .expect("Expected function node");

        let speech = describe_function_definition(function_node, source);

        assert_eq!(
            speech,
            "Function calculate total. Parameters: price; tax rate."
        );
    }

    #[test]
    fn describes_function_definition_with_no_parameters() {
        let source = "def main():\n    return 0";
        let tree = parse_python(source).unwrap();
        let root = tree.root_node();

        let function_node = 
            find_function_definition_starting_on_line(root, 0)
                .expect("Expected function node");

        let speech = describe_function_definition(function_node, source);

        assert_eq!(
            speech,
            "Function main. No parameters."
        );
    }

    #[test]
    fn describes_typed_parameters_without_reading_types_yet() {
        let source = "def calculate_total(price: float, tax_rate: float = 0.19):\n    return price * tax_rate";
        let tree = parse_python(source).unwrap();
        let root = tree.root_node();

        let function_node = 
            find_function_definition_starting_on_line(root, 0)
                .expect("Expected function node");
        
        let speech = describe_function_definition(function_node, source);

        assert_eq!(
            speech,
            "Function calculate total. Parameters: price; tax rate."
        );
    }
}