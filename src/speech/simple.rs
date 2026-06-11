/// Converts a punctuation-heavy programming line into a very rough speech form.
///
/// This is not the final speech system.
/// It is just enough to prove that the CLI and tests work.
pub fn make_simple_speech_text(line: &str) -> String {
    line.replace("(", " ")
        .replace(")", " ")
        .replace(":", " ")
        .replace(",", " ")
        .replace("_", " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}