use crate::scanner::*;
use crate::token::{Token, TokenKind};

pub struct BashScanner;

const KEYWORDS: &[&[u8]] = &[
    b"if", b"then", b"else", b"elif", b"fi", b"for", b"while", b"until", b"do", b"done",
    b"case", b"esac", b"in", b"function", b"select", b"time", b"coproc",
    b"return", b"exit", b"break", b"continue", b"declare", b"local", b"export",
    b"readonly", b"typeset", b"unset", b"shift", b"trap", b"eval", b"exec",
    b"source", b"set",
];

const BUILTINS: &[&[u8]] = &[
    b"echo", b"printf", b"read", b"cd", b"pwd", b"pushd", b"popd", b"dirs",
    b"test", b"true", b"false", b"alias", b"unalias", b"type", b"which",
    b"command", b"builtin", b"enable", b"help", b"hash", b"getopts",
    b"let", b"shopt", b"complete", b"compgen",
];

fn at(b: &[u8], i: usize) -> u8 {
    if i < b.len() { b[i] } else { 0 }
}

impl Scanner for BashScanner {
    fn scan(&self, code: &str) -> Vec<Token> {
        let b = code.as_bytes();
        let mut tokens = Vec::new();
        let mut i = 0;

        while i < b.len() {
            let c = b[i];

            if c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' {
                i += 1;
                continue;
            }

            // Comments
            if c == b'#' {
                // But not $# or ${#...}
                if i == 0 || !matches!(at(b, i - 1), b'$' | b'{') {
                    if let Some(end) = scan_hash_comment(b, i) {
                        tokens.push(Token { kind: TokenKind::Comment, start: i, end });
                        i = end;
                        continue;
                    }
                }
            }

            // Heredoc (basic: <<EOF ... EOF)
            if c == b'<' && at(b, i + 1) == b'<' && at(b, i + 2) != b'<' {
                if let Some(end) = scan_heredoc(b, i) {
                    tokens.push(Token { kind: TokenKind::String, start: i, end });
                    i = end;
                    continue;
                }
            }

            // Strings
            if c == b'"' {
                let end = scan_bash_double_string(b, i);
                tokens.push(Token { kind: TokenKind::String, start: i, end });
                i = end;
                continue;
            }
            if c == b'\'' {
                if let Some(end) = scan_single_string(b, i) {
                    tokens.push(Token { kind: TokenKind::String, start: i, end });
                    i = end;
                    continue;
                }
            }

            // Variables: $VAR, ${VAR}, $(...), $((...))
            if c == b'$' {
                let start = i;
                i += 1;
                if at(b, i) == b'{' {
                    // ${...}
                    i += 1;
                    while i < b.len() && b[i] != b'}' {
                        i += 1;
                    }
                    if i < b.len() { i += 1; }
                    tokens.push(Token { kind: TokenKind::Variable, start, end: i });
                    continue;
                } else if at(b, i) == b'(' {
                    // $(...) — command substitution
                    i += 1;
                    let mut depth = 1u32;
                    while i < b.len() && depth > 0 {
                        if b[i] == b'(' { depth += 1; }
                        if b[i] == b')' { depth -= 1; }
                        i += 1;
                    }
                    tokens.push(Token { kind: TokenKind::Variable, start, end: i });
                    continue;
                } else if at(b, i).is_ascii_alphanumeric() || at(b, i) == b'_' || matches!(at(b, i), b'?' | b'!' | b'@' | b'#' | b'*' | b'-' | b'$' | b'0'..=b'9') {
                    while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                        i += 1;
                    }
                    tokens.push(Token { kind: TokenKind::Variable, start, end: i });
                    continue;
                }
                // Lone $
                tokens.push(Token { kind: TokenKind::Operator, start, end: i });
                continue;
            }

            // Numbers (only at word boundary)
            if c.is_ascii_digit() && (i == 0 || !at(b, i - 1).is_ascii_alphanumeric()) {
                if let Some(end) = scan_number(b, i) {
                    tokens.push(Token { kind: TokenKind::Number, start: i, end });
                    i = end;
                    continue;
                }
            }

