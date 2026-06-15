use tree_sitter::{Node, Parser, Tree};

use crate::speech::make_simple_speech_text;

/// A simplified description of one Python function parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ParameterSummary {
    name: String,
    type_annotation: Option<String>,
    default_value: Option<String>,
}

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

/// Recursively searches the syntax tree for the most specific Python class
/// definition that contains the requested cursor line.
///
/// This is useful for `current_context`, because the cursor may be inside
/// a class body or inside a method belonging to a class.
pub fn find_class_definition_containing_line<'tree>(
    node: Node<'tree>,
    cursor_line: usize,
) -> Option<Node<'tree>> {
    let mut cursor = node.walk();

    for child in node.named_children(&mut cursor) {
        if node_contains_line(child, cursor_line) {
            if let Some(found) = find_class_definition_containing_line(child, cursor_line) {
                return Some(found);
            }
        }
    }

    if node.kind() == "class_definition" && node_contains_line(node, cursor_line) {
        return Some(node);
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

/// Recursively searches the syntax tree for the most specific local Python
/// scope node that contains the requested cursor line.
///
/// This is used by `current_scope`.
/// It does not return function or class nodes.
pub fn find_local_scope_node_containing_line<'tree>(
    node: Node<'tree>,
    cursor_line: usize,
) -> Option<Node<'tree>> {
    if !node_contains_line(node, cursor_line) {
        return None;
    }

    let mut best_match = if is_local_scope_node(node) {
        Some(node)
    } else {
        None
    };

    let mut cursor = node.walk();

    for child in node.named_children(&mut cursor) {
        if node_contains_line(child, cursor_line) {
            if let Some(found) = find_local_scope_node_containing_line(child, cursor_line) {
                best_match = Some(found);
            }
        }
    }

    best_match
}

/// Returns true when a syntax node represents a local Python scope/block.
///
/// This is intentionally separate from function/class detection.
/// Function and class nodes are broader context containers, while this helper
/// focuses on local block structures.
fn is_local_scope_node(node: Node) -> bool {
    matches!(
        node.kind(),
        "for_statement"
            |"if_statement"
            | "match_statement"
            | "try_statement"
            | "while_statement"
            | "with_statement"
            | "elif_clause"
            | "else_clause"
            | "except_clause"
            | "finally_clause"
    )
}

/// Returns true when a syntax node covers the requested line.
fn node_contains_line(node: Node, line: usize) -> bool {
    let start_row = node.start_position().row;
    let end_row = node.end_position().row;

    start_row <= line && line <= end_row
}

/// Extracts and speech-formats the name of a Python class definition.
pub fn get_class_name(class_node: Node, source: &str) -> String {
    class_node
        .child_by_field_name("name")
        .and_then(|name_node| name_node.utf8_text(source.as_bytes()).ok())
        .map(make_simple_speech_text)
        .unwrap_or_else(|| "unknown class".to_string())
}

/// Extracts and speech-formats the name of a Python function definition.
pub fn get_function_name(function_node: Node, source: &str) -> String {
    function_node
        .child_by_field_name("name")
        .and_then(|name_node| name_node.utf8_text(source.as_bytes()).ok())
        .map(make_simple_speech_text)
        .unwrap_or_else(|| "unknown function".to_string())
}

/// Converts a local Python scope node into short speech.
///
/// This describes the kind of local block, not the full source condition.
pub fn describe_local_scope_node(scope_node: Node) -> String {
    match scope_node.kind() {
        "for_statement" => "for loop".to_string(),
        "if_statement" => "if statement".to_string(),
        "match_statement" => "match statement".to_string(),
        "try_statement" => "try statement".to_string(),
        "while_statement" => "while loop".to_string(),
        "with_statement" => "with statement".to_string(),
        "elif_clause" => "elif clause".to_string(),
        "else_clause" => "else clause".to_string(),
        "except_clause" => "except clause".to_string(),
        "finally_clause" => "finally clause".to_string(),
        other => format!("{other} scope")
    }
}

