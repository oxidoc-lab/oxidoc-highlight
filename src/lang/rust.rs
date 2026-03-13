use crate::scanner::*;
use crate::token::{Token, TokenKind};

pub struct RustScanner;

const KEYWORDS: &[&[u8]] = &[
    b"as",
    b"async",
    b"await",
    b"break",
    b"const",
    b"continue",
    b"crate",
    b"dyn",
    b"else",
    b"enum",
    b"extern",
    b"false",
    b"fn",
    b"for",
    b"if",
    b"impl",
    b"in",
    b"let",
    b"loop",
    b"match",
    b"mod",
    b"move",
    b"mut",
    b"pub",
    b"ref",
    b"return",
    b"static",
    b"struct",
    b"super",
    b"trait",
    b"true",
    b"type",
    b"unsafe",
    b"use",
    b"where",
    b"while",
    b"yield",
    b"Self",
];

const TYPE_KEYWORDS: &[&[u8]] = &[
    b"i8",
    b"i16",
    b"i32",
    b"i64",
    b"i128",
    b"isize",
    b"u8",
    b"u16",
    b"u32",
    b"u64",
    b"u128",
    b"usize",
    b"f32",
    b"f64",
    b"bool",
    b"char",
    b"str",
    b"String",
    b"Vec",
    b"Option",
    b"Result",
    b"Box",
    b"Rc",
    b"Arc",
    b"HashMap",
    b"HashSet",
    b"BTreeMap",
    b"BTreeSet",
    b"Cow",
    b"Pin",
    b"Cell",
    b"RefCell",
    b"Mutex",
    b"RwLock",
];

const BUILTINS: &[&[u8]] = &[
    b"println",
    b"print",
    b"eprintln",
    b"eprint",
    b"format",
    b"write",
    b"writeln",
    b"vec",
    b"todo",
    b"unimplemented",
    b"unreachable",
    b"panic",
    b"assert",
    b"assert_eq",
    b"assert_ne",
    b"debug_assert",
    b"debug_assert_eq",
    b"debug_assert_ne",
    b"cfg",
    b"include",
    b"include_str",
    b"include_bytes",
    b"env",
    b"concat",
    b"stringify",
    b"file",
    b"line",
    b"column",
    b"module_path",
];

fn at(b: &[u8], i: usize) -> u8 {
    if i < b.len() { b[i] } else { 0 }
}

impl Scanner for RustScanner {
    fn scan(&self, code: &str) -> Vec<Token> {
        let b = code.as_bytes();
        let mut tokens = Vec::new();
        let mut i = 0;
        let mut prev_kind: Option<TokenKind> = None;

        while i < b.len() {
            let c = b[i];

            // Whitespace / newlines — skip
            if c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' {
                i += 1;
                continue;
            }

            // Attributes: #[...] or #![...]
            if c == b'#' && (at(b, i + 1) == b'[' || (at(b, i + 1) == b'!' && at(b, i + 2) == b'['))
            {
                let start = i;
                i += if at(b, i + 1) == b'!' { 3 } else { 2 };
                let mut depth = 1u32;
                while i < b.len() && depth > 0 {
                    if b[i] == b'[' {
                        depth += 1;
                    } else if b[i] == b']' {
                        depth -= 1;
                    }
                    i += 1;
                }
                tokens.push(Token {
                    kind: TokenKind::Attr,
                    start,
                    end: i,
                });
                prev_kind = Some(TokenKind::Attr);
                continue;
            }

            // Comments
            if let Some(end) = scan_c_comment(b, i) {
                tokens.push(Token {
                    kind: TokenKind::Comment,
                    start: i,
                    end,
                });
                prev_kind = Some(TokenKind::Comment);
                i = end;
                continue;
            }

            // Raw strings: r"...", r#"..."#, r##"..."##, etc.
            if c == b'r'
                && (at(b, i + 1) == b'"' || at(b, i + 1) == b'#')
                && let Some(end) = scan_rust_raw_string(b, i)
            {
                tokens.push(Token {
                    kind: TokenKind::String,
                    start: i,
                    end,
                });
                prev_kind = Some(TokenKind::String);
                i = end;
                continue;
            }

            // Byte strings: b"...", b'...'
            if c == b'b' && (at(b, i + 1) == b'"' || at(b, i + 1) == b'\'') {
                let delim = b[i + 1];
                if let Some(end) = scan_quoted_string(b, i + 1, delim) {
                    tokens.push(Token {
                        kind: TokenKind::String,
                        start: i,
                        end,
                    });
                    prev_kind = Some(TokenKind::String);
                    i = end;
                    continue;
                }
            }

            // Strings
            if c == b'"'
                && let Some(end) = scan_double_string(b, i)
            {
                tokens.push(Token {
                    kind: TokenKind::String,
                    start: i,
                    end,
                });
                prev_kind = Some(TokenKind::String);
                i = end;
                continue;
            }

            // Char literals
            if c == b'\'' {
                if let Some(end) = scan_rust_char_or_lifetime(b, i) {
                    // It's a char literal
                    tokens.push(Token {
                        kind: TokenKind::String,
                        start: i,
                        end,
                    });
                    prev_kind = Some(TokenKind::String);
                    i = end;
                    continue;
                }
                // Lifetime: 'a, 'static, etc.
                if at(b, i + 1).is_ascii_alphabetic() || at(b, i + 1) == b'_' {
                    let start = i;
                    i += 1;
                    while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                        i += 1;
                    }
                    tokens.push(Token {
                        kind: TokenKind::Variable,
                        start,
                        end: i,
                    });
                    prev_kind = Some(TokenKind::Variable);
                    continue;
                }
            }