            // Identifiers / keywords
            if let Some((end, ident)) = scan_ident(b, i) {
                let kind = if is_keyword(ident, KEYWORDS) {
                    TokenKind::Keyword
                } else if is_keyword(ident, BUILTINS) {
                    TokenKind::Builtin
                } else if is_function_call(b, end) {
                    TokenKind::Function
                } else {
                    TokenKind::Plain
                };
                tokens.push(Token { kind, start: i, end });
                i = end;
                continue;
            }

            // Operators
            if matches!(c, b'|' | b'&' | b'>' | b'<' | b'!' | b'=' | b'-') {
                let start = i;
                // Handle 2-char: ||, &&, >>, <<, >=, etc
                let n = at(b, i + 1);
                if (c == b'|' && n == b'|') || (c == b'&' && n == b'&') || (c == b'>' && n == b'>') {
                    i += 2;
                } else {
                    i += 1;
                }
                tokens.push(Token { kind: TokenKind::Operator, start, end: i });
                continue;
            }

            // Punctuation
            if let Some(end) = scan_punctuation(b, i) {
                tokens.push(Token { kind: TokenKind::Punctuation, start: i, end });
                i = end;
                continue;
            }

            i += 1;
        }

        tokens
    }
}

/// Scan bash double-quoted string (allows $var interpolation but we treat whole thing as string).
fn scan_bash_double_string(b: &[u8], pos: usize) -> usize {
    let mut i = pos + 1;
    while i < b.len() {
        if b[i] == b'\\' {
            i += 2;
        } else if b[i] == b'"' {
            return i + 1;
        } else {
            i += 1;
        }
    }
    b.len()
}

/// Simple heredoc scanner: <<EOF ... EOF or <<'EOF' ... EOF or <<-EOF ... EOF
fn scan_heredoc(b: &[u8], pos: usize) -> Option<usize> {
    let mut i = pos + 2;
    // Skip optional -
    if at(b, i) == b'-' { i += 1; }
    // Skip whitespace
    while i < b.len() && b[i] == b' ' { i += 1; }
    // Get delimiter (may be quoted)
    let strip_quotes = at(b, i) == b'\'' || at(b, i) == b'"';
    if strip_quotes { i += 1; }
    let delim_start = i;
    while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
        i += 1;
    }
    if i == delim_start { return None; }
    let delim = &b[delim_start..i];
    if strip_quotes && i < b.len() { i += 1; } // closing quote

    // Skip to next line
    while i < b.len() && b[i] != b'\n' { i += 1; }
    if i < b.len() { i += 1; }

    // Find closing delimiter on its own line
    while i < b.len() {
        let line_start = i;
        // Skip leading whitespace/tabs
        while i < b.len() && (b[i] == b' ' || b[i] == b'\t') { i += 1; }
        if b[i..].starts_with(delim) {
            let after = i + delim.len();
            if after >= b.len() || b[after] == b'\n' || b[after] == b'\r' {
                return Some(after);
            }
        }
        // Skip to next line
        while i < b.len() && b[i] != b'\n' { i += 1; }
        if i < b.len() { i += 1; }
        if i == line_start { break; } // safety
    }
    Some(b.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::render;

    fn hl(code: &str) -> String {
        let tokens = BashScanner.scan(code);
        render(code, &tokens)
    }

    #[test]
    fn variable() {
        let out = hl("echo $HOME");
        assert!(out.contains("tok-variable"));
    }

    #[test]
    fn comment() {
        let out = hl("# comment");
        assert!(out.contains("tok-comment"));
    }

    #[test]
    fn keyword() {
        let out = hl("if [ -f file ]; then");
        assert!(out.contains("tok-keyword"));
    }

    #[test]
    fn string() {
        let out = hl(r#"echo "hello""#);
        assert!(out.contains("tok-string"));
    }

    #[test]
    fn builtin() {
        let out = hl("echo hello");
        assert!(out.contains("tok-builtin"));
    }
}