/// Converts a Python function_definition node into speech text.
pub fn describe_function_definition(function_node: Node, source: &str) -> String {
    let source_bytes = source.as_bytes();

    let function_name = get_function_name(function_node, source);

    let parameters = function_node
        .child_by_field_name("parameters")
        .and_then(|parameters_node| parameters_node.utf8_text(source_bytes).ok())
        .map(extract_parameter_summaries)
        .unwrap_or_default();

    let return_type = extract_return_type(function_node, source);

    let parameter_speech = describe_parameters(&parameters);

    match return_type {
        Some(return_type) => format!(
            "Function {function_name}. {parameter_speech}. Returns {return_type}."
        ),
        None => format!(
            "Function {function_name}. {parameter_speech}."
        )
    }
}

/// Converts a Python function_definition node into parameter-only speech.
///
/// This is used by the `FunctionParameters` request.
///
/// It intentionally does not include:
///
/// - the function name
/// - the return type
/// - the class/function context
pub fn describe_function_parameters(function_node: Node, source: &str) -> String {
    let parameters = function_node
        .child_by_field_name("parameters")
        .and_then(
            |parameters_node| parameters_node
                .utf8_text(source.as_bytes())
                .ok())
        .map(extract_parameter_summaries)
        .unwrap_or_default();

    let parameter_speech = describe_parameters(&parameters);

    format!("{parameter_speech}.")
}

/// Converts a list of parameter summaries into one speech sentence.
///
/// Examples:
///
/// - no parameters: `No parameters`
/// - one parameter: `Parameters: price, float`
/// - two parameters: `Parameters: price, float; tax rate, float, default zero point one nine`
fn describe_parameters(parameters: &[ParameterSummary]) -> String {
    if parameters.is_empty() {
        return "No parameters".to_string();
    }

    let spoken_parameters = parameters
        .iter()
        .map(describe_parameter)
        .collect::<Vec<_>>()
        .join("; ");

    format!("Parameters: {spoken_parameters}")
}

/// Converts one parameter into speech.
///
/// Examples:
///
/// - `price`
/// - `price, float`
/// - `tax rate, float, default zero point one nine`
fn describe_parameter(parameter: &ParameterSummary) -> String {
    let mut parts = vec![parameter.name.clone()];

    if let Some(type_annotation) = &parameter.type_annotation {
        parts.push(type_annotation.clone());
    }

    if let Some(default_value) = &parameter.default_value {
        parts.push(format!("default {default_value}"));
    }

    parts.join(", ")
}

/// Extracts simple parameter summaries from a Python parameter list.
///
/// This first version intentionally handles common simple cases.
///
/// Examples:
///
/// - `(price, tax_rate)`
/// - `(price: float, tax_rate: float = 0.19)`
/// - `()`
///
/// It does not yet fully support every complex Python signature.
fn extract_parameter_summaries(parameters_text: &str) -> Vec<ParameterSummary> {
    let without_parentheses = parameters_text
        .trim()
        .trim_start_matches('(')
        .trim_end_matches(')')
        .trim();

    if without_parentheses.is_empty() {
        return Vec::new();
    }

    split_top_level_commas(without_parentheses)
        .into_iter()
        .filter_map(|parameter_text| extract_single_parameter_summary(&parameter_text))
        .collect()
}

/// Extracts one parameter summary from one parameter fragment.
///
/// Examples:
///
/// - `price`
/// - `price: float`
/// - `tax_rate: float = 0.19`
/// - `*args`
/// - `**kwargs`
fn extract_single_parameter_summary(parameter_text: &str) -> Option<ParameterSummary> {
    let trimmed = parameter_text.trim();

    if trimmed.is_empty() {
        return None;
    }

    // Python signatures can contain standalone markers:
    //
    // - `/` for positional-only separator
    // - `*` for keyword-only separator
    //
    // Remember to come back to these
    if trimmed == "/" || trimmed == "*" {
        return None;
    }

    let (before_default, default_value) = split_once_top_level(trimmed, '=');

    let (name_text, type_annotation) = split_once_top_level(before_default.trim(), ':');

    let name = name_text
        .trim()
        .trim_start_matches('*')
        .trim();

    if name.is_empty() {
        return None;
    }

    Some(ParameterSummary {
        name: make_simple_speech_text(name),
        type_annotation: type_annotation.map(|value| translate_annotation_to_speech(value.trim())),
        default_value: default_value.map(|value| translate_default_value_to_speech(value.trim())),
    })
}

