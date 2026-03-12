use crate::scanner::*;
use crate::token::{Token, TokenKind};

pub struct YamlScanner;

fn at(b: &[u8], i: usize) -> u8 {
    if i < b.len() { b[i] } else { 0 }
}

impl Scanner for YamlScanner {
    fn scan(&self, code: &str) -> Vec<Token> {
        let b = code.as_bytes();
        let mut tokens = Vec::new();
        let mut i = 0;
        let mut line_start = true;

        while i < b.len() {
            let c = b[i];

            if c == b'\n' {
                line_start = true;
                i += 1;
                continue;
            }
            if c == b' ' || c == b'\t' || c == b'\r' {
                i += 1;
                continue;
            }

            // Document markers
            if line_start && (b[i..].starts_with(b"---") || b[i..].starts_with(b"...")) {
                let start = i;
                i += 3;
                tokens.push(Token { kind: TokenKind::Keyword, start, end: i });
                line_start = false;
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

            // Strings
            if c == b'"' {
                if let Some(end) = scan_double_string(b, i) {
                    tokens.push(Token { kind: TokenKind::String, start: i, end });
                    i = end;
                    line_start = false;
                    continue;
                }
            }
            if c == b'\'' {
                if let Some(end) = scan_single_string(b, i) {
                    tokens.push(Token { kind: TokenKind::String, start: i, end });
                    i = end;
                    line_start = false;
                    continue;
                }
            }

            // Key: value pattern — scan identifier then check for :
            if c.is_ascii_alphanumeric() || c == b'_' || c == b'-' {
                let start = i;
                while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_' || b[i] == b'-' || b[i] == b'.') {
                    i += 1;
                }
                let ident = &b[start..i];
                let after = skip_whitespace_no_newline(b, i);

                if at(b, after) == b':' && (at(b, after + 1) == b' ' || at(b, after + 1) == b'\n' || after + 1 >= b.len()) {
                    tokens.push(Token { kind: TokenKind::Property, start, end: i });
                } else {
                    // Value keywords
                    let kind = match ident {
                        b"true" | b"false" | b"yes" | b"no" | b"on" | b"off" | b"True" | b"False" | b"Yes" | b"No" => TokenKind::Keyword,
                        b"null" | b"Null" | b"~" => TokenKind::Keyword,
                        _ => {
                            // Try number: must start with digit or sign followed by digit
                            if !ident.is_empty()
                                && (ident[0].is_ascii_digit() || ((ident[0] == b'-' || ident[0] == b'+') && ident.len() > 1 && ident[1].is_ascii_digit()))
                                && ident[1..].iter().all(|&c| c.is_ascii_digit() || c == b'.' || c == b'e' || c == b'E' || c == b'_' || c == b'+' || c == b'-')
                            {
                                TokenKind::Number
                            } else {
                                TokenKind::Plain
                            }
                        }
                    };
                    tokens.push(Token { kind, start, end: i });
                }
                line_start = false;
                continue;
            }

            // Punctuation: : - [ ] { } ,
            if matches!(c, b':' | b'-' | b'[' | b']' | b'{' | b'}' | b',') {
                tokens.push(Token { kind: TokenKind::Punctuation, start: i, end: i + 1 });
                i += 1;
                line_start = false;
                continue;
            }

            // Anchors & aliases
            if c == b'&' || c == b'*' {
                let start = i;
                i += 1;
                while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_' || b[i] == b'-') {
                    i += 1;
                }
                tokens.push(Token { kind: TokenKind::Variable, start, end: i });
                line_start = false;
                continue;
            }

            // Tags !tag
            if c == b'!' {
                let start = i;
                i += 1;
                while i < b.len() && !b[i].is_ascii_whitespace() {
                    i += 1;
                }
                tokens.push(Token { kind: TokenKind::Attr, start, end: i });
                line_start = false;
                continue;
            }

            line_start = false;
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
        let tokens = YamlScanner.scan(code);
        render(code, &tokens)
    }

    #[test]
    fn key_value() {
        let out = hl("name: test");
        assert!(out.contains("tok-property"));
    }

    #[test]
    fn boolean() {
        let out = hl("enabled: true");
        assert!(out.contains("tok-keyword"));
    }

    #[test]
    fn comment() {
        let out = hl("# comment");
        assert!(out.contains("tok-comment"));
    }

    #[test]
    fn document_marker() {
        let out = hl("---");
        assert!(out.contains("tok-keyword"));
    }

    #[test]
    fn string() {
        let out = hl(r#"name: "hello""#);
        assert!(out.contains("tok-string"));
    }
}
