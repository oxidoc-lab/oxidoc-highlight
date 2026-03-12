use crate::scanner::*;
use crate::token::{Token, TokenKind};

pub struct JsScanner {
    pub typescript: bool,
    pub jsx: bool,
}

const JS_KEYWORDS: &[&[u8]] = &[
    b"async", b"await", b"break", b"case", b"catch", b"class", b"const", b"continue",
    b"debugger", b"default", b"delete", b"do", b"else", b"export", b"extends", b"false",
    b"finally", b"for", b"from", b"function", b"if", b"import", b"in", b"instanceof",
    b"let", b"new", b"null", b"of", b"return", b"static", b"switch", b"throw", b"true",
    b"try", b"typeof", b"undefined", b"var", b"void", b"while", b"with", b"yield",
];

const TS_EXTRA_KEYWORDS: &[&[u8]] = &[
    b"abstract", b"as", b"asserts", b"declare", b"enum", b"implements", b"interface",
    b"is", b"keyof", b"namespace", b"override", b"private", b"protected", b"public",
    b"readonly", b"satisfies", b"type", b"using", b"infer",
];

const BUILTINS: &[&[u8]] = &[
    b"console", b"Math", b"JSON", b"Object", b"Array", b"Map", b"Set", b"Promise",
    b"Date", b"RegExp", b"Error", b"parseInt", b"parseFloat", b"isNaN", b"isFinite",
    b"setTimeout", b"setInterval", b"clearTimeout", b"clearInterval",
    b"require", b"module", b"exports", b"globalThis", b"window", b"document",
    b"fetch", b"Response", b"Request", b"URL", b"URLSearchParams",
    b"Buffer", b"process",
];

const TYPE_KEYWORDS: &[&[u8]] = &[
    b"string", b"number", b"boolean", b"any", b"unknown", b"never", b"void", b"object",
    b"symbol", b"bigint", b"undefined", b"null",
];

fn at(b: &[u8], i: usize) -> u8 {
    if i < b.len() { b[i] } else { 0 }
}

/// Heuristic: is the character before `pos` one that suggests `/` starts a regex?
/// `prev_punct_byte` is the actual byte of the last punctuation token (if prev was punctuation).
fn slash_is_regex(prev_kind: Option<TokenKind>, prev_punct_byte: u8) -> bool {
    // After operator, keyword, open paren/bracket, comma, semicolon, or start of line => regex
    match prev_kind {
        None => true,
        Some(TokenKind::Operator) => true,
        Some(TokenKind::Keyword) => true,
        Some(TokenKind::Punctuation) => {
            // After ) or ] it's division; after ( [ { , ; it's regex
            !matches!(prev_punct_byte, b')' | b']')
        }
        _ => false,
    }
}

