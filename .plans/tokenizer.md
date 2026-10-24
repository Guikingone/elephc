# Complete PHP tokenizer support

- [x] Implement a shared, byte-preserving PHP 8.5 tokenization engine and TOKEN_PARSE validation.
- [x] Expose token_get_all(), token_name(), PhpToken and all 154 baseline constants through AOT and eval.
- [x] Add focused regression and PHP differential coverage, plus a practical example.
- [x] Update generated builtin documentation and compatibility inventory.
- [ ] Open a draft PR, follow CI and resolve failures. Do not run local builds or tests.

The reference surface is the PHP 8.5.10 snapshot in scripts/docs/php_baseline.json,
confirmed against the installed PHP 8.5.12 tokenizer extension. The class includes
six methods and four public properties. Tokenization must preserve bytes, comments,
whitespace, line numbers and byte positions independently of elephc's source grammar.
Both backends must consume the same engine on every supported target. A Rust bridge
may use a maintained PHP lexer/parser with an explicit compatibility adapter.

Validation is CI-only at the user's request. No local build or test command is authorized.

Draft PR: https://github.com/illegalstudio/elephc/pull/2062

The PHP 8.5 differential corpus, native example and generated builtin docs pass CI.
The full executable matrix and both iOS link jobs remain under observation.
CI fixes include lexical compatibility, subclass validation, bridge archive inclusion
and the standalone codegen harness's on-demand bridge build list.
