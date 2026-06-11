use std::io::{self, Read};

use code_reader_core::{read_code, ReaderInput};

fn main() {
    let mut stdin_text = String::new();

    io::stdin()
        .read_to_string(&mut stdin_text)
        .expect("Failed to read from stdin");

    let input: ReaderInput =
        serde_json::from_str(&stdin_text)
            .expect("Failed to parse JSON input");

    let output = read_code(input);

    let json_output = 
        serde_json::to_string_pretty(&output)
            .expect("Failed to serialise JSON output");

    println!("{json_output}");
}