impl Scanner for JsScanner {
    fn scan(&self, code: &str) -> Vec<Token> {
        let b = code.as_bytes();
        let mut tokens = Vec::new();
        let mut i = 0;
        let mut prev_kind: Option<TokenKind> = None;
        let mut prev_punct_byte: u8 = 0;

        while i < b.len() {
            let c = b[i];

            if c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' {
                i += 1;
                continue;
            }

            // Comments
            if let Some(end) = scan_c_comment(b, i) {
                tokens.push(Token { kind: TokenKind::Comment, start: i, end });
                prev_kind = Some(TokenKind::Comment);
                i = end;
                continue;
            }

            // Regex literals
            if c == b'/' && slash_is_regex(prev_kind, prev_punct_byte) && at(b, i + 1) != b'/' && at(b, i + 1) != b'*' {
                if let Some(end) = scan_js_regex(b, i) {
                    tokens.push(Token { kind: TokenKind::String, start: i, end });
                    prev_kind = Some(TokenKind::String);
                    i = end;
                    continue;
                }
            }

            // Template literals
            if c == b'`' {
                let end = scan_template_literal(b, i);
                tokens.push(Token { kind: TokenKind::String, start: i, end });
                prev_kind = Some(TokenKind::String);
                i = end;
                continue;
            }

            // Strings
            if c == b'"' {
                if let Some(end) = scan_double_string(b, i) {
                    tokens.push(Token { kind: TokenKind::String, start: i, end });
                    prev_kind = Some(TokenKind::String);
                    i = end;
                    continue;
                }
            }
            if c == b'\'' {
                if let Some(end) = scan_single_string(b, i) {
                    tokens.push(Token { kind: TokenKind::String, start: i, end });
                    prev_kind = Some(TokenKind::String);
                    i = end;
                    continue;
                }
            }

            // Decorators (TS)
            if c == b'@' && self.typescript {
                let start = i;
                i += 1;
                if let Some((end, _)) = scan_ident(b, i) {
                    // Include dotted names like @Module.decorator
                    let mut e = end;
                    while at(b, e) == b'.' {
                        if let Some((end2, _)) = scan_ident(b, e + 1) {
                            e = end2;
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token { kind: TokenKind::Attr, start, end: e });
                    prev_kind = Some(TokenKind::Attr);
                    i = e;
                    continue;
                }
            }

            // Numbers
            if let Some(end) = scan_number(b, i) {
                tokens.push(Token { kind: TokenKind::Number, start: i, end });
                prev_kind = Some(TokenKind::Number);
                i = end;
                continue;
            }

            // Identifiers
            if let Some((end, ident)) = scan_ident(b, i) {
                let kind = if is_keyword(ident, JS_KEYWORDS) || (self.typescript && is_keyword(ident, TS_EXTRA_KEYWORDS)) {
                    TokenKind::Keyword
                } else if ident == b"this" || ident == b"super" {
                    TokenKind::Variable
                } else if self.typescript && is_keyword(ident, TYPE_KEYWORDS) {
                    TokenKind::Type
                } else if is_keyword(ident, BUILTINS) {
                    TokenKind::Builtin
                } else if is_pascal_case(ident) {
                    TokenKind::Type
                } else if is_function_call(b, end) && !was_keyword(prev_kind) {
                    TokenKind::Function
                } else if matches!(prev_kind, Some(TokenKind::Punctuation)) && i >= 1 && b[i - 1] == b'.' {
                    if is_function_call(b, end) {
                        TokenKind::Function
                    } else {
                        TokenKind::Property
                    }
                } else {
                    TokenKind::Plain
                };
                tokens.push(Token { kind, start: i, end });
                prev_kind = Some(kind);
                i = end;
                continue;
            }

            // JSX tags: <Component />, <div>, </div>, <>, </>
            if self.jsx && c == b'<' && at(b, i + 1) != b'<' {
                // Not << (left shift)
                let tag_start = i;
                let next = at(b, i + 1);

                // Closing tag </Name> or fragment </>
                if next == b'/' {
                    if at(b, i + 2) == b'>' {
                        // Fragment close </>
                        tokens.push(Token { kind: TokenKind::Punctuation, start: i, end: i + 3 });
                        prev_kind = Some(TokenKind::Punctuation);
                        i += 3;
                        continue;
                    }
                    let name_pos = i + 2;
                    if at(b, name_pos).is_ascii_alphabetic() {
                        i = name_pos;
                        while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_' || b[i] == b'-' || b[i] == b'.') {
                            i += 1;
                        }
                        let kind = if at(b, name_pos).is_ascii_uppercase() { TokenKind::Keyword } else { TokenKind::Attr };
                        tokens.push(Token { kind, start: tag_start, end: i });
                        prev_kind = Some(kind);
                        // Skip to >
                        while i < b.len() && b[i] != b'>' { i += 1; }
                        if i < b.len() {
                            tokens.push(Token { kind: TokenKind::Punctuation, start: i, end: i + 1 });
                            prev_kind = Some(TokenKind::Punctuation);
                            i += 1;
                        }
                        continue;
                    }
                }

                // Fragment open <>
                if next == b'>' {
                    tokens.push(Token { kind: TokenKind::Punctuation, start: i, end: i + 2 });
                    prev_kind = Some(TokenKind::Punctuation);
                    i += 2;
                    continue;
                }

                // Opening tag <Name ...> or <Name ... />
                if next.is_ascii_alphabetic() || next == b'_' {
                    i += 1;
                    while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_' || b[i] == b'-' || b[i] == b'.') {
                        i += 1;
                    }
                    let kind = if next.is_ascii_uppercase() { TokenKind::Keyword } else { TokenKind::Attr };
                    tokens.push(Token { kind, start: tag_start, end: i });
                    prev_kind = Some(kind);

                    // Scan JSX attributes until > or />
                    while i < b.len() && b[i] != b'>' {
                        if b[i] == b' ' || b[i] == b'\t' || b[i] == b'\n' || b[i] == b'\r' { i += 1; continue; }
                        if b[i] == b'/' && at(b, i + 1) == b'>' {
                            tokens.push(Token { kind: TokenKind::Punctuation, start: i, end: i + 2 });
                            prev_kind = Some(TokenKind::Punctuation);
                            i += 2;
                            break;
                        }
                        // Spread {...expr}
                        if b[i] == b'{' {
                            let vs = i;
                            i += 1;
                            let mut depth = 1u32;
                            while i < b.len() && depth > 0 {
                                if b[i] == b'{' { depth += 1; }
                                if b[i] == b'}' { depth -= 1; }
                                i += 1;
                            }
                            tokens.push(Token { kind: TokenKind::Plain, start: vs, end: i });
                            continue;
                        }
                        // Attribute name
                        if b[i].is_ascii_alphabetic() || b[i] == b'_' {
                            let as_ = i;
                            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_' || b[i] == b'-') { i += 1; }
                            tokens.push(Token { kind: TokenKind::Attr, start: as_, end: i });
                            // =
                            if at(b, i) == b'=' {
                                tokens.push(Token { kind: TokenKind::Operator, start: i, end: i + 1 });
                                i += 1;
                                // Value: string or {expression}
                                if at(b, i) == b'"' || at(b, i) == b'\'' {
                                    let q = b[i];
                                    let vs = i;
                                    if let Some(end) = scan_quoted_string(b, i, q) {
                                        tokens.push(Token { kind: TokenKind::String, start: vs, end });
                                        i = end;
                                    }
                                } else if at(b, i) == b'{' {
                                    let vs = i;
                                    i += 1;
                                    let mut depth = 1u32;
                                    while i < b.len() && depth > 0 {
                                        if b[i] == b'{' { depth += 1; }
                                        if b[i] == b'}' { depth -= 1; }
                                        i += 1;
                                    }
                                    tokens.push(Token { kind: TokenKind::Plain, start: vs, end: i });
                                }
                            }
                            continue;
                        }
                        i += 1;
                    }
                    if i < b.len() && b[i] == b'>' {
                        tokens.push(Token { kind: TokenKind::Punctuation, start: i, end: i + 1 });
                        prev_kind = Some(TokenKind::Punctuation);
                        i += 1;
                    }
                    continue;
                }
            }

