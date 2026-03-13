use crate::scanner::*;
use crate::token::{Token, TokenKind};

pub struct JsonScanner;

fn at(b: &[u8], i: usize) -> u8 {
    if i < b.len() { b[i] } else { 0 }
}

impl Scanner for JsonScanner {
    fn scan(&self, code: &str) -> Vec<Token> {
        let b = code.as_bytes();
        let mut tokens = Vec::new();
        let mut i = 0;

        while i < b.len() {
            let c = b[i];

            if c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' {
                i += 1;
                continue;
            }

            // JSONC comments
            if let Some(end) = scan_c_comment(b, i) {
                tokens.push(Token {
                    kind: TokenKind::Comment,
                    start: i,
                    end,
                });
                i = end;
                continue;
            }

            // Strings (keys vs values determined by context: key if followed by :)
            if c == b'"'
                && let Some(end) = scan_double_string(b, i)
            {
                // Check if this is a key (followed by :)
                let after = skip_whitespace_no_newline(b, end);
                let kind = if at(b, after) == b':' {
                    TokenKind::Property
                } else {
                    TokenKind::String
                };
                tokens.push(Token {
                    kind,
                    start: i,
                    end,
                });
                i = end;
                continue;
            }

            // Numbers
            if c.is_ascii_digit() || (c == b'-' && at(b, i + 1).is_ascii_digit()) {
                let start = i;
                if c == b'-' {
                    i += 1;
                }
                if let Some(end) = scan_number(b, i) {
                    tokens.push(Token {
                        kind: TokenKind::Number,
                        start,
                        end,
                    });
                    i = end;
                    continue;
                }
                i = start;
            }

            // Keywords: true, false, null
            if let Some((end, ident)) = scan_ident(b, i) {
                let kind = match ident {
                    b"true" | b"false" | b"null" => TokenKind::Keyword,
                    _ => TokenKind::Plain,
                };
                tokens.push(Token {
                    kind,
                    start: i,
                    end,
                });
                i = end;
                continue;
            }

            // Punctuation
            if matches!(c, b'{' | b'}' | b'[' | b']' | b':' | b',') {
                tokens.push(Token {
                    kind: TokenKind::Punctuation,
                    start: i,
                    end: i + 1,
                });
                i += 1;
                continue;
            }

            i += 1;
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::render;

    fn hl(code: &str) -> String {
        let tokens = JsonScanner.scan(code);
        render(code, &tokens)
    }

    #[test]
    fn key_value() {
        let out = hl(r#"{"key": "value"}"#);
        assert!(out.contains("tok-property")); // key
        assert!(out.contains("tok-string")); // value
    }

    #[test]
    fn numbers() {
        let out = hl(r#"{"n": 42}"#);
        assert!(out.contains("tok-number"));
    }

    #[test]
    fn booleans_null() {
        let out = hl(r#"{"a": true, "b": false, "c": null}"#);
        assert!(out.contains("tok-keyword"));
    }

    #[test]
    fn nested() {
        let out = hl(r#"{"a": {"b": [1, 2]}}"#);
        assert!(out.contains("tok-property"));
        assert!(out.contains("tok-number"));
    }

    #[test]
    fn jsonc_comment() {
        let out = hl(r#"// comment
{"key": 1}"#);
        assert!(out.contains("tok-comment"));
    }
}
