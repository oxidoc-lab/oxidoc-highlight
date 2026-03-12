use crate::scanner::*;
use crate::token::{Token, TokenKind};

pub struct PhpScanner;

const KEYWORDS: &[&[u8]] = &[
    b"abstract",
    b"and",
    b"as",
    b"break",
    b"callable",
    b"case",
    b"catch",
    b"class",
    b"clone",
    b"const",
    b"continue",
    b"declare",
    b"default",
    b"do",
    b"echo",
    b"else",
    b"elseif",
    b"empty",
    b"enddeclare",
    b"endfor",
    b"endforeach",
    b"endif",
    b"endswitch",
    b"endwhile",
    b"enum",
    b"extends",
    b"final",
    b"finally",
    b"fn",
    b"for",
    b"foreach",
    b"function",
    b"global",
    b"goto",
    b"if",
    b"implements",
    b"include",
    b"include_once",
    b"instanceof",
    b"insteadof",
    b"interface",
    b"isset",
    b"list",
    b"match",
    b"namespace",
    b"new",
    b"or",
    b"print",
    b"private",
    b"protected",
    b"public",
    b"readonly",
    b"require",
    b"require_once",
    b"return",
    b"static",
    b"switch",
    b"throw",
    b"trait",
    b"try",
    b"unset",
    b"use",
    b"var",
    b"while",
    b"xor",
    b"yield",
    b"from",
    b"true",
    b"false",
    b"null",
    b"TRUE",
    b"FALSE",
    b"NULL",
];

const TYPE_KEYWORDS: &[&[u8]] = &[
    b"array",
    b"bool",
    b"boolean",
    b"float",
    b"double",
    b"int",
    b"integer",
    b"object",
    b"string",
    b"void",
    b"mixed",
    b"never",
    b"null",
    b"iterable",
    b"self",
    b"parent",
];

const BUILTINS: &[&[u8]] = &[
    b"array_map",
    b"array_filter",
    b"array_push",
    b"array_pop",
    b"array_merge",
    b"array_keys",
    b"array_values",
    b"array_slice",
    b"array_splice",
    b"array_shift",
    b"array_unshift",
    b"array_reverse",
    b"array_search",
    b"array_unique",
    b"count",
    b"sizeof",
    b"strlen",
    b"substr",
    b"strpos",
    b"str_replace",
    b"str_contains",
    b"str_starts_with",
    b"str_ends_with",
    b"implode",
    b"explode",
    b"trim",
    b"ltrim",
    b"rtrim",
    b"strtolower",
    b"strtoupper",
    b"ucfirst",
    b"lcfirst",
    b"sprintf",
    b"printf",
    b"var_dump",
    b"print_r",
    b"var_export",
    b"isset",
    b"unset",
    b"empty",
    b"is_null",
    b"is_array",
    b"is_string",
    b"is_int",
    b"is_float",
    b"is_bool",
    b"is_numeric",
    b"is_callable",
    b"intval",
    b"floatval",
    b"strval",
    b"boolval",
    b"json_encode",
    b"json_decode",
    b"file_get_contents",
    b"file_put_contents",
    b"file_exists",
    b"preg_match",
    b"preg_replace",
    b"preg_split",
    b"in_array",
    b"array_key_exists",
    b"compact",
    b"extract",
    b"date",
    b"time",
    b"mktime",
    b"strtotime",
    b"die",
    b"exit",
    b"header",
    b"setcookie",
];

fn at(b: &[u8], i: usize) -> u8 {
    if i < b.len() { b[i] } else { 0 }
}

impl Scanner for PhpScanner {
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

