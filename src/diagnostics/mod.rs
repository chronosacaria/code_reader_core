use crate::model::ReaderDiagnostic;

/// Describes diagnostics on or near the cursor line.
///
/// This module is intentionally language-agnostic.
///
/// Diagnostics should usually come from an editor, compiler, linter, or
/// language server. The core receives them as data and turns them into
/// speech-friendly text.
pub fn describe_diagnostics_near_cursor(
    diagnostics: &[ReaderDiagnostic],
    cursor_line: usize,
) -> String {
    if diagnostics.is_empty() {
        return "No diagnostics near cursor.".to_string();
    }

    let diagnostics_on_cursor_line = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.contains_line(cursor_line))
        .collect::<Vec<_>>();

    if !diagnostics_on_cursor_line.is_empty() {
        return describe_diagnostics_on_cursor_line(&diagnostics_on_cursor_line);
    }

    describe_nearest_diagnostics(diagnostics, cursor_line)
}

/// Describes diagnostics that directly overlap the cursor line.
fn describe_diagnostics_on_cursor_line(diagnostics: &[&ReaderDiagnostic]) -> String {
    if diagnostics.len() == 1 {
        let diagnostic = diagnostics[0];

        return format!(
            "{} on current line: {}.",
            diagnostic.severity.speech_label(),
            diagnostic.message,
        );
    }

    let diagnostic_speech = diagnostics
        .iter()
        .map(|diagnostic| describe_diagnostic_brief(diagnostic))
        .collect::<Vec<_>>()
        .join("; ");

    format! (
        "{} diagnostics on current line. {diagnostic_speech}.",
        diagnostics.len(),
    )
}

/// Describes the nearest diagnostics when no diagnostic overlaps the cursor line.
fn describe_nearest_diagnostics(
    diagnostics: &[ReaderDiagnostic],
    cursor_line: usize,
) -> String {
    let nearest_distance = diagnostics
        .iter()
        .map(|diagnostic| distance_from_line(diagnostic, cursor_line))
        .min()
        .expect("Expected at least one diagnostic");

    let nearest_diagnostics = diagnostics
        .iter()
        .filter(|diagnostic| distance_from_line(diagnostic, cursor_line) == nearest_distance)
        .collect::<Vec<_>>();

    if nearest_diagnostics.len() == 1 {
        let diagnostic = nearest_diagnostics[0];

        return format!(
            "Nearest diagnostic on line {}. {}: {}.",
            diagnostic.display_start_line(),
            diagnostic.severity.speech_label(),
            diagnostic.message,
        );
    }

    let diagnostic_speech = nearest_diagnostics
        .iter()
        .map(|diagnostic| {
            format!(
                "Line {}: {}",
                diagnostic.display_start_line(),
                describe_diagnostic_brief(diagnostic),
            )
        })
        .collect::<Vec<_>>()
        .join("; ");

    format!(
        "{} nearest diagnostics. {diagnostic_speech}.",
        nearest_diagnostics.len(),
    )
}

/// Gives a short speech version of one diagnostic.
/// 
/// Example:
/// 
/// `Error: Undefined name taxrate`
fn describe_diagnostic_brief(diagnostic: &ReaderDiagnostic) -> String {
    format!(
        "{}: {}",
        diagnostic.severity.speech_label(),
        diagnostic.message,
    )
}

