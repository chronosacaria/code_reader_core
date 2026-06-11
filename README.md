# Code Reader Core
Code Reader Core is the foundational core of a screen reader extension that can read out code in
various programming languages in order to facilitate easier checking, troubleshooting, development
and maintainence by low vision and blind developers.
## Goals
In an attempt to avoid as much feature creep as possible, the basic goals for this project are as
follows:
- Multiple Programming Language Support
- Speech Output in English, initially, with the goal of other Language outputs
- Initially focusing on VS Code with the plan of making the tool as IDE and language agnostic as
  possible
- Different levels of feedback and verbosity
- Initially build out with Speech Dispatcher for basic readout of code
## Example of Intention
The ideal output would be based on the requested level of verbosity. A simplified example can be
as follows:

Code to be Read:

Rust Example
```rust
def calculate_total(price: float, tax_rate: float) -> float:
  return price * (1 + tax_rate)
```

Java Example
```java
public float calculateTotal(float price, float taxRate) {
  return price * (1 + taxRate);
}
```
Basic Intended Speech Output should both be:
```
"Function calculate total. Parameters: price, float; tax_rate, float. Returns float."
```
It is worth noting that characters such as `(`, `)`, `_`, etc. can be excluded from the speech output
for a more natural sounding speech.

# Artificial Intelligence (AI) Usage Disclosure
As I am blind, I have found that using AI to check behind me with my code to be extremely useful. 
Whilst I do not intend to vibe-code this application, I believe that it is important to disclose
when AI is used in one's production pipeline. As such, this disclosure should serve as an indication
that this project is being created with the *assistance* of Artificial Intelligence, but that the
code is not being solely "created" and "provided" by the Artificial Intelligence that is being used.

An attempt has not been made to verify the usage of AI in the development of Dependencies or Visual
Studio Code Extensions.

# Rust Comprehension Disclosure
It is important to note that this is the first project that I have ever made with Rust. I appreciate
any and all constructive feedback from various members of the programming community with how I can
improve the code that I am writing.

# Dependencies Used
This list should not be considered exhaustive until the completion of this project. This list will
be updated as the project evolves. The names of the extensions, their license and any links to
repositiories shall be provided.
- serde
  - Author: serde-rs
  - Licenses: [Apache 2.0](https://github.com/serde-rs/serde/blob/master/LICENSE-APACHE),
    [MIT License](https://github.com/serde-rs/serde/blob/master/LICENSE-MIT)
  - Repo Link: [https://github.com/serde-rs/serde](https://github.com/serde-rs/serde)
- serde_json
  - Author: serde-rs
  - Licenses: [Apache 2.0](https://github.com/serde-rs/json/blob/master/LICENSE-APACHE),
    [MIT License](https://github.com/serde-rs/json/blob/master/LICENSE-MIT)
  - Repo Link: [https://github.com/serde-rs/serde](https://github.com/serde-rs/json)

# Visual Studio Code Extensions Used
This list should not be considered exhaustive until the completion of this project. This list will
be updated as the project evolves. The names of the extensions, their license and any links to
repositiories shall be provided.
- CodeLLDB
  - Author: Vadim Chugunov
  - License: [MIT License](https://github.com/vadimcn/codelldb/blob/master/LICENSE)
  - Repo Link: [https://github.com/vadimcn/codelldb/](https://github.com/vadimcn/codelldb/)
- Codex - OpenAI's coding agent
  - Author: OpenAI
  - License: [Apache 2.0](https://github.com/openai/codex/blob/main/LICENSE)
  - Repo Link: [https://github.com/openai/codex](https://github.com/openai/codex)
- Dependi
  - Author: Fill Labs
  - License: [Custom License](https://openvsx.eclipsecontent.org/fill-labs/dependi/0.7.22/LICENSE.txt)
  - Repo Link: https://github.com/filllabs/dependi
- Error Lense
  - Author: usernamehw
  - License: [MIT License](https://github.com/usernamehw/vscode-error-lens/blob/master/LICENSE)
  - Repo Link: [https://github.com/usernamehw/vscode-error-lens](https://github.com/usernamehw/vscode-error-lens)
- ESLint
  - Author: Microsoft
  - License: [MIT License](https://github.com/microsoft/vscode-eslint/blob/main/License.txt)
  - Repo Link: [https://github.com/Microsoft/vscode-eslint](https://github.com/Microsoft/vscode-eslint)
- Even Better TOML
  - Author: tamasfe
  - License: [MIT License](https://github.com/tamasfe/taplo/blob/master/LICENSE)
  - Repo Link: [https://github.com/tamasfe/taplo](https://github.com/tamasfe/taplo)
- Prettier - Code formatter
  - Author: Prettier
  - License: [MIT License](https://github.com/prettier/prettier-vscode/blob/main/LICENSE)
  - Repo Link: [https://github.com/prettier/prettier-vscode](https://github.com/prettier/prettier-vscode)
- rust-analyzer
  - Author: rust-lang
  - Licenses: [Apache 2.0](https://github.com/rust-lang/rust-analyzer/blob/master/LICENSE-APACHE),
    [MIT License](https://github.com/rust-lang/rust-analyzer/blob/master/LICENSE-MIT)
  - Repo Link: [https://github.com/rust-lang/rust-analyzer](https://github.com/rust-lang/rust-analyzer)
- Vitest
  - Author: Vitest
  - License: [MIT License](https://github.com/vitest-dev/vscode/blob/main/LICENSE)
  - Repo Link: [https://github.com/vitest-dev/vscode](https://github.com/vitest-dev/vscode)
