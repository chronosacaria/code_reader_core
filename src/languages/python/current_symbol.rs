// Questions Answered:
// What symbol am I on?
// What token is under the cursor?

use super::syntax::describe_symbol_at_position;

/// Describes the Python symbol or token at the cursor.
pub fn describe_current_symbol(
    source: &str,
    cursor_line: usize,
    cursor_column: usize,
) -> Option<String> {
    describe_symbol_at_position(source, cursor_line, cursor_column)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describe_symbol_under_cursor() {
        let source = "total = price + tax_rate";

        let speech = describe_current_symbol(source, 0, 16);

        assert_eq!(speech, Some("Symbol tax rate.".to_string()));
    }
    
    #[test]
    fn describes_operator_under_cursor() {
        let source = "total = price + tax_rate";

        let speech = describe_current_symbol(source, 0, 14);

        assert_eq!(speech, Some("Operator plus.".to_string()));
    }

    #[test]
    fn returns_none_when_no_symbol_is_under_cursor() {
        let source = "total = price + tax_rate";

        let speech = describe_current_symbol(source, 0, 13);

        assert_eq!(speech, None);
    }
}