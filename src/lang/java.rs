use crate::scanner::*;
use crate::token::{Token, TokenKind};

pub struct JavaScanner;

const KEYWORDS: &[&[u8]] = &[
    b"abstract", b"assert", b"break", b"case", b"catch", b"class", b"const",
    b"continue", b"default", b"do", b"else", b"enum", b"extends", b"final",
    b"finally", b"for", b"goto", b"if", b"implements", b"import", b"instanceof",
    b"interface", b"native", b"new", b"package", b"private", b"protected", b"public",
    b"return", b"static", b"strictfp", b"super", b"switch", b"synchronized",
    b"throw", b"throws", b"transient", b"try", b"volatile", b"while",
    b"true", b"false", b"null", b"var", b"yield", b"record", b"sealed", b"permits",
];

const TYPES: &[&[u8]] = &[
    b"boolean", b"byte", b"char", b"double", b"float", b"int", b"long", b"short", b"void",
    b"String", b"Integer", b"Long", b"Double", b"Float", b"Boolean", b"Character",
    b"Object", b"Class", b"Void",
];

impl Scanner for JavaScanner {
    fn scan(&self, code: &str) -> Vec<Token> {
        let b = code.as_bytes();
        let mut tokens = Vec::new();
        let mut i = 0;
        let mut prev_kind: Option<TokenKind> = None;

        while i < b.len() {
            let c = b[i];
            if c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' { i += 1; continue; }

            // Annotations
            if c == b'@' {
                let start = i;
                i += 1;
                if let Some((end, _)) = scan_ident(b, i) {
                    tokens.push(Token { kind: TokenKind::Attr, start, end });
                    prev_kind = Some(TokenKind::Attr);
                    i = end;
                    continue;
                }
            }

            if let Some(end) = scan_c_comment(b, i) { tokens.push(Token { kind: TokenKind::Comment, start: i, end }); prev_kind = Some(TokenKind::Comment); i = end; continue; }
            if c == b'"' { if let Some(end) = scan_double_string(b, i) { tokens.push(Token { kind: TokenKind::String, start: i, end }); prev_kind = Some(TokenKind::String); i = end; continue; } }
            if c == b'\'' { if let Some(end) = scan_single_string(b, i) { tokens.push(Token { kind: TokenKind::String, start: i, end }); prev_kind = Some(TokenKind::String); i = end; continue; } }
            if let Some(end) = scan_number(b, i) { tokens.push(Token { kind: TokenKind::Number, start: i, end }); prev_kind = Some(TokenKind::Number); i = end; continue; }

            if let Some((end, ident)) = scan_ident(b, i) {
                let kind = if is_keyword(ident, KEYWORDS) { TokenKind::Keyword }
                    else if ident == b"this" { TokenKind::Variable }
                    else if is_keyword(ident, TYPES) || is_pascal_case(ident) { TokenKind::Type }
                    else if is_function_call(b, end) && !was_keyword(prev_kind) { TokenKind::Function }
                    else if matches!(prev_kind, Some(TokenKind::Punctuation)) && i >= 1 && b[i-1] == b'.' {
                        if is_function_call(b, end) { TokenKind::Function } else { TokenKind::Property }
                    }
                    else { TokenKind::Plain };
                tokens.push(Token { kind, start: i, end });
                prev_kind = Some(kind);
                i = end;
                continue;
            }

            if let Some(end) = scan_operator(b, i) { tokens.push(Token { kind: TokenKind::Operator, start: i, end }); prev_kind = Some(TokenKind::Operator); i = end; continue; }
            if let Some(end) = scan_punctuation(b, i) { tokens.push(Token { kind: TokenKind::Punctuation, start: i, end }); prev_kind = Some(TokenKind::Punctuation); i = end; continue; }
            i += 1;
        }
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::render;
    fn hl(code: &str) -> String { render(code, &JavaScanner.scan(code)) }

    #[test]
    fn annotation() { assert!(hl("@Override").contains("tok-attr")); }
    #[test]
    fn keyword() { assert!(hl("public class Foo").contains("tok-keyword")); }
    #[test]
    fn type_name() { assert!(hl("public class Foo").contains("tok-type")); }
    #[test]
    fn method() { assert!(hl("foo.bar()").contains("tok-function")); }
    #[test]
    fn string() { assert!(hl(r#""hello""#).contains("tok-string")); }
}
