use crate::scanner::*;
use crate::token::{Token, TokenKind};

pub struct GoScanner;

const KEYWORDS: &[&[u8]] = &[
    b"break",
    b"case",
    b"chan",
    b"const",
    b"continue",
    b"default",
    b"defer",
    b"else",
    b"fallthrough",
    b"for",
    b"func",
    b"go",
    b"goto",
    b"if",
    b"import",
    b"interface",
    b"map",
    b"package",
    b"range",
    b"return",
    b"select",
    b"struct",
    b"switch",
    b"type",
    b"var",
    b"true",
    b"false",
    b"nil",
];

const TYPES: &[&[u8]] = &[
    b"bool",
    b"byte",
    b"complex64",
    b"complex128",
    b"error",
    b"float32",
    b"float64",
    b"int",
    b"int8",
    b"int16",
    b"int32",
    b"int64",
    b"rune",
    b"string",
    b"uint",
    b"uint8",
    b"uint16",
    b"uint32",
    b"uint64",
    b"uintptr",
    b"any",
];

const BUILTINS: &[&[u8]] = &[
    b"append", b"cap", b"close", b"complex", b"copy", b"delete", b"imag", b"len", b"make", b"new",
    b"panic", b"print", b"println", b"real", b"recover",
];

impl Scanner for GoScanner {
    fn scan(&self, code: &str) -> Vec<Token> {
        let b = code.as_bytes();
        let mut tokens = Vec::new();
        let mut i = 0;
        let mut prev_kind: Option<TokenKind> = None;

        while i < b.len() {
            let c = b[i];
            if c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' {
                i += 1;
                continue;
            }

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
            if c == b'"' {
                if let Some(end) = scan_double_string(b, i) {
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
            if c == b'\'' {
                if let Some(end) = scan_single_string(b, i) {
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
            if c == b'`' {
                if let Some(end) = scan_backtick_string(b, i) {
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

            if let Some((end, ident)) = scan_ident(b, i) {
                let kind = if is_keyword(ident, KEYWORDS) {
                    TokenKind::Keyword
                } else if is_keyword(ident, TYPES) {
                    TokenKind::Type
                } else if is_keyword(ident, BUILTINS) {
                    TokenKind::Builtin
                } else if is_pascal_case(ident) {
                    TokenKind::Type
                } else if is_function_call(b, end) && !was_keyword(prev_kind) {
                    TokenKind::Function
                } else if matches!(prev_kind, Some(TokenKind::Punctuation))
                    && i >= 1
                    && b[i - 1] == b'.'
                {
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
        render(code, &GoScanner.scan(code))
    }

    #[test]
    fn func() {
        assert!(hl("func main() {}").contains("tok-keyword"));
    }
    #[test]
    fn types() {
        assert!(hl("var x int").contains("tok-type"));
    }
    #[test]
    fn builtin() {
        assert!(hl("len(s)").contains("tok-builtin"));
    }
    #[test]
    fn comment() {
        assert!(hl("// comment").contains("tok-comment"));
    }
    #[test]
    fn string() {
        assert!(hl(r#""hello""#).contains("tok-string"));
    }
}
