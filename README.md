# oxidoc-highlight

Lightweight, zero-dependency syntax highlighting that emits HTML with `<span class="tok-*">` tokens. Designed for documentation engines where you want fast, predictable output without pulling in tree-sitter or heavyweight grammars.

## Supported Languages

Bash, C, CSS, Diff, Go, HTML, Java, JavaScript/JSX, JSON, Markdown, PHP, Python, RDX, Rust, SQL, TOML, TypeScript/TSX, XML, YAML

## Usage

```rust
let html = oxidoc_highlight::highlight("let x = 42;", "rust");
// → <span class="tok-keyword">let</span> x <span class="tok-operator">=</span> <span class="tok-number">42</span><span class="tok-punctuation">;</span>
```

```rust
// Check language support
assert!(oxidoc_highlight::is_supported("rust"));
let langs = oxidoc_highlight::supported_languages();
```

## Wasm

Enable the `wasm` feature to get `wasm-bindgen` exports:

```toml
oxidoc-highlight = { version = "0.1", features = ["wasm"] }
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