            // Arrow =>
            if c == b'=' && at(b, i + 1) == b'>' {
                tokens.push(Token { kind: TokenKind::Operator, start: i, end: i + 2 });
                prev_kind = Some(TokenKind::Operator);
                i += 2;
                continue;
            }

            // Operators
            if let Some(end) = scan_operator(b, i) {
                tokens.push(Token { kind: TokenKind::Operator, start: i, end });
                prev_kind = Some(TokenKind::Operator);
                i = end;
                continue;
            }

            // Punctuation
            if let Some(end) = scan_punctuation(b, i) {
                prev_punct_byte = b[i];
                tokens.push(Token { kind: TokenKind::Punctuation, start: i, end });
                prev_kind = Some(TokenKind::Punctuation);
                i = end;
                continue;
            }

            i += 1;
        }

        tokens
    }
}

/// Scan a JS regex literal `/pattern/flags`.
fn scan_js_regex(b: &[u8], pos: usize) -> Option<usize> {
    if at(b, pos) != b'/' {
        return None;
    }
    let mut i = pos + 1;
    let mut in_class = false;
    while i < b.len() {
        match b[i] {
            b'\\' => i += 2,
            b'[' => { in_class = true; i += 1; }
            b']' => { in_class = false; i += 1; }
            b'/' if !in_class => {
                i += 1;
                // Consume flags
                while i < b.len() && b[i].is_ascii_alphabetic() {
                    i += 1;
                }
                return Some(i);
            }
            b'\n' | b'\r' => return None, // regex can't span lines
            _ => i += 1,
        }
    }
    None
}

