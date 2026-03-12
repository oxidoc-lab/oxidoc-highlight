use crate::scanner::Scanner;
use crate::token::{Token, TokenKind};

pub struct XmlScanner;

fn at(b: &[u8], i: usize) -> u8 {
    if i < b.len() { b[i] } else { 0 }
}

impl Scanner for XmlScanner {
    fn scan(&self, code: &str) -> Vec<Token> {
        let b = code.as_bytes();
        let mut tokens = Vec::new();
        let mut i = 0;

        while i < b.len() {
            // Comments <!-- -->
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
                tokens.push(Token {
                    kind: TokenKind::Comment,
                    start,
                    end: i.min(b.len()),
                });
                continue;
            }

            // CDATA <![CDATA[...]]>
            if b[i..].starts_with(b"<![CDATA[") {
                let start = i;
                i += 9;
                while i + 2 < b.len() {
                    if b[i] == b']' && b[i + 1] == b']' && b[i + 2] == b'>' {
                        i += 3;
                        break;
                    }
                    i += 1;
                }
                tokens.push(Token {
                    kind: TokenKind::String,
                    start,
                    end: i.min(b.len()),
                });
                continue;
            }

            // Processing instructions <?...?>
            if b[i..].starts_with(b"<?") {
                let start = i;
                i += 2;
                while i + 1 < b.len() {
                    if b[i] == b'?' && b[i + 1] == b'>' {
                        i += 2;
                        break;
                    }
                    i += 1;
                }
                tokens.push(Token {
                    kind: TokenKind::Keyword,
                    start,
                    end: i.min(b.len()),
                });
                continue;
            }

            // Tags
            if b[i] == b'<' {
                let start = i;
                i += 1;
                if at(b, i) == b'/' {
                    i += 1;
                }

                if at(b, i).is_ascii_alphabetic() || at(b, i) == b'_' {
                    while i < b.len()
                        && (b[i].is_ascii_alphanumeric()
                            || b[i] == b'-'
                            || b[i] == b'_'
                            || b[i] == b':'
                            || b[i] == b'.')
                    {
                        i += 1;
                    }
                    tokens.push(Token {
                        kind: TokenKind::Keyword,
                        start,
                        end: i,
                    });

                    // Attributes
                    while i < b.len() && b[i] != b'>' {
                        if b[i] == b' ' || b[i] == b'\t' || b[i] == b'\n' || b[i] == b'\r' {
                            i += 1;
                            continue;
                        }
                        if b[i] == b'/' && at(b, i + 1) == b'>' {
                            tokens.push(Token {
                                kind: TokenKind::Punctuation,
                                start: i,
                                end: i + 2,
                            });
                            i += 2;
                            break;
                        }
                        if b[i].is_ascii_alphabetic() || b[i] == b'_' || b[i] == b':' {
                            let as_ = i;
                            while i < b.len()
                                && (b[i].is_ascii_alphanumeric()
                                    || b[i] == b'-'
                                    || b[i] == b'_'
                                    || b[i] == b':')
                            {
                                i += 1;
                            }
                            tokens.push(Token {
                                kind: TokenKind::Attr,
                                start: as_,
                                end: i,
                            });
                            while i < b.len() && b[i] == b' ' {
                                i += 1;
                            }
                            if at(b, i) == b'=' {
                                tokens.push(Token {
                                    kind: TokenKind::Operator,
                                    start: i,
                                    end: i + 1,
                                });
                                i += 1;
                                while i < b.len() && b[i] == b' ' {
                                    i += 1;
                                }
                                if at(b, i) == b'"' || at(b, i) == b'\'' {
                                    let q = b[i];
                                    let vs = i;
                                    i += 1;
                                    while i < b.len() && b[i] != q {
                                        i += 1;
                                    }
                                    if i < b.len() {
                                        i += 1;
                                    }
                                    tokens.push(Token {
                                        kind: TokenKind::String,
                                        start: vs,
                                        end: i,
                                    });
                                }
                            }
                            continue;
                        }
                        i += 1;
                    }
                    if i < b.len() && b[i] == b'>' {
                        tokens.push(Token {
                            kind: TokenKind::Punctuation,
                            start: i,
                            end: i + 1,
                        });
                        i += 1;
                    }
                    continue;
                }
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
        render(code, &XmlScanner.scan(code))
    }

    #[test]
    fn tag() {
        assert!(hl("<root>").contains("tok-keyword"));
    }
    #[test]
    fn attr() {
        assert!(hl(r#"<div id="x">"#).contains("tok-attr"));
    }
    #[test]
    fn comment() {
        assert!(hl("<!-- hi -->").contains("tok-comment"));
    }
    #[test]
    fn cdata() {
        assert!(hl("<![CDATA[data]]>").contains("tok-string"));
    }
    #[test]
    fn closing() {
        assert!(hl("</div>").contains("tok-keyword"));
    }
}
