use crate::token::{Token, TokenKind};
use crate::scanner::Scanner;

pub struct HtmlScanner;

fn at(b: &[u8], i: usize) -> u8 {
    if i < b.len() { b[i] } else { 0 }
}

impl Scanner for HtmlScanner {
    fn scan(&self, code: &str) -> Vec<Token> {
        let b = code.as_bytes();
        let mut tokens = Vec::new();
        let mut i = 0;

        while i < b.len() {
            // HTML comment <!-- ... -->
            if b[i..].starts_with(b"<!--") {
                let start = i;
                i += 4;
                while i + 2 < b.len() {
                    if b[i] == b'-' && b[i + 1] == b'-' && b[i + 2] == b'>' {
                        i += 3;
                        break;
                    }
                    i += 1;
                }
                // Terminated or unterminated — both are comment tokens
                tokens.push(Token { kind: TokenKind::Comment, start, end: i.min(b.len()) });
                continue;
            }

            // Tags: < ... >
            if b[i] == b'<' {
                let start = i;
                i += 1;
                let is_closing = at(b, i) == b'/';
                if is_closing { i += 1; }

                // Tag name
                if at(b, i).is_ascii_alphabetic() || at(b, i) == b'!' || at(b, i) == b'_' {
                    while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'-' || b[i] == b'_' || b[i] == b':' || b[i] == b'.') {
                        i += 1;
                    }
                    // The whole <tagname or </tagname as keyword
                    tokens.push(Token { kind: TokenKind::Keyword, start, end: i });

                    // Attributes
                    while i < b.len() && b[i] != b'>' {
                        if b[i] == b' ' || b[i] == b'\t' || b[i] == b'\n' || b[i] == b'\r' {
                            i += 1;
                            continue;
                        }
                        // Self-closing />
                        if b[i] == b'/' && at(b, i + 1) == b'>' {
                            tokens.push(Token { kind: TokenKind::Punctuation, start: i, end: i + 2 });
                            i += 2;
                            break;
                        }
                        // Attribute name
                        if b[i].is_ascii_alphabetic() || b[i] == b'_' || b[i] == b'-' || b[i] == b':' || b[i] == b'@' {
                            let attr_start = i;
                            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'-' || b[i] == b'_' || b[i] == b':' || b[i] == b'@' || b[i] == b'.') {
                                i += 1;
                            }
                            tokens.push(Token { kind: TokenKind::Attr, start: attr_start, end: i });

                            // =
                            while i < b.len() && b[i] == b' ' { i += 1; }
                            if at(b, i) == b'=' {
                                tokens.push(Token { kind: TokenKind::Operator, start: i, end: i + 1 });
                                i += 1;
                                while i < b.len() && b[i] == b' ' { i += 1; }
                                // Attribute value
                                if at(b, i) == b'"' || at(b, i) == b'\'' {
                                    let q = b[i];
                                    let vs = i;
                                    i += 1;
                                    while i < b.len() && b[i] != q { i += 1; }
                                    if i < b.len() { i += 1; }
                                    tokens.push(Token { kind: TokenKind::String, start: vs, end: i });
                                } else {
                                    // Unquoted value
                                    let vs = i;
                                    while i < b.len() && b[i] != b' ' && b[i] != b'>' && b[i] != b'/' {
                                        i += 1;
                                    }
                                    if i > vs {
                                        tokens.push(Token { kind: TokenKind::String, start: vs, end: i });
                                    }
                                }
                            }
                            continue;
                        }
                        i += 1;
                    }
                    // Closing >
                    if i < b.len() && b[i] == b'>' {
                        tokens.push(Token { kind: TokenKind::Punctuation, start: i, end: i + 1 });
                        i += 1;
                    }
                    continue;
                }
                // Not a valid tag, treat < as plain
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
        let tokens = HtmlScanner.scan(code);
        render(code, &tokens)
    }

    #[test]
    fn tag() {
        let out = hl("<div>");
        assert!(out.contains("tok-keyword"));
    }

    #[test]
    fn attribute() {
        let out = hl(r#"<div class="foo">"#);
        assert!(out.contains("tok-attr"));
        assert!(out.contains("tok-string"));
    }

    #[test]
    fn comment() {
        let out = hl("<!-- comment -->");
        assert!(out.contains("tok-comment"));
    }

    #[test]
    fn self_closing() {
        let out = hl("<br />");
        assert!(out.contains("tok-keyword"));
    }

    #[test]
    fn closing_tag() {
        let out = hl("</div>");
        assert!(out.contains("tok-keyword"));
    }
}
