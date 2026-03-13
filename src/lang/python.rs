use crate::scanner::*;
use crate::token::{Token, TokenKind};

pub struct PythonScanner;

const KEYWORDS: &[&[u8]] = &[
    b"False",
    b"None",
    b"True",
    b"and",
    b"as",
    b"assert",
    b"async",
    b"await",
    b"break",
    b"class",
    b"continue",
    b"def",
    b"del",
    b"elif",
    b"else",
    b"except",
    b"finally",
    b"for",
    b"from",
    b"global",
    b"if",
    b"import",
    b"in",
    b"is",
    b"lambda",
    b"nonlocal",
    b"not",
    b"or",
    b"pass",
    b"raise",
    b"return",
    b"try",
    b"while",
    b"with",
    b"yield",
];

const BUILTINS: &[&[u8]] = &[
    b"print",
    b"len",
    b"range",
    b"type",
    b"int",
    b"str",
    b"float",
    b"bool",
    b"list",
    b"dict",
    b"set",
    b"tuple",
    b"enumerate",
    b"zip",
    b"map",
    b"filter",
    b"sorted",
    b"reversed",
    b"min",
    b"max",
    b"sum",
    b"abs",
    b"all",
    b"any",
    b"isinstance",
    b"issubclass",
    b"hasattr",
    b"getattr",
    b"setattr",
    b"delattr",
    b"open",
    b"input",
    b"repr",
    b"super",
    b"property",
    b"classmethod",
    b"staticmethod",
    b"iter",
    b"next",
    b"id",
    b"hash",
    b"callable",
    b"vars",
    b"dir",
    b"help",
    b"hex",
    b"oct",
    b"bin",
    b"ord",
    b"chr",
    b"format",
];

const TYPE_NAMES: &[&[u8]] = &[
    b"int",
    b"str",
    b"float",
    b"bool",
    b"list",
    b"dict",
    b"set",
    b"tuple",
    b"bytes",
    b"bytearray",
    b"memoryview",
    b"complex",
    b"frozenset",
    b"Optional",
    b"Union",
    b"List",
    b"Dict",
    b"Set",
    b"Tuple",
    b"Any",
    b"Callable",
    b"Iterator",
    b"Generator",
    b"Coroutine",
    b"Type",
];

fn at(b: &[u8], i: usize) -> u8 {
    if i < b.len() { b[i] } else { 0 }
}

impl Scanner for PythonScanner {
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

            // Comments
            if c == b'#'
                && let Some(end) = scan_hash_comment(b, i)
            {
                tokens.push(Token {
                    kind: TokenKind::Comment,
                    start: i,
                    end,
                });
                prev_kind = Some(TokenKind::Comment);
                i = end;
                continue;
            }

            // Decorators
            if c == b'@' {
                let start = i;
                i += 1;
                if let Some((end, _)) = scan_ident(b, i) {
                    let mut e = end;
                    while at(b, e) == b'.' {
                        if let Some((end2, _)) = scan_ident(b, e + 1) {
                            e = end2;
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token {
                        kind: TokenKind::Attr,
                        start,
                        end: e,
                    });
                    prev_kind = Some(TokenKind::Attr);
                    i = e;
                    continue;
                }
            }

            // Triple-quoted strings (must check before single/double)
            if (c == b'"' || c == b'\'') && at(b, i + 1) == c && at(b, i + 2) == c {
                let end = scan_triple_string(b, i, c);
                tokens.push(Token {
                    kind: TokenKind::String,
                    start: i,
                    end,
                });
                prev_kind = Some(TokenKind::String);
                i = end;
                continue;
            }

            // f-strings: f"...", f'...', f"""...""", f'''...'''
            if c == b'f' || c == b'F' || c == b'b' || c == b'B' || c == b'r' || c == b'R' {
                // Handle prefixed strings: f"", b"", r"", rb"", br"", etc.
                let mut prefix_len = 0;
                let mut j = i;
                while j < b.len()
                    && matches!(b[j], b'f' | b'F' | b'b' | b'B' | b'r' | b'R' | b'u' | b'U')
                {
                    j += 1;
                    prefix_len += 1;
                    if prefix_len > 3 {
                        break;
                    }
                }
                if prefix_len <= 3 && j < b.len() && (b[j] == b'"' || b[j] == b'\'') {
                    let q = b[j];
                    if at(b, j + 1) == q && at(b, j + 2) == q {
                        let end = scan_triple_string(b, j, q);
                        tokens.push(Token {
                            kind: TokenKind::String,
                            start: i,
                            end,
                        });
                        prev_kind = Some(TokenKind::String);
                        i = end;
                        continue;
                    }
                    if let Some(end) = scan_quoted_string(b, j, q) {
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
            }

            // Regular strings
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
            if c == b'\''
                && let Some(end) = scan_single_string(b, i)
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

            // Identifiers
            if let Some((end, ident)) = scan_ident(b, i) {
                let kind = if is_keyword(ident, KEYWORDS) {
                    TokenKind::Keyword
                } else if ident == b"self" || ident == b"cls" {
                    TokenKind::Variable
                } else if (is_keyword(ident, TYPE_NAMES) && !is_function_call(b, end))
                    || is_pascal_case(ident)
                {
                    TokenKind::Type
                } else if is_keyword(ident, BUILTINS) {
                    TokenKind::Builtin
                } else if is_function_call(b, end) {
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

            i += 1;
        }

        tokens
    }
}

fn scan_triple_string(b: &[u8], pos: usize, q: u8) -> usize {
    let mut i = pos + 3;
    while i + 2 < b.len() {
        if b[i] == b'\\' {
            i += 2;
        } else if b[i] == q && b[i + 1] == q && b[i + 2] == q {
            return i + 3;
        } else {
            i += 1;
        }
    }
    b.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::render;

    fn hl(code: &str) -> String {
        let tokens = PythonScanner.scan(code);
        render(code, &tokens)
    }

    #[test]
    fn keywords() {
        let out = hl("def foo():");
        assert!(out.contains("tok-keyword"));
        assert!(out.contains("tok-function"));
    }

    #[test]
    fn decorator() {
        let out = hl("@property");
        assert!(out.contains("tok-attr"));
    }

    #[test]
    fn triple_string() {
        let out = hl(r#""""hello""""#);
        assert!(out.contains("tok-string"));
    }

    #[test]
    fn f_string() {
        let out = hl(r#"f"hello {name}""#);
        assert!(out.contains("tok-string"));
    }

    #[test]
    fn comment() {
        let out = hl("# comment");
        assert!(out.contains("tok-comment"));
    }

    #[test]
    fn self_variable() {
        let out = hl("self.x");
        assert!(out.contains("tok-variable"));
    }

    #[test]
    fn builtin() {
        let out = hl("print(x)");
        assert!(out.contains("tok-builtin"));
    }
}