/// Scan a JS template literal `` `...` `` including `${...}` expressions.
/// For simplicity, we treat the entire thing as a string token.
fn scan_template_literal(b: &[u8], pos: usize) -> usize {
    let mut i = pos + 1;
    let mut depth = 0u32;
    while i < b.len() {
        match b[i] {
            b'\\' => i += 2,
            b'$' if at(b, i + 1) == b'{' && depth == 0 => {
                depth += 1;
                i += 2;
            }
            b'{' if depth > 0 => { depth += 1; i += 1; }
            b'}' if depth > 0 => {
                depth -= 1;
                i += 1;
            }
            b'`' if depth == 0 => return i + 1,
            _ => i += 1,
        }
    }
    b.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::render;

    fn hl(code: &str) -> String {
        let tokens = JsScanner { typescript: false, jsx: false }.scan(code);
        render(code, &tokens)
    }

    fn hl_ts(code: &str) -> String {
        let tokens = JsScanner { typescript: true, jsx: false }.scan(code);
        render(code, &tokens)
    }

    #[test]
    fn keywords() {
        let out = hl("const x = 42;");
        assert!(out.contains("tok-keyword"));
        assert!(out.contains("tok-number"));
    }

    #[test]
    fn template_literal() {
        let out = hl("`hello ${name}`");
        assert!(out.contains("tok-string"));
    }

    #[test]
    fn regex_vs_division() {
        // After = it's regex
        let out = hl("x = /pattern/g");
        assert!(out.contains("tok-string")); // regex as string token

        // After number it's division
        let out2 = hl("a / b");
        assert!(out2.contains("tok-operator"));
    }

    #[test]
    fn arrow_function() {
        let out = hl("(x) => x");
        assert!(out.contains("tok-operator"));
        assert!(out.contains("=&gt;"));
    }

    #[test]
    fn ts_decorator() {
        let out = hl_ts("@Component");
        assert!(out.contains("tok-attr"));
    }

    #[test]
    fn ts_type_keyword() {
        let out = hl_ts("let x: string");
        assert!(out.contains("tok-type"));
    }

    #[test]
    fn function_call() {
        let out = hl("foo(x)");
        assert!(out.contains("tok-function"));
    }

    #[test]
    fn if_not_function() {
        let out = hl("if (true)");
        // "if" should be keyword, not function
        assert!(out.contains("tok-keyword"));
        assert!(!out.contains("tok-function"));
    }

    fn hl_jsx(code: &str) -> String {
        let tokens = JsScanner { typescript: false, jsx: true }.scan(code);
        render(code, &tokens)
    }

    fn hl_tsx(code: &str) -> String {
        let tokens = JsScanner { typescript: true, jsx: true }.scan(code);
        render(code, &tokens)
    }

    #[test]
    fn jsx_component() {
        let out = hl_jsx(r#"<Button onClick={handleClick}>Click</Button>"#);
        assert!(out.contains("tok-keyword")); // <Button
        assert!(out.contains("tok-attr")); // onClick
    }

    #[test]
    fn jsx_self_closing() {
        let out = hl_jsx(r#"<Header />"#);
        assert!(out.contains("tok-keyword")); // <Header (PascalCase = keyword)
    }

    #[test]
    fn jsx_html_tag() {
        let out = hl_jsx(r#"<div className="app">"#);
        assert!(out.contains("tok-attr")); // <div (lowercase = attr), className
        assert!(out.contains("tok-string")); // "app"
    }

    #[test]
    fn jsx_closing_tag() {
        let out = hl_jsx("</Button>");
        assert!(out.contains("tok-keyword")); // </Button
    }

    #[test]
    fn jsx_fragment() {
        let out = hl_jsx("<>content</>");
        assert!(out.contains("tok-punctuation")); // <>, </>
    }

    #[test]
    fn tsx_with_generics() {
        let out = hl_tsx("const App: React.FC<Props> = () => <Header />;");
        assert!(out.contains("tok-keyword")); // const, <Header
        assert!(out.contains("tok-type")); // Props (PascalCase)
    }
}