/// Extracts the return type annotation from a function definition.
///
/// For now, this uses the first line of the function definition text.
/// This handles the common single-line signature form:
///
/// `def example(value: int) -> str:`
fn extract_return_type(function_node: Node, source: &str) -> Option<String> {
    let function_text = function_node.utf8_text(source.as_bytes()).ok()?;
    let signature_line = function_text.lines().next()?.trim();

    let after_arrow = signature_line.split("->").nth(1)?.trim();
    let before_colon = after_arrow.rsplit_once(':')?.0.trim();

    if before_colon.is_empty() {
        None
    } else {
        Some(translate_annotation_to_speech(before_colon))
    }
}

/// Converts a type annotation into speech.
///
/// Examples:
///
/// - `float` becomes `float`
/// - `str | None` becomes `str or none`
/// - `list[str]` becomes `list of str`
fn translate_annotation_to_speech(annotation: &str) -> String {
    annotation
        .replace('[', " of ")
        .replace(']', "")
        .replace('|', " or ")
        .replace('.', " dot ")
        .replace('_', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Converts a default value into speech.
///
/// Examples:
///
/// - `0.19` becomes `zero point one nine`
/// - `"guest"` becomes `guest`
/// - `None` becomes `none`
/// - `True` becomes `true`
fn translate_default_value_to_speech(default_value: &str) -> String {
    let trimmed = default_value.trim();

    if let Some(number_speech) = speak_simple_number(trimmed) {
        return number_speech;
    }

    let without_quites = trimmed
        .trim_matches('"')
        .trim_matches('\'')
        .trim();

    make_simple_speech_text(without_quites).to_lowercase()
}

/// Speaks simple integer and decimal literals.
///
/// This intentionally handles a small useful subset:
///
/// - `0`
/// - `12`
/// - `0.19`
/// - `-1.5`
fn speak_simple_number(value: &str) -> Option<String> {
    let mut chars = value.chars();

    let is_negative = value.starts_with('-');
    let unsigned_value = if is_negative {
        chars.next();
        chars.as_str()
    } else {
        value
    };

    if unsigned_value.is_empty() {
        return None;
    }

    if !unsigned_value
        .chars()
        .all(|character| character.is_ascii_digit() || character == '.')
    {
        return None;
    }

    if unsigned_value.matches('.').count() > 1 {
        return None;
    }

    let mut parts = Vec::new();

    if is_negative {
        parts.push("minus".to_string());
    }

    if let Some((whole, fractional)) = unsigned_value.split_once('.') {
        parts.push(speak_digits_as_number_word(whole)?);
        parts.push("point".to_string());

        for digit in fractional.chars() {
            parts.push(speak_digit(digit)?.to_string())
        }
    } else {
        parts.push(speak_digits_as_number_word(unsigned_value)?);
    }

    Some(parts.join(" "))
}

/// Speaks a short string of digits.
///
/// This currently gives special treatment to single digits and otherwise
/// returns the original number text. Otherwise, screenreader output can
/// get kind of wonky. There has to be a way to improve this.
///
/// Examples:
///
/// - `0` becomes `zero`
/// - `9` becomes `nine`
/// - `12` stays `12`
fn speak_digits_as_number_word(digits: &str) -> Option<String> {
    if digits.is_empty() {
        return Some("zero".to_string());
    }

    if digits.len() == 1 {
        return Some(speak_digit(digits.chars().next()?)?.to_string());
    }

    Some(digits.to_string())
}

/// Speaks one digit.
fn speak_digit(digit: char) -> Option<&'static str> {
    match digit {
        '0' => Some("zero"),
        '1' => Some("one"),
        '2' => Some("two"),
        '3' => Some("three"),
        '4' => Some("four"),
        '5' => Some("five"),
        '6' => Some("six"),
        '7' => Some("seven"),
        '8' => Some("eight"),
        '9' => Some("nine"),
        _ => None,
    }
}

/// Splits text by top-level commas.
///
/// This avoids splitting commas inside simple bracket, parenthesis, or brace
/// pairs.
///
/// Example:
///
/// `items: list[str], default=(1, 2)`
///
/// should split into:
///
/// - `items: list[str]`
/// - `default=(1, 2)`
fn split_top_level_commas(text: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;
    let mut quote: Option<char> = None;

    for character in text.chars() {
        if let Some(quote_character) = quote {
            current.push(character);

            if character == quote_character {
                quote = None;
            }

            continue;
        }

        match character {
            '\'' | '"' => {
                quote = Some(character);
                current.push(character);
            }
            '(' | '[' | '{' => {
                depth += 1;
                current.push(character);
            }
            ')' | ']' | '}' => {
                depth = depth.saturating_sub(1);
                current.push(character);
            }
            ',' if depth == 0 => {
                parts.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(character),
        }
    }

    if !current.trim().is_empty() {
        parts.push(current.trim().to_string())
    }

    parts
}

/// Splits once on a top-level separator character.
///
/// This is used for `name: type` and `name = default`.
///
/// It avoids splitting inside simple bracket, parenthesis, brace, or quoted
/// sections.
fn split_once_top_level(text: &str, separator: char) -> (&str, Option<&str>) {
    let mut depth = 0usize;
    let mut quote: Option<char> = None;

    for (index, character) in text.char_indices() {
        if let Some(quote_character) = quote {
            if character == quote_character {
                quote = None;
            }

            continue;
        }

        match character {
            '\'' | '"' => quote = Some(character),
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            _ => {}
        }

        if character == separator && depth == 0 {
            let before = &text[..index];
            let after = &text[index + character.len_utf8()..];

            return(before, Some(after));
        }
    }
    (text, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Finding Tests

    #[test]
    fn finds_class_containing_cursor_line() {
        let source = "\
    class Cart:
        def calculate_total(self):
            return 0
    ";
        let tree = parse_python(source).unwrap();
        let root = tree.root_node();

        let class_node = find_class_definition_containing_line(root, 2);

        assert!(class_node.is_some());
    }

    #[test]
    fn finds_function_that_starts_on_cursor_line() {
        let source = "def calculate_total(price, tax_rate):\n   return price * tax_rate";
        let tree = parse_python(source).unwrap();
        let root = tree.root_node();

        let function_node = find_function_definition_starting_on_line(root, 0);

        assert!(function_node.is_some());
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
    fn finds_local_if_scope_containing_cursor_line() {
        let source = "\
            def calculate_total(price):
                if price > 0:
                    return price
            ";
        let tree = parse_python(source).unwrap();
        let root = tree.root_node();

        let scope_node = find_local_scope_node_containing_line(root, 2);

        assert!(scope_node.is_some());
        assert_eq!(
            describe_local_scope_node(scope_node.unwrap()),
            "if statement"
        );
    }

    #[test]
    fn finds_inner_local_scope_when_scopes_are_nested() {
        let source = "\
            def process_items(items):
                for item in items:
                    if item:
                        print(item)
            ";
        let tree = parse_python(source).unwrap();
        let root = tree.root_node();

        let scope_node = find_local_scope_node_containing_line(root, 3);

        assert!(scope_node.is_some());
        assert_eq!(
            describe_local_scope_node(scope_node.unwrap()),
            "if statement"
        );
    }

    #[test]
    fn finds_for_loop_scope_containing_cursor_line() {
        let source = "\
            def process_items(items):
                for item in items:
                    print(item)
            ";
        let tree = parse_python(source).unwrap();
        let root = tree.root_node();

        let scope_node = find_local_scope_node_containing_line(root, 2);

        assert!(scope_node.is_some());
        assert_eq!(
            describe_local_scope_node(scope_node.unwrap()),
            "for loop"
        );
    }

    // Does Not Find Tests

    // No Class

    #[test]
    fn does_not_find_class_when_cursor_is_outside_class() {
        let source = "\
    class Cart:
        pass

    x = 10
    ";
        let tree = parse_python(source).unwrap();
        let root = tree.root_node();

        let class_node = find_class_definition_containing_line(root, 3);

        assert!(class_node.is_none());
    }

    // No Function

    #[test]
    fn does_not_find_starting_function_when_cursor_is_in_body() {
        let source = "def calculate_total(price, tax_rate):\n   return price * tax_rate";
        let tree = parse_python(source).unwrap();
        let root = tree.root_node();

        let function_node = find_function_definition_starting_on_line(root, 1);

        assert!(function_node.is_none());
    }

    // No Local Scope
    
    #[test]
    fn does_not_find_local_scope_contains_cursor_line() {
        let source = "\
            def calculate_total(price):
                return price
            ";
        let tree = parse_python(source).unwrap();
        let root = tree.root_node();

        let scope_node = find_local_scope_node_containing_line(root, 1);

        assert!(scope_node.is_none());
    }


    // Getting Tests

    #[test]
    fn gets_class_name() {
        let source = "\
    class Cart:
        pass
    ";
        let tree = parse_python(source).unwrap();
        let root = tree.root_node();

        let class_node = find_class_definition_containing_line(root, 1)
            .expect("Expected class node");

        let name = get_class_name(class_node, source);

        assert_eq!(name, "Cart");
    }


    #[test]
    fn gets_function_name() {
        let source = "\
    def calculate_total():
        return 0
    ";
        let tree = parse_python(source).unwrap();
        let root = tree.root_node();

        let function_node = find_function_definition_containing_line(root, 1)
            .expect("Expected function node");

        let name = get_function_name(function_node, source);

        assert_eq!(name, "calculate total");
    }

    // Describing Tests
    
    #[test]
    fn describes_function_parameters_only() {
        let source = "def calculate_total(price, tax_rate):\n    return price * tax_rate";
        let tree = parse_python(source).unwrap();
        let root = tree.root_node();

        let function_node =
            find_function_definition_starting_on_line(root, 0)
                .expect("Expected function node");

        let speech = describe_function_parameters(function_node, source);

        assert_eq!(speech, "Parameters: price; tax rate.");
    }

    #[test]
    fn describes_typed_function_parameters_only() {
        let source = "def calculate_total(price: float, tax_rate: float = 0.19) -> float:\n    return price * tax_rate";
        let tree = parse_python(source).unwrap();
        let root = tree.root_node();

        let function_node =
            find_function_definition_starting_on_line(root, 0)
                .expect("Expected function node");

        let speech = describe_function_parameters(function_node, source);

        assert_eq!(
            speech,
            "Parameters: price, float; tax rate, float, default zero point one nine."
        );
    }

    #[test]
    fn describes_no_parameters_only() {
        let source = "def main():\n    return 0";
        let tree = parse_python(source).unwrap();
        let root = tree.root_node();

        let function_node =
            find_function_definition_starting_on_line(root, 0)
                .expect("Expected function node");

        let speech = describe_function_parameters(function_node, source);

        assert_eq!(speech, "No parameters.");
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
    fn describes_typed_parameters_and_return_type() {
        let source = "def calculate_total(price: float, tax_rate: float) -> float:\n    return price * tax_rate";
        let tree = parse_python(source).unwrap();
        let root = tree.root_node();

        let function_node = 
            find_function_definition_starting_on_line(root, 0)
                .expect("Expected function node");
        
        let speech = describe_function_definition(function_node, source);

        assert_eq!(
            speech,
            "Function calculate total. Parameters: price, float; tax rate, float. Returns float."
        );
    }

    #[test]
    fn describes_default_parameter_values() {
        let source = "def calculate_total(price: float, tax_rate: float = 0.19) -> float:\n    return price * (1 + tax_rate)";
        let tree = parse_python(source).unwrap();
        let root = tree.root_node();

        let function_node =
            find_function_definition_starting_on_line(root, 0).expect("Expected function node");
        
        let speech = describe_function_definition(function_node, source);

        assert_eq!(
            speech,
            "Function calculate total. Parameters: price, float; tax rate, float, default zero point one nine. Returns float."
        );
    }

    #[test]
    fn describes_string_default_values() {
        let source = "def greet(name: str = \"guest\") -> str:\n    return name";
        let tree = parse_python(source).unwrap();
        let root = tree.root_node();

        let function_node =
            find_function_definition_starting_on_line(root, 0).expect("Expected function node");

        let speech = describe_function_definition(function_node, source);

        assert_eq!(
            speech,
            "Function greet. Parameters: name, str, default guest. Returns str."
        );
    }

    #[test]
    fn describes_union_like_annotations() {
        let source = "def find_user(name: str | None = None) -> bool:\n    return True";
        let tree = parse_python(source).unwrap();
        let root = tree.root_node();

        let function_node =
            find_function_definition_starting_on_line(root, 0).expect("Expected function node");

        let speech = describe_function_definition(function_node, source);

        assert_eq!(
            speech,
            "Function find user. Parameters: name, str or none, default none. Returns bool."
        );
    }

    // Splitting Tests

    #[test]
    fn splits_top_level_commas_without_splitting_inside_defaults() {
        let parts = split_top_level_commas("value: tuple[int, int] = (1, 2), name: str = \"a,b\"");

        assert_eq!(
            parts,
            vec![
                "value: tuple[int, int] = (1, 2)".to_string(),
                "name: str = \"a,b\"".to_string()
            ]
        );
    }
}