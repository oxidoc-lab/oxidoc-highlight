use crate::scanner::*;
use crate::token::{Token, TokenKind};

pub struct MarkdownScanner;

fn at(b: &[u8], i: usize) -> u8 {
    if i < b.len() { b[i] } else { 0 }
}

impl Scanner for MarkdownScanner {
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

            if line_start {
                // Skip leading spaces
                if c == b' ' || c == b'\t' {
                    i += 1;
                    continue;
                }
                line_start = false;

                // Headings: # ## ### etc
                if c == b'#' {
                    let start = i;
                    while i < b.len() && b[i] == b'#' {
                        i += 1;
                    }
                    // Rest of line is heading content
                    while i < b.len() && b[i] != b'\n' {
                        i += 1;
                    }
                    tokens.push(Token {
                        kind: TokenKind::Keyword,
                        start,
                        end: i,
                    });
                    continue;
                }

                // Code fences ```
                if c == b'`' && at(b, i + 1) == b'`' && at(b, i + 2) == b'`' {
                    let start = i;
                    i += 3;
                    // Skip language tag
                    while i < b.len() && b[i] != b'\n' {
                        i += 1;
                    }
                    if i < b.len() {
                        i += 1;
                    }
                    // Find closing ```
                    while i < b.len() {
                        if b[i] == b'`' && at(b, i + 1) == b'`' && at(b, i + 2) == b'`' {
                            i += 3;
                            while i < b.len() && b[i] != b'\n' {
                                i += 1;
                            }
                            break;
                        }
                        i += 1;
                    }
                    tokens.push(Token {
                        kind: TokenKind::String,
                        start,
                        end: i,
                    });
                    continue;
                }

                // Blockquotes >
                if c == b'>' {
                    tokens.push(Token {
                        kind: TokenKind::Keyword,
                        start: i,
                        end: i + 1,
                    });
                    i += 1;
                    continue;
                }

                // Horizontal rules --- *** ___
                if matches!(c, b'-' | b'*' | b'_') && at(b, i + 1) == c && at(b, i + 2) == c {
                    let start = i;
                    while i < b.len() && (b[i] == c || b[i] == b' ') {
                        i += 1;
                    }
                    if at(b, i) == b'\n' || i >= b.len() {
                        tokens.push(Token {
                            kind: TokenKind::Keyword,
                            start,
                            end: i,
                        });
                        continue;
                    }
                    i = start; // wasn't a rule
                }

                // List markers: - * + or 1. 2. etc
                if matches!(c, b'-' | b'*' | b'+') && at(b, i + 1) == b' ' {
                    tokens.push(Token {
                        kind: TokenKind::Keyword,
                        start: i,
                        end: i + 1,
                    });
                    i += 1;
                    continue;
                }
            }

            // Inline code `...`
            if c == b'`' {
                let start = i;
                i += 1;
                while i < b.len() && b[i] != b'`' && b[i] != b'\n' {
                    i += 1;
                }
                if i < b.len() && b[i] == b'`' {
                    i += 1;
                }
                tokens.push(Token {
                    kind: TokenKind::String,
                    start,
                    end: i,
                });
                continue;
            }

            // Bold **...**  __...__
            if (c == b'*' && at(b, i + 1) == b'*') || (c == b'_' && at(b, i + 1) == b'_') {
                let delim = c;
                let start = i;
                i += 2;
                while i + 1 < b.len() && !(b[i] == delim && b[i + 1] == delim) {
                    if b[i] == b'\n' {
                        break;
                    }
                    i += 1;
                }
                if i + 1 < b.len() && b[i] == delim && b[i + 1] == delim {
                    i += 2;
                }
                tokens.push(Token {
                    kind: TokenKind::Keyword,
                    start,
                    end: i,
                });
                continue;
            }

            // Italic *...* or _..._
            if (c == b'*' || c == b'_') && at(b, i + 1) != b' ' {
                let delim = c;
                let start = i;
                i += 1;
                while i < b.len() && b[i] != delim && b[i] != b'\n' {
                    i += 1;
                }
                if i < b.len() && b[i] == delim {
                    i += 1;
                }
                tokens.push(Token {
                    kind: TokenKind::Keyword,
                    start,
                    end: i,
                });
                continue;
            }

            // Links [text](url)
            if c == b'[' {
                let start = i;
                i += 1;
                while i < b.len() && b[i] != b']' && b[i] != b'\n' {
                    i += 1;
                }
                if at(b, i) == b']' && at(b, i + 1) == b'(' {
                    i += 2;
                    while i < b.len() && b[i] != b')' && b[i] != b'\n' {
                        i += 1;
                    }
                    if at(b, i) == b')' {
                        i += 1;
                    }
                    tokens.push(Token {
                        kind: TokenKind::String,
                        start,
                        end: i,
                    });
                    continue;
                }
            }

            // HTML tags within markdown
            if c == b'<' && (at(b, i + 1).is_ascii_alphabetic() || at(b, i + 1) == b'/') {
                let start = i;
                while i < b.len() && b[i] != b'>' {
                    i += 1;
                }
                if i < b.len() {
                    i += 1;
                }
                tokens.push(Token {
                    kind: TokenKind::Attr,
                    start,
                    end: i,
                });
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
        render(code, &MarkdownScanner.scan(code))
    }

    #[test]
    fn heading() {
        assert!(hl("# Hello").contains("tok-keyword"));
    }
    #[test]
    fn code_span() {
        assert!(hl("`code`").contains("tok-string"));
    }
    #[test]
    fn bold() {
        assert!(hl("**bold**").contains("tok-keyword"));
    }
    #[test]
    fn link() {
        assert!(hl("[text](url)").contains("tok-string"));
    }
    #[test]
    fn code_fence() {
        assert!(hl("```rust\nlet x = 1;\n```").contains("tok-string"));
    }
}