            // Numbers
            if let Some(end) = scan_number(b, i) {
                tokens.push(Token {
                    kind: TokenKind::Number,
                    start: i,
                    end,
                });
                prev_kind = Some(TokenKind::Number);
                i = end;
                continue;
            }

            // Identifiers / keywords
            if let Some((end, ident)) = scan_ident(b, i) {
                let kind = if is_keyword(ident, KEYWORDS) {
                    TokenKind::Keyword
                } else if ident == b"self" {
                    TokenKind::Variable
                } else if is_keyword(ident, TYPE_KEYWORDS) || is_pascal_case(ident) {
                    TokenKind::Type
                } else if is_keyword(ident, BUILTINS) && at(b, end) == b'!' {
                    TokenKind::Builtin
                } else if at(b, end) == b'!' {
                    // Any macro call: name!
                    TokenKind::Builtin
                } else if is_function_call(b, end) && !was_keyword(prev_kind) {
                    TokenKind::Function
                } else if matches!(prev_kind, Some(TokenKind::Punctuation))
                    && i >= 1
                    && b[i - 1] == b'.'
                {
                    // After a dot — property or method
                    if is_function_call(b, end) {
                        TokenKind::Function
                    } else {
                        TokenKind::Property
                    }
                } else {
                    TokenKind::Plain
                };
                tokens.push(Token {
                    kind,
                    start: i,
                    end,
                });
                prev_kind = Some(kind);
                i = end;
                continue;
            }

            // Operators
            if let Some(end) = scan_operator(b, i) {
                tokens.push(Token {
                    kind: TokenKind::Operator,
                    start: i,
                    end,
                });
                prev_kind = Some(TokenKind::Operator);
                i = end;
                continue;
            }

            // Punctuation
            if let Some(end) = scan_punctuation(b, i) {
                tokens.push(Token {
                    kind: TokenKind::Punctuation,
                    start: i,
                    end,
                });
                prev_kind = Some(TokenKind::Punctuation);
                i = end;
                continue;
            }

            // Unknown byte — skip
            i += 1;
        }

        tokens
    }
}

/// Scan a Rust raw string: r"...", r#"..."#, r##"..."## etc.
fn scan_rust_raw_string(b: &[u8], pos: usize) -> Option<usize> {
    if at(b, pos) != b'r' {
        return None;
    }
    let mut i = pos + 1;
    let mut hashes = 0u32;
    while i < b.len() && b[i] == b'#' {
        hashes += 1;
        i += 1;
    }
    if at(b, i) != b'"' {
        return None;
    }
    i += 1;
    // Search for closing "###
    while i < b.len() {
        if b[i] == b'"' {
            let mut j = 0u32;
            while j < hashes && at(b, i + 1 + j as usize) == b'#' {
                j += 1;
            }
            if j == hashes {
                return Some(i + 1 + hashes as usize);
            }
        }
        i += 1;
    }
    Some(b.len()) // unterminated
}