/// Calculates how far a diagnostic is from a zero-based line.
///
/// If the diagnostic contains the line, the distance is zero.
/// If the diagnostic is before the line, the distance is the number of lines
/// between the diagnostic end and the cursor.
/// If the diagnostic is after the line, the distance is the number of lines
/// between the cursor and the diagnostic start.
fn distance_from_line(diagnostic: &ReaderDiagnostic, line: usize) -> usize {
    if diagnostic.contains_line(line) {
        return 0;
    }

    if line < diagnostic.start_line {
        diagnostic.start_line - line
    } else {
        line - diagnostic.effective_end_line()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{DiagnosticSeverity, ReaderDiagnostic};

    fn diagnostic(
        severity: DiagnosticSeverity,
        message: &str,
        start_line: usize,
    ) -> ReaderDiagnostic {
        ReaderDiagnostic { 
            severity, 
            message: message.to_string(), 
            start_line, 
            end_line: None, 
            source: None, 
            code: None, 
        }
    }

    #[test]
    fn describes_no_diagnostics() {
        let speech = describe_diagnostics_near_cursor(&[], 0);

        assert_eq!(speech, "No diagnostics near cursor.");
    }

    #[test]
    fn describes_error_on_cursor_line() {
        let diagnostics = vec![diagnostic(
            DiagnosticSeverity::Error,
            "Undefined name taxrate",
            0,
        )];

        let speech = describe_diagnostics_near_cursor(&diagnostics, 0);

        assert_eq!(speech, "Error on current line: Undefined name taxrate.");
    }

    #[test]
    fn describes_warning_on_cursor_line() {
        let diagnostics = vec![diagnostic(
            DiagnosticSeverity::Warning,
            "Unused variable price",
            2,
        )];

        let speech = describe_diagnostics_near_cursor(&diagnostics, 2);

        assert_eq!(speech, "Warning on current line: Unused variable price.");
    }

    #[test]
    fn describes_information_on_cursor_line() {
        let diagnostics = vec![diagnostic(
            DiagnosticSeverity::Information,
            "Type is float",
            1,
        )];

        let speech = describe_diagnostics_near_cursor(&diagnostics, 1);

        assert_eq!(speech, "Information on current line: Type is float.");
    }

    #[test]
    fn describes_hint_on_cursor_line() {
        let diagnostics = vec![diagnostic(
            DiagnosticSeverity::Hint,
            "Consider simplifying this expression",
            1,
        )];

        let speech = describe_diagnostics_near_cursor(&diagnostics, 1);

        assert_eq!(
            speech,
            "Hint on current line: Consider simplifying this expression."
        );
    }

    #[test]
    fn describes_multiple_diagnostics_on_cursor_line() {
        let diagnostics = vec![
            diagnostic(DiagnosticSeverity::Error, "Undefined name taxrate", 0),
            diagnostic(DiagnosticSeverity::Warning, "Unused import math", 0),
        ];

        let speech = describe_diagnostics_near_cursor(&diagnostics, 0);

        assert_eq!(
            speech,
            "2 diagnostics on current line. Error: Undefined name taxrate; Warning: Unused import math."
        );
    }

    #[test]
    fn describes_nearest_diagnostic_when_none_are_on_cursor_line() {
        let diagnostics = vec![diagnostic(
            DiagnosticSeverity::Error,
            "Undefined name taxrate",
            3,
        )];

        let speech = describe_diagnostics_near_cursor(&diagnostics, 1);

        assert_eq!(
            speech,
            "Nearest diagnostic on line 4. Error: Undefined name taxrate."
        );
    }

    #[test]
    fn prefers_exact_line_over_nearby_diagnostic() {
        let diagnostics = vec![
            diagnostic(DiagnosticSeverity::Warning, "Unused import math", 0),
            diagnostic(DiagnosticSeverity::Error, "Undefined name taxrate", 2),
        ];

        let speech = describe_diagnostics_near_cursor(&diagnostics, 2);

        assert_eq!(speech, "Error on current line: Undefined name taxrate.");
    }

    #[test]
    fn handles_multiline_diagnostic_covering_cursor_line() {
        let diagnostics = vec![ReaderDiagnostic {
            severity: DiagnosticSeverity::Error,
            message: "Unclosed string literal".to_string(),
            start_line: 1,
            end_line: Some(3),
            source: None,
            code: None,
        }];

        let speech = describe_diagnostics_near_cursor(&diagnostics, 2);

        assert_eq!(speech, "Error on current line: Unclosed string literal.");
    }

    #[test]
    fn describes_two_equally_near_diagnostics() {
        let diagnostics = vec![
            diagnostic(DiagnosticSeverity::Warning, "Issue before cursor", 1),
            diagnostic(DiagnosticSeverity::Error, "Issue after cursor", 3),
        ];

        let speech = describe_diagnostics_near_cursor(&diagnostics, 2);

        assert_eq!(
            speech,
            "2 nearest diagnostics. Line 2: Warning: Issue before cursor; Line 4: Error: Issue after cursor."
        );
    }
}