            // PHP open/close tags
            if b[i..].starts_with(b"<?php") {
                tokens.push(Token {
                    kind: TokenKind::Keyword,
                    start: i,
                    end: i + 5,
                });
                prev_kind = Some(TokenKind::Keyword);
                i += 5;
                continue;
            }
            if b[i..].starts_with(b"<?=") {
                tokens.push(Token {
                    kind: TokenKind::Keyword,
                    start: i,
                    end: i + 3,
                });
                prev_kind = Some(TokenKind::Keyword);
                i += 3;
                continue;
            }
            if b[i..].starts_with(b"<?") {
                tokens.push(Token {
                    kind: TokenKind::Keyword,
                    start: i,
                    end: i + 2,
                });
                prev_kind = Some(TokenKind::Keyword);
                i += 2;
                continue;
            }
            if b[i..].starts_with(b"?>") {
                tokens.push(Token {
                    kind: TokenKind::Keyword,
                    start: i,
                    end: i + 2,
                });
                prev_kind = Some(TokenKind::Keyword);
                i += 2;
                continue;
            }

            // Comments: //, #, /* */
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
            if c == b'#' && at(b, i + 1) != b'[' {
                // # line comment (but not #[attribute])
                if let Some(end) = scan_hash_comment(b, i) {
                    tokens.push(Token {
                        kind: TokenKind::Comment,
                        start: i,
                        end,
                    });
                    prev_kind = Some(TokenKind::Comment);
                    i = end;
                    continue;
                }
            }

