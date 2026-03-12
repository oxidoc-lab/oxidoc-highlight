use crate::scanner::*;
use crate::token::{Token, TokenKind};

pub struct CScanner {
    pub cpp: bool,
}

const C_KEYWORDS: &[&[u8]] = &[
    b"auto", b"break", b"case", b"const", b"continue", b"default", b"do", b"else",
    b"enum", b"extern", b"for", b"goto", b"if", b"inline", b"register", b"restrict",
    b"return", b"sizeof", b"static", b"struct", b"switch", b"typedef", b"union",
    b"volatile", b"while", b"true", b"false", b"NULL",
];

const CPP_EXTRA: &[&[u8]] = &[
    b"alignas", b"alignof", b"and", b"catch", b"class", b"constexpr", b"consteval",
    b"constinit", b"co_await", b"co_return", b"co_yield", b"concept", b"decltype",
    b"delete", b"dynamic_cast", b"explicit", b"export", b"final", b"friend",
    b"module", b"mutable", b"namespace", b"new", b"noexcept", b"not", b"nullptr",
    b"operator", b"or", b"override", b"private", b"protected", b"public",
    b"reinterpret_cast", b"requires", b"static_assert", b"static_cast",
    b"template", b"throw", b"try", b"typeid", b"typename", b"using", b"virtual",
    b"xor",
];

const TYPES: &[&[u8]] = &[
    b"void", b"char", b"short", b"int", b"long", b"float", b"double", b"signed",
    b"unsigned", b"bool", b"size_t", b"ssize_t", b"ptrdiff_t", b"intptr_t",
    b"uintptr_t", b"int8_t", b"int16_t", b"int32_t", b"int64_t",
    b"uint8_t", b"uint16_t", b"uint32_t", b"uint64_t",
    b"string", b"vector", b"map", b"set", b"unique_ptr", b"shared_ptr",
    b"optional", b"variant", b"tuple", b"pair", b"array",
    b"wchar_t", b"char8_t", b"char16_t", b"char32_t",
];

impl Scanner for CScanner {
    fn scan(&self, code: &str) -> Vec<Token> {
        let b = code.as_bytes();
        let mut tokens = Vec::new();
        let mut i = 0;
        let mut prev_kind: Option<TokenKind> = None;

        while i < b.len() {
            let c = b[i];
            if c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' { i += 1; continue; }

            // Preprocessor directives
            if c == b'#' {
                let start = i;
                i += 1;
                while i < b.len() && (b[i] == b' ' || b[i] == b'\t') { i += 1; }
                while i < b.len() && b[i].is_ascii_alphanumeric() { i += 1; }
                // Continue to end of line (handling line continuation)
                while i < b.len() {
                    if b[i] == b'\n' {
                        if i > 0 && b[i-1] == b'\\' { i += 1; continue; }
                        break;
                    }
                    i += 1;
                }
                tokens.push(Token { kind: TokenKind::Attr, start, end: i });
                prev_kind = Some(TokenKind::Attr);
                continue;
            }

            if let Some(end) = scan_c_comment(b, i) {
                tokens.push(Token { kind: TokenKind::Comment, start: i, end });
                prev_kind = Some(TokenKind::Comment);
                i = end;
                continue;
            }
            if c == b'"' { if let Some(end) = scan_double_string(b, i) { tokens.push(Token { kind: TokenKind::String, start: i, end }); prev_kind = Some(TokenKind::String); i = end; continue; } }
            if c == b'\'' { if let Some(end) = scan_single_string(b, i) { tokens.push(Token { kind: TokenKind::String, start: i, end }); prev_kind = Some(TokenKind::String); i = end; continue; } }

            if let Some(end) = scan_number(b, i) { tokens.push(Token { kind: TokenKind::Number, start: i, end }); prev_kind = Some(TokenKind::Number); i = end; continue; }

            if let Some((end, ident)) = scan_ident(b, i) {
                let kind = if is_keyword(ident, C_KEYWORDS) || (self.cpp && is_keyword(ident, CPP_EXTRA)) { TokenKind::Keyword }
                    else if ident == b"this" && self.cpp { TokenKind::Variable }
                    else if is_keyword(ident, TYPES) { TokenKind::Type }
                    else if is_pascal_case(ident) { TokenKind::Type }
                    else if is_function_call(b, end) && !was_keyword(prev_kind) { TokenKind::Function }
                    else if matches!(prev_kind, Some(TokenKind::Punctuation)) && i >= 1 && (b[i-1] == b'.' || (i >= 2 && b[i-2] == b'-' && b[i-1] == b'>')) {
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
    fn hl(code: &str) -> String { render(code, &CScanner { cpp: false }.scan(code)) }
    fn hl_cpp(code: &str) -> String { render(code, &CScanner { cpp: true }.scan(code)) }

    #[test]
    fn preprocessor() { assert!(hl("#include <stdio.h>").contains("tok-attr")); }
    #[test]
    fn keyword() { assert!(hl("if (x) return 0;").contains("tok-keyword")); }
    #[test]
    fn types() { assert!(hl("int x;").contains("tok-type")); }
    #[test]
    fn function() { assert!(hl("printf(\"hi\")").contains("tok-function")); }
    #[test]
    fn cpp_class() { assert!(hl_cpp("class Foo {}").contains("tok-keyword")); }
}
