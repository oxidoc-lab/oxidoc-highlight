use crate::scanner::*;
use crate::token::{Token, TokenKind};

pub struct TomlScanner;

fn at(b: &[u8], i: usize) -> u8 {
    if i < b.len() { b[i] } else { 0 }
}

impl Scanner for TomlScanner {
    fn scan(&self, code: &str) -> Vec<Token> {
        let b = code.as_bytes();
        let mut tokens = Vec::new();
        let mut i = 0;

        while i < b.len() {
            let c = b[i];

            if c == b' ' || c == b'\t' || c == b'\r' {
                i += 1;
                continue;
            }

            // Newlines — reset line context
            if c == b'\n' {
                i += 1;
                continue;
            }

            // Comments
            if c == b'#' {
                if let Some(end) = scan_hash_comment(b, i) {
                    tokens.push(Token { kind: TokenKind::Comment, start: i, end });
                    i = end;
                    continue;
                }
            }

            // Section headers [section] or [[array]]
            if c == b'[' {
                let start = i;
                let double = at(b, i + 1) == b'[';
                i += if double { 2 } else { 1 };
                while i < b.len() && b[i] != b']' && b[i] != b'\n' {
                    i += 1;
                }
                if at(b, i) == b']' { i += 1; }
                if double && at(b, i) == b']' { i += 1; }
                tokens.push(Token { kind: TokenKind::Keyword, start, end: i });
                continue;
            }

            // Triple-quoted strings
            if c == b'"' && at(b, i + 1) == b'"' && at(b, i + 2) == b'"' {
                let start = i;
                i += 3;
                while i + 2 < b.len() {
                    if b[i] == b'"' && b[i + 1] == b'"' && b[i + 2] == b'"' {
                        i += 3;
                        break;
                    }
                    i += 1;
                }
                tokens.push(Token { kind: TokenKind::String, start, end: i.min(b.len()) });
                continue;
            }
            if c == b'\'' && at(b, i + 1) == b'\'' && at(b, i + 2) == b'\'' {
                let start = i;
                i += 3;
                while i + 2 < b.len() {
                    if b[i] == b'\'' && b[i + 1] == b'\'' && b[i + 2] == b'\'' {
                        i += 3;
                        break;
                    }
                    i += 1;
                }
                tokens.push(Token { kind: TokenKind::String, start, end: i.min(b.len()) });
                continue;
            }

            // Strings
            if c == b'"' {
                if let Some(end) = scan_double_string(b, i) {
                    tokens.push(Token { kind: TokenKind::String, start: i, end });
                    i = end;
                    continue;
                }
            }
            if c == b'\'' {
                if let Some(end) = scan_single_string(b, i) {
                    tokens.push(Token { kind: TokenKind::String, start: i, end });
                    i = end;
                    continue;
                }
            }

            // Numbers
            if c.is_ascii_digit() || (c == b'+' || c == b'-') && at(b, i + 1).is_ascii_digit() {
                let start = i;
                if c == b'+' || c == b'-' { i += 1; }
                if let Some(end) = scan_number(b, i) {
                    tokens.push(Token { kind: TokenKind::Number, start, end });
                    i = end;
                    continue;
                }
                i = start;
            }

            // Date-time (basic detection: digit-digit-digit-digit-)
            // Handled as numbers already mostly

            // Identifiers (keys, or boolean/date values)
            if let Some((end, ident)) = scan_ident(b, i) {
                let kind = match ident {
                    b"true" | b"false" => TokenKind::Keyword,
                    b"inf" | b"nan" => TokenKind::Number,
                    _ => {
                        // Key if followed by =
                        let after = skip_whitespace_no_newline(b, end);
                        if at(b, after) == b'=' {
                            TokenKind::Property
                        } else {
                            TokenKind::Plain
                        }
                    }
                };
                tokens.push(Token { kind, start: i, end });
                i = end;
                continue;
            }

            // = operator
            if c == b'=' {
                tokens.push(Token { kind: TokenKind::Operator, start: i, end: i + 1 });
                i += 1;
                continue;
            }

            // Punctuation
            if matches!(c, b'{' | b'}' | b',' | b'.' | b']') {
                tokens.push(Token { kind: TokenKind::Punctuation, start: i, end: i + 1 });
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
        let tokens = TomlScanner.scan(code);
        render(code, &tokens)
    }

    #[test]
    fn section() {
        let out = hl("[package]");
        assert!(out.contains("tok-keyword"));
    }

    #[test]
    fn key_value() {
        let out = hl(r#"name = "test""#);
        assert!(out.contains("tok-property"));
        assert!(out.contains("tok-string"));
    }

    #[test]
    fn boolean() {
        let out = hl("enabled = true");
        assert!(out.contains("tok-keyword"));
    }

    #[test]
    fn comment() {
        let out = hl("# comment");
        assert!(out.contains("tok-comment"));
    }

    #[test]
    fn number() {
        let out = hl("port = 8080");
        assert!(out.contains("tok-number"));
    }
}
