use crate::scanner::*;
use crate::token::{Token, TokenKind};

pub struct CssScanner;

fn at(b: &[u8], i: usize) -> u8 {
    if i < b.len() { b[i] } else { 0 }
}

impl Scanner for CssScanner {
    fn scan(&self, code: &str) -> Vec<Token> {
        let b = code.as_bytes();
        let mut tokens = Vec::new();
        let mut i = 0;
        let mut in_block = false; // inside { ... }

        while i < b.len() {
            let c = b[i];

            if c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' {
                i += 1;
                continue;
            }

            // Comments
            if let Some(end) = scan_block_comment(b, i) {
                tokens.push(Token {
                    kind: TokenKind::Comment,
                    start: i,
                    end,
                });
                i = end;
                continue;
            }

            // Strings
            if c == b'"' {
                if let Some(end) = scan_double_string(b, i) {
                    tokens.push(Token {
                        kind: TokenKind::String,
                        start: i,
                        end,
                    });
                    i = end;
                    continue;
                }
            }
            if c == b'\'' {
                if let Some(end) = scan_single_string(b, i) {
                    tokens.push(Token {
                        kind: TokenKind::String,
                        start: i,
                        end,
                    });
                    i = end;
                    continue;
                }
            }

            // @ rules
            if c == b'@' {
                let start = i;
                i += 1;
                while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'-') {
                    i += 1;
                }
                tokens.push(Token {
                    kind: TokenKind::Keyword,
                    start,
                    end: i,
                });
                continue;
            }

            // Track block depth
            if c == b'{' {
                in_block = true;
                tokens.push(Token {
                    kind: TokenKind::Punctuation,
                    start: i,
                    end: i + 1,
                });
                i += 1;
                continue;
            }
            if c == b'}' {
                in_block = false;
                tokens.push(Token {
                    kind: TokenKind::Punctuation,
                    start: i,
                    end: i + 1,
                });
                i += 1;
                continue;
            }

            // Inside block: property: value;
            if in_block {
                // Property name (before colon)
                if c.is_ascii_alphabetic() || c == b'-' || c == b'_' {
                    let start = i;
                    while i < b.len()
                        && (b[i].is_ascii_alphanumeric() || b[i] == b'-' || b[i] == b'_')
                    {
                        i += 1;
                    }
                    let after = skip_whitespace_no_newline(b, i);
                    if at(b, after) == b':' {
                        tokens.push(Token {
                            kind: TokenKind::Property,
                            start,
                            end: i,
                        });
                    } else {
                        tokens.push(Token {
                            kind: TokenKind::Plain,
                            start,
                            end: i,
                        });
                    }
                    continue;
                }

                if c == b':' {
                    tokens.push(Token {
                        kind: TokenKind::Punctuation,
                        start: i,
                        end: i + 1,
                    });
                    i += 1;
                    continue;
                }

                // Numbers with units
                if c.is_ascii_digit() || (c == b'.' && at(b, i + 1).is_ascii_digit()) {
                    if let Some(end) = scan_number(b, i) {
                        let mut e = end;
                        // Consume CSS units
                        while e < b.len() && (b[e].is_ascii_alphabetic() || b[e] == b'%') {
                            e += 1;
                        }
                        tokens.push(Token {
                            kind: TokenKind::Number,
                            start: i,
                            end: e,
                        });
                        i = e;
                        continue;
                    }
                }

                // Hash colors
                if c == b'#' {
                    let start = i;
                    i += 1;
                    while i < b.len() && b[i].is_ascii_hexdigit() {
                        i += 1;
                    }
                    tokens.push(Token {
                        kind: TokenKind::Number,
                        start,
                        end: i,
                    });
                    continue;
                }

                // Function calls in values (rgb, calc, etc)
                if c.is_ascii_alphabetic() || c == b'-' {
                    let start = i;
                    while i < b.len()
                        && (b[i].is_ascii_alphanumeric() || b[i] == b'-' || b[i] == b'_')
                    {
                        i += 1;
                    }
                    if at(b, i) == b'(' {
                        tokens.push(Token {
                            kind: TokenKind::Function,
                            start,
                            end: i,
                        });
                    } else {
                        tokens.push(Token {
                            kind: TokenKind::Plain,
                            start,
                            end: i,
                        });
                    }
                    continue;
                }
            } else {
                // Selectors (outside blocks)
                if c == b'.' || c == b'#' || c == b'*' || c == b':' || c == b'[' {
                    let start = i;
                    // Just consume selector chars
                    if c == b'.' || c == b'#' {
                        i += 1;
                        while i < b.len()
                            && (b[i].is_ascii_alphanumeric() || b[i] == b'-' || b[i] == b'_')
                        {
                            i += 1;
                        }
                    } else if c == b':' {
                        i += 1;
                        if at(b, i) == b':' {
                            i += 1;
                        }
                        while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'-') {
                            i += 1;
                        }
                    } else {
                        i += 1;
                    }
                    tokens.push(Token {
                        kind: TokenKind::Keyword,
                        start,
                        end: i,
                    });
                    continue;
                }

                // Tag selectors
                if c.is_ascii_alphabetic() {
                    let start = i;
                    while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'-') {
                        i += 1;
                    }
                    tokens.push(Token {
                        kind: TokenKind::Keyword,
                        start,
                        end: i,
                    });
                    continue;
                }
            }

            // Punctuation
            if let Some(end) = scan_punctuation(b, i) {
                tokens.push(Token {
                    kind: TokenKind::Punctuation,
                    start: i,
                    end,
                });
                i = end;
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
        let tokens = CssScanner.scan(code);
        render(code, &tokens)
    }

    #[test]
    fn selector_and_property() {
        let out = hl("body { color: red; }");
        assert!(out.contains("tok-keyword")); // selector
        assert!(out.contains("tok-property")); // color
    }

    #[test]
    fn at_rule() {
        let out = hl("@media screen");
        assert!(out.contains("tok-keyword"));
    }

    #[test]
    fn comment() {
        let out = hl("/* comment */");
        assert!(out.contains("tok-comment"));
    }

    #[test]
    fn number_with_unit() {
        let out = hl("body { width: 100px; }");
        assert!(out.contains("tok-number"));
    }

    #[test]
    fn class_selector() {
        let out = hl(".foo { }");
        assert!(out.contains("tok-keyword"));
    }
}
