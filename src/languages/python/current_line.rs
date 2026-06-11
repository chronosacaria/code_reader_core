use tree_sitter::{Node, Parser};

use crate::speech::make_simple_speech_text;

/// Tries to describe the current Python line structurally.
///
/// For this first Tree-sitter step, we only detect function definitions
/// that start on the cursor line.
///
/// If the current line is not a function definition, this returns `None`.
/// The caller can then fall back to the simple speech behavior.
pub fn describe_current_line(source: &str, cursor_line: usize) -> Option<String> {
    let tree = match parse_python(source) {
        Some(tree) => tree,
        None => return None,
    };
    let root = tree.root_node();

    let function_node = find_function_definition_starting_on_line(root, cursor_line)?;

    Some(describe_function_definition(function_node, source))
}

/// Parses Python source code using Tree-sitter.
///
/// This creates a parser, assigns the Python grammar to it, and parses
/// the source text into a syntax tree.
fn parse_python(source: &str) -> Option<tree_sitter::Tree> {
    let mut parser = Parser::new();

    let language = tree_sitter_python::LANGUAGE;
    
    parser
        .set_language(&language.into())
        .expect("Error loading Python grammar");

    parser.parse(source, None)
}

/// Recursively searches the syntax tree for a Python function definition
/// that starts on the requested cursor line.
fn find_function_definition_starting_on_line<'tree>(
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

/// Converts a Python function_definition node into speech text.
fn describe_function_definition(function_node: Node, source: &str) -> String {
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
    fn describes_function_definition_with_parameters() {
        let source = "def calculate_total(price, tax_rate):\n    return price * tax_rate";

        let speech = describe_current_line(source, 0);

        assert_eq!(
            speech,
            Some("Function calculate total. Parameters: price; tax rate.".to_string())
        );
    }

    #[test]
    fn describes_function_definition_with_no_parameters() {
        let source = "def main():\n    return 0";

        let speech = describe_current_line(source, 0);

        assert_eq!(
            speech,
            Some("Function main. No parameters.".to_string())
        );
    }

    #[test]
    fn describes_typed_parameters_without_reading_types_yet() {
        let source = "def calculate_total(price: float, tax_rate: float = 0.19):\n    return price * tax_rate";

        let speech = describe_current_line(source, 0);

        assert_eq!(
            speech,
            Some("Function calculate total. Parameters: price; tax rate.".to_string())
        );
    }

    #[test]
    fn return_none_when_line_is_not_function_definition() {
        let source = "def calculate_total(price, tax_rate):\n    return price * tax_rate";

        let speech = describe_current_line(source, 1);

        assert_eq!(speech, None);
    }
}