/// Scan a Rust char literal like 'a', '\n', '\x41', '\u{1F600}'.
/// Returns None if it's a lifetime instead.
fn scan_rust_char_or_lifetime(b: &[u8], pos: usize) -> Option<usize> {
    if at(b, pos) != b'\'' {
        return None;
    }
    let mut i = pos + 1;
    if i >= b.len() {
        return None;
    }
    if b[i] == b'\\' {
        // Escaped char
        i += 1;
        if i >= b.len() {
            return None;
        }
        match b[i] {
            b'x' => {
                i += 1;
                while i < b.len() && b[i].is_ascii_hexdigit() {
                    i += 1;
                }
            }
            b'u' => {
                i += 1;
                if at(b, i) == b'{' {
                    i += 1;
                    while i < b.len() && b[i] != b'}' {
                        i += 1;
                    }
                    if i < b.len() {
                        i += 1;
                    }
                }
            }
            _ => {
                i += 1;
            }
        }
    } else {
        // Regular char — must be a single char followed by '
        // If it's alphanumeric and NOT followed by ', it's a lifetime
        if b[i].is_ascii_alphanumeric() || b[i] == b'_' {
            i += 1;
            // Check for immediate closing quote
            if at(b, i) == b'\'' {
                return Some(i + 1);
            }
            return None; // It's a lifetime, not a char
        }
        i += 1;
    }
    if at(b, i) == b'\'' { Some(i + 1) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::render;

    fn hl(code: &str) -> String {
        let tokens = RustScanner.scan(code);
        render(code, &tokens)
    }

    #[test]
    fn keywords_and_types() {
        assert_eq!(
            hl("let x: i32 = 5;"),
            "<span class=\"tok-keyword\">let</span> x<span class=\"tok-punctuation\">:</span> <span class=\"tok-type\">i32</span> <span class=\"tok-operator\">=</span> <span class=\"tok-number\">5</span><span class=\"tok-punctuation\">;</span>"
        );
    }

    #[test]
    fn function_call() {
        let out = hl("foo(x)");
        assert!(out.contains("tok-function"));
        assert!(out.contains("foo"));
    }

    #[test]
    fn macro_call() {
        let out = hl("println!(\"hi\")");
        assert!(out.contains("tok-builtin"));
    }

    #[test]
    fn raw_string() {
        let out = hl(r##"r#"hello"#"##);
        assert!(out.contains("tok-string"));
    }

    #[test]
    fn lifetime() {
        let out = hl("'a");
        assert!(out.contains("tok-variable"));
    }

    #[test]
    fn attribute() {
        let out = hl("#[derive(Debug)]");
        assert!(out.contains("tok-attr"));
    }

    #[test]
    fn comment() {
        let out = hl("// comment");
        assert!(out.contains("tok-comment"));
    }

    #[test]
    fn block_comment() {
        let out = hl("/* block */");
        assert!(out.contains("tok-comment"));
    }

    #[test]
    fn string_with_escapes() {
        let out = hl(r#""he said \"hello\"""#);
        assert!(out.contains("tok-string"));
        // Should be a single string token
        assert_eq!(out.matches("tok-string").count(), 1);
    }

    #[test]
    fn pascal_case_type() {
        let out = hl("MyStruct");
        assert!(out.contains("tok-type"));
    }

    #[test]
    fn property_access() {
        let out = hl("x.field");
        assert!(out.contains("tok-property"));
    }

    #[test]
    fn method_call() {
        let out = hl("x.method()");
        assert!(out.contains("tok-function"));
        assert!(out.contains("method"));
    }

    #[test]
    fn hex_number() {
        let out = hl("0xFF");
        assert!(out.contains("tok-number"));
    }

    #[test]
    fn self_variable() {
        let out = hl("self");
        assert!(out.contains("tok-variable"));
    }

    #[test]
    fn html_escape() {
        let out = hl("x < y && z > w");
        assert!(out.contains("&lt;"));
        assert!(out.contains("&gt;"));
        assert!(out.contains("&amp;&amp;"));
    }
}
