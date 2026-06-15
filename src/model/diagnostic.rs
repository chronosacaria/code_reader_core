use serde::{Deserialize, Serialize};

/// The severity level of a diagnostic.
///
/// These values are intentionally generic so they can represent diagnostics
/// from different tools later
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

impl DiagnosticSeverity {
    /// Converts the severity into speech-friendly text.
    ///
    /// This version returns title-case labels because they often start
    /// diagnostic sentences.
    pub fn speech_label(self) -> &'static str {
        match self {
            DiagnosticSeverity::Error => "Error",
            DiagnosticSeverity::Warning => "Warning",
            DiagnosticSeverity::Information => "Information",
            DiagnosticSeverity::Hint => "Hint",
        }
    }
}

/// A diagnostic provided by an editor, language server, linter, or compiler.
///
/// The code reader core does not create most diagnostics itself.
/// Instead, diagnostics should usually be supplied by the caller.
///
/// For example, a future VS Code extension can read diagnostics from VS Code
/// and translate them into this structure before calling the Rust core.
///
/// Line numbers are zero-based, matching `cursor_line`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReaderDiagnostic {
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub start_line: usize,

    /// Optional end line for diagnostics that span multiple lines.
    ///
    /// If this is absent, the diagnostic is treated as a single-line diagnostic
    /// on `start_line`.
    #[serde(default)]
    pub end_line: Option<usize>,

    /// Optional source of the diagnostic, such as `pyright`, `rust-analyzer`,
    /// or `eslint`.
    #[serde(default)]
    pub source: Option<String>,

    /// Optional diagnostic code, such as `reportUndefinedVariable`.
    #[serde(default)]
    pub code: Option<String>,
}

impl ReaderDiagnostic {
    /// Returns the effective end line of the diagnostic.
    ///
    /// If `end_line` is not provided, the diagnostic ends on its start line.
    pub fn effective_end_line(&self) -> usize {
        self.end_line.unwrap_or(self.start_line)
    }

    /// Returns true if this diagnostic covers the given zero-based line.
    pub fn contains_line(&self, line: usize) -> bool {
        self.start_line <= line && line <= self.effective_end_line()
    }

    /// Returns a one-based line number for user-facing speech.
    ///
    /// Internally, line numbers are zero-based.
    /// For users, line numbers should be one-based.
    pub fn display_start_line(&self) -> usize {
        self.start_line + 1
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn deserializes_diagnostic_severity_from_snake_case() {
        let severity: DiagnosticSeverity = serde_json::from_str(r#""error""#).unwrap();

        assert_eq!(severity, DiagnosticSeverity::Error);
    }

    #[test]
    fn serializes_diagnostic_severity_to_snake_case() {
        let json = serde_json::to_string(&DiagnosticSeverity::Information).unwrap();

        assert_eq!(json, r#""information""#);
    }

    #[test]
    fn diagnostic_without_end_line_is_single_line() {
        let diagnostic = ReaderDiagnostic {
            severity: DiagnosticSeverity::Error,
            message: "Undefined name taxrate".to_string(),
            start_line: 3,
            end_line: None,
            source: None,
            code: None,
        };

        assert_eq!(diagnostic.effective_end_line(), 3);
        assert!(diagnostic.contains_line(3));
        assert!(!diagnostic.contains_line(2));
        assert!(!diagnostic.contains_line(4));
    }

    #[test]
    fn diagnostic_with_end_line_can_span_multiple_lines() {
        let diagnostic = ReaderDiagnostic {
            severity: DiagnosticSeverity::Warning,
            message: "Multiline issue".to_string(),
            start_line: 3,
            end_line: Some(5),
            source: None,
            code: None,
        };

        assert!(diagnostic.contains_line(3));
        assert!(diagnostic.contains_line(4));
        assert!(diagnostic.contains_line(5));
        assert!(!diagnostic.contains_line(2));
        assert!(!diagnostic.contains_line(6));
    }

    #[test]
    fn display_start_line_is_one_based() {
        let diagnostic = ReaderDiagnostic {
            severity: DiagnosticSeverity::Hint,
            message: "Consider simplifying this expression".to_string(),
            start_line: 0,
            end_line: None,
            source: None,
            code: None,
        };

        assert_eq!(diagnostic.display_start_line(), 1);
    }

    #[test]
    fn deserializes_reader_diagnostic_with_optional_fields() {
        let json = r#"
        {
            "severity": "error",
            "message": "Undefined name taxrate",
            "start_line": 0,
            "source": "pyright",
            "code": "reportUndefinedVariable"
        }
        "#;

        let diagnostic: ReaderDiagnostic = serde_json::from_str(json).unwrap();

        assert_eq!(diagnostic.severity, DiagnosticSeverity::Error);
        assert_eq!(diagnostic.message, "Undefined name taxrate");
        assert_eq!(diagnostic.start_line, 0);
        assert_eq!(diagnostic.end_line, None);
        assert_eq!(diagnostic.source, Some("pyright".to_string()));
        assert_eq!(
            diagnostic.code,
            Some("reportUndefinedVariable".to_string())
        );
    }
}