            // Attributes #[...]
            if c == b'#' && at(b, i + 1) == b'[' {
                let start = i;
                i += 2;
                let mut depth = 1u32;
                while i < b.len() && depth > 0 {
                    if b[i] == b'[' {
                        depth += 1;
                    }
                    if b[i] == b']' {
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

            // Heredoc/nowdoc
            if b[i..].starts_with(b"<<<") {
                if let Some(end) = scan_php_heredoc(b, i) {
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

            // Variables: $name, $$name
            if c == b'$' {
                let start = i;
                i += 1;
                if at(b, i) == b'$' {
                    i += 1;
                } // $$var
                if at(b, i).is_ascii_alphabetic() || at(b, i) == b'_' {
                    while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                        i += 1;
                    }
                    let var_name = &b[start..i];
                    let kind = if var_name == b"$this" {
                        TokenKind::Variable
                    } else {
                        TokenKind::Variable
                    };
                    tokens.push(Token {
                        kind,
                        start,
                        end: i,
                    });
                    prev_kind = Some(kind);
                    continue;
                }
                // Lone $
                i = start + 1;
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
                } else if is_keyword(ident, TYPE_KEYWORDS) {
                    TokenKind::Type
                } else if is_keyword(ident, BUILTINS) && is_function_call(b, end) {
                    TokenKind::Builtin
                } else if is_pascal_case(ident) {
                    TokenKind::Type
                } else if is_function_call(b, end) && !was_keyword(prev_kind) {
                    TokenKind::Function
                } else if matches!(prev_kind, Some(TokenKind::Operator))
                    && i >= 2
                    && b[i - 1] == b'>'
                    && b[i - 2] == b'-'
                {
                    // After ->
                    if is_function_call(b, end) {
                        TokenKind::Function
                    } else {
                        TokenKind::Property
                    }
                } else if matches!(prev_kind, Some(TokenKind::Operator))
                    && i >= 2
                    && b[i - 1] == b':'
                    && b[i - 2] == b':'
                {
                    // After ::
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

            // -> and :: operators
            if c == b'-' && at(b, i + 1) == b'>' {
                tokens.push(Token {
                    kind: TokenKind::Operator,
                    start: i,
                    end: i + 2,
                });
                prev_kind = Some(TokenKind::Operator);
                i += 2;
                continue;
            }
            if c == b':' && at(b, i + 1) == b':' {
                tokens.push(Token {
                    kind: TokenKind::Operator,
                    start: i,
                    end: i + 2,
                });
                prev_kind = Some(TokenKind::Operator);
                i += 2;
                continue;
            }
            // => (array key-value)
            if c == b'=' && at(b, i + 1) == b'>' {
                tokens.push(Token {
                    kind: TokenKind::Operator,
                    start: i,
                    end: i + 2,
                });
                prev_kind = Some(TokenKind::Operator);
                i += 2;
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

/// Scan PHP heredoc/nowdoc: <<<EOT ... EOT or <<<'EOT' ... EOT
fn scan_php_heredoc(b: &[u8], pos: usize) -> Option<usize> {
    let mut i = pos + 3;
    // Skip whitespace
    while i < b.len() && b[i] == b' ' {
        i += 1;
    }
    // Nowdoc uses quotes
    let is_nowdoc = at(b, i) == b'\'';
    if is_nowdoc || at(b, i) == b'"' {
        i += 1;
    }
    let delim_start = i;
    while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
        i += 1;
    }
    if i == delim_start {
        return None;
    }
    let delim = &b[delim_start..i];
    if is_nowdoc || at(b, i) == b'"' {
        i += 1;
    }
    // Skip to next line
    while i < b.len() && b[i] != b'\n' {
        i += 1;
    }
    if i < b.len() {
        i += 1;
    }

    // Find closing delimiter on its own line
    while i < b.len() {
        let mut ws = i;
        while ws < b.len() && (b[ws] == b' ' || b[ws] == b'\t') {
            ws += 1;
        }
        if b[ws..].starts_with(delim) {
            let after = ws + delim.len();
            // Must be followed by ; or newline or EOF
            if after >= b.len() || b[after] == b'\n' || b[after] == b';' || b[after] == b'\r' {
                return Some(after);
            }
        }
        while i < b.len() && b[i] != b'\n' {
            i += 1;
        }
        if i < b.len() {
            i += 1;
        }
    }
    Some(b.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::render;

    fn hl(code: &str) -> String {
        render(code, &PhpScanner.scan(code))
    }

    #[test]
    fn php_tags() {
        let out = hl("<?php echo 'hello'; ?>");
        assert!(out.contains("tok-keyword")); // <?php, echo, ?>
    }

    #[test]
    fn variable() {
        let out = hl("$name = 'test';");
        assert!(out.contains("tok-variable"));
    }

    #[test]
    fn this_variable() {
        let out = hl("$this->name");
        assert!(out.contains("tok-variable")); // $this
        assert!(out.contains("tok-property")); // name
    }

    #[test]
    fn function_call() {
        let out = hl("strlen($str)");
        assert!(out.contains("tok-builtin")); // strlen
    }

    #[test]
    fn class() {
        let out = hl("class UserController extends Controller {}");
        assert!(out.contains("tok-keyword")); // class, extends
        assert!(out.contains("tok-type")); // UserController, Controller
    }

    #[test]
    fn arrow_operator() {
        let out = hl("$obj->method()");
        assert!(out.contains("tok-variable")); // $obj
        assert!(out.contains("tok-function")); // method
    }

    #[test]
    fn static_call() {
        let out = hl("User::find(1)");
        assert!(out.contains("tok-type")); // User
        assert!(out.contains("tok-function")); // find
    }

    #[test]
    fn comment_styles() {
        assert!(hl("// comment").contains("tok-comment"));
        assert!(hl("# comment").contains("tok-comment"));
        assert!(hl("/* block */").contains("tok-comment"));
    }

    #[test]
    fn attribute() {
        let out = hl("#[Route('/api')]");
        assert!(out.contains("tok-attr"));
    }

    #[test]
    fn string() {
        assert!(hl(r#""hello""#).contains("tok-string"));
        assert!(hl("'hello'").contains("tok-string"));
    }

    #[test]
    fn heredoc() {
        let out = hl("<<<EOT\nhello world\nEOT");
        assert!(out.contains("tok-string"));
    }

    #[test]
    fn array_arrow() {
        let out = hl("['key' => 'value']");
        assert!(out.contains("tok-operator")); // =>
        assert!(out.contains("tok-string")); // 'key', 'value'
    }

    #[test]
    fn type_hints() {
        let out = hl("function foo(int $x, string $y): bool {}");
        assert!(out.contains("tok-type")); // int, string, bool
        assert!(out.contains("tok-keyword")); // function
    }

    #[test]
    fn no_panic_garbage() {
        let _ = hl("$}{<<?php ?><<<\n\r\t\x00🎉");
    }
}
