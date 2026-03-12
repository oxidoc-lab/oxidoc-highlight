use crate::token::{Token, TokenKind};
use crate::scanner::Scanner;

pub struct DiffScanner;

fn at(b: &[u8], i: usize) -> u8 {
    if i < b.len() { b[i] } else { 0 }
}

impl Scanner for DiffScanner {
    fn scan(&self, code: &str) -> Vec<Token> {
        let b = code.as_bytes();
        let mut tokens = Vec::new();
        let mut i = 0;

        while i < b.len() {
            let line_start = i;
            // Find end of line
            while i < b.len() && b[i] != b'\n' { i += 1; }
            let line_end = i;
            if i < b.len() { i += 1; } // consume \n

            if line_start == line_end { continue; }

            let c = b[line_start];
            let kind = if c == b'+' {
                if at(b, line_start + 1) == b'+' && at(b, line_start + 2) == b'+' {
                    TokenKind::Keyword // +++ header
                } else {
                    TokenKind::String // added line (green)
                }
            } else if c == b'-' {
                if at(b, line_start + 1) == b'-' && at(b, line_start + 2) == b'-' {
                    TokenKind::Keyword // --- header
                } else {
                    TokenKind::Comment // removed line (red) — using comment for red styling
                }
            } else if c == b'@' && at(b, line_start + 1) == b'@' {
                TokenKind::Attr // @@ hunk header
            } else if b[line_start..line_end].starts_with(b"diff ") || b[line_start..line_end].starts_with(b"index ") {
                TokenKind::Keyword
            } else {
                TokenKind::Plain
            };

            tokens.push(Token { kind, start: line_start, end: line_end });
        }

        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::render;
    fn hl(code: &str) -> String { render(code, &DiffScanner.scan(code)) }

    #[test]
    fn added() { assert!(hl("+added line").contains("tok-string")); }
    #[test]
    fn removed() { assert!(hl("-removed line").contains("tok-comment")); }
    #[test]
    fn header() { assert!(hl("@@ -1,3 +1,4 @@").contains("tok-attr")); }
    #[test]
    fn diff_header() { assert!(hl("diff --git a/f b/f").contains("tok-keyword")); }
    #[test]
    fn context() {
        let out = hl(" context line");
        // Context lines are plain
        assert!(!out.contains("tok-keyword"));
    }
}
