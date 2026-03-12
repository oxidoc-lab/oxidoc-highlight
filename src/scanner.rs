use crate::token::{Token, TokenKind};

/// Scanner trait — each language implements this.
pub trait Scanner {
    fn scan(&self, code: &str) -> Vec<Token>;
}

// ── Shared scanning helpers ──────────────────────────────────────────

/// Bytes helper: get byte at position, or 0 if out of bounds.
#[inline]
fn at(b: &[u8], i: usize) -> u8 {
    if i < b.len() { b[i] } else { 0 }
}

/// Scan a C-style line comment (`//` to end of line). Returns end position or `None`.
pub fn scan_line_comment(b: &[u8], pos: usize, prefix: &[u8]) -> Option<usize> {
    if b[pos..].starts_with(prefix) {
        let mut i = pos + prefix.len();
        while i < b.len() && b[i] != b'\n' {
            i += 1;
        }
        Some(i)
    } else {
        None
    }
}

/// Scan a C-style block comment (`/* ... */`). Returns end position or `None`.
pub fn scan_block_comment(b: &[u8], pos: usize) -> Option<usize> {
    if at(b, pos) == b'/' && at(b, pos + 1) == b'*' {
        let mut i = pos + 2;
        while i + 1 < b.len() {
            if b[i] == b'*' && b[i + 1] == b'/' {
                return Some(i + 2);
            }
            i += 1;
        }
        Some(b.len()) // unterminated
    } else {
        None
    }
}

/// Scan a `#`-style line comment. Returns end position or `None`.
pub fn scan_hash_comment(b: &[u8], pos: usize) -> Option<usize> {
    scan_line_comment(b, pos, b"#")
}

/// Scan `//` line comment.
pub fn scan_slash_comment(b: &[u8], pos: usize) -> Option<usize> {
    scan_line_comment(b, pos, b"//")
}

/// Scan C-style comment (either `//` or `/* */`).
pub fn scan_c_comment(b: &[u8], pos: usize) -> Option<usize> {
    scan_slash_comment(b, pos).or_else(|| scan_block_comment(b, pos))
}

/// Scan a double-quoted string with backslash escapes. Returns end position or `None`.
pub fn scan_double_string(b: &[u8], pos: usize) -> Option<usize> {
    scan_quoted_string(b, pos, b'"')
}

/// Scan a single-quoted string with backslash escapes. Returns end position or `None`.
pub fn scan_single_string(b: &[u8], pos: usize) -> Option<usize> {
    scan_quoted_string(b, pos, b'\'')
}

/// Scan a quoted string with the given delimiter and backslash escapes.
pub fn scan_quoted_string(b: &[u8], pos: usize, delim: u8) -> Option<usize> {
    if at(b, pos) != delim {
        return None;
    }
    let mut i = pos + 1;
    while i < b.len() {
        if b[i] == b'\\' {
            i += 2; // skip escaped char
        } else if b[i] == delim {
            return Some(i + 1);
        } else {
            i += 1;
        }
    }
    Some(b.len()) // unterminated
}

/// Scan a backtick string (no escape handling, for simple cases). Returns end position or `None`.
pub fn scan_backtick_string(b: &[u8], pos: usize) -> Option<usize> {
    scan_quoted_string(b, pos, b'`')
}

/// Scan a numeric literal: int, float, hex, binary, octal, with `_` separators.
/// Returns end position or `None`.
pub fn scan_number(b: &[u8], pos: usize) -> Option<usize> {
    let c = at(b, pos);
    if !c.is_ascii_digit() {
        // Leading dot like `.5`
        if c == b'.' && at(b, pos + 1).is_ascii_digit() {
            let mut i = pos + 1;
            while i < b.len() && (b[i].is_ascii_digit() || b[i] == b'_') {
                i += 1;
            }
            // exponent
            if at(b, i) == b'e' || at(b, i) == b'E' {
                i += 1;
                if at(b, i) == b'+' || at(b, i) == b'-' {
                    i += 1;
                }
                while i < b.len() && (b[i].is_ascii_digit() || b[i] == b'_') {
                    i += 1;
                }
            }
            return Some(i);
        }
        return None;
    }

    let mut i = pos;

    // Check for 0x, 0b, 0o prefixes
    if c == b'0' {
        let next = at(b, pos + 1);
        if next == b'x' || next == b'X' {
            i = pos + 2;
            while i < b.len() && (b[i].is_ascii_hexdigit() || b[i] == b'_') {
                i += 1;
            }
            return if i > pos + 2 { Some(i) } else { Some(pos + 1) };
        }
        if next == b'b' || next == b'B' {
            i = pos + 2;
            while i < b.len() && (b[i] == b'0' || b[i] == b'1' || b[i] == b'_') {
                i += 1;
            }
            return if i > pos + 2 { Some(i) } else { Some(pos + 1) };
        }
        if next == b'o' || next == b'O' {
            i = pos + 2;
            while i < b.len() && ((b[i] >= b'0' && b[i] <= b'7') || b[i] == b'_') {
                i += 1;
            }
            return if i > pos + 2 { Some(i) } else { Some(pos + 1) };
        }
    }

    // Integer part
    while i < b.len() && (b[i].is_ascii_digit() || b[i] == b'_') {
        i += 1;
    }

    // Fractional part
    if at(b, i) == b'.' && at(b, i + 1).is_ascii_digit() {
        i += 1;
        while i < b.len() && (b[i].is_ascii_digit() || b[i] == b'_') {
            i += 1;
        }
    }

    // Exponent
    if at(b, i) == b'e' || at(b, i) == b'E' {
        i += 1;
        if at(b, i) == b'+' || at(b, i) == b'-' {
            i += 1;
        }
        while i < b.len() && (b[i].is_ascii_digit() || b[i] == b'_') {
            i += 1;
        }
    }

    // Type suffix (e.g., i32, u64, f64, usize)
    if at(b, i).is_ascii_alphabetic() {
        let suffix_start = i;
        while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
            i += 1;
        }
        // Only consume known numeric suffixes
        let suffix = &b[suffix_start..i];
        let known = matches!(
            suffix,
            b"i8"
                | b"i16"
                | b"i32"
                | b"i64"
                | b"i128"
                | b"isize"
                | b"u8"
                | b"u16"
                | b"u32"
                | b"u64"
                | b"u128"
                | b"usize"
                | b"f32"
                | b"f64"
                | b"n" // BigInt in JS
        );
        if !known {
            i = suffix_start;
        }
    }

    if i > pos { Some(i) } else { None }
}

/// Scan an identifier `[a-zA-Z_][a-zA-Z0-9_]*`. Returns `(end, identifier_slice)` or `None`.
pub fn scan_ident(b: &[u8], pos: usize) -> Option<(usize, &[u8])> {
    let c = at(b, pos);
    if !c.is_ascii_alphabetic() && c != b'_' {
        return None;
    }
    let mut i = pos + 1;
    while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
        i += 1;
    }
    Some((i, &b[pos..i]))
}

/// Check if a byte slice is PascalCase (starts uppercase, has at least one lowercase).
pub fn is_pascal_case(ident: &[u8]) -> bool {
    if ident.is_empty() || !ident[0].is_ascii_uppercase() {
        return false;
    }
    // Must have at least one lowercase letter (to exclude ALL_CAPS constants)
    ident.iter().any(|&c| c.is_ascii_lowercase())
}

/// Skip whitespace (not newlines) starting at `pos`. Returns new position.
pub fn skip_whitespace_no_newline(b: &[u8], pos: usize) -> usize {
    let mut i = pos;
    while i < b.len() && (b[i] == b' ' || b[i] == b'\t') {
        i += 1;
    }
    i
}

/// Look at the character after optional whitespace. Used for function detection.
pub fn peek_after_whitespace(b: &[u8], pos: usize) -> u8 {
    at(b, skip_whitespace_no_newline(b, pos))
}

/// Check if an identifier is followed by `(` (function call heuristic).
pub fn is_function_call(b: &[u8], end: usize) -> bool {
    peek_after_whitespace(b, end) == b'('
}

/// Scan common operators (greedy, longest match). Returns `(end, is_operator)` or `None`.
pub fn scan_operator(b: &[u8], pos: usize) -> Option<usize> {
    let c = at(b, pos);
    let n = at(b, pos + 1);
    let nn = at(b, pos + 2);

    // 3-char operators
    match (c, n, nn) {
        (b'>', b'>', b'>')
        | (b'<', b'<', b'=')
        | (b'>', b'>', b'=')
        | (b'=', b'=', b'=')
        | (b'!', b'=', b'=')
        | (b'.', b'.', b'.')
        | (b'.', b'.', b'=') => return Some(pos + 3),
        _ => {}
    }

    // 2-char operators
    match (c, n) {
        (b'=', b'=')
        | (b'!', b'=')
        | (b'<', b'=')
        | (b'>', b'=')
        | (b'&', b'&')
        | (b'|', b'|')
        | (b'+', b'=')
        | (b'-', b'=')
        | (b'*', b'=')
        | (b'/', b'=')
        | (b'%', b'=')
        | (b'&', b'=')
        | (b'|', b'=')
        | (b'^', b'=')
        | (b'<', b'<')
        | (b'>', b'>')
        | (b'-', b'>')
        | (b'=', b'>')
        | (b'+', b'+')
        | (b'-', b'-')
        | (b'?', b'?')
        | (b'?', b'.')
        | (b'.', b'.') => return Some(pos + 2),
        _ => {}
    }

    // 1-char operators
    if matches!(
        c,
        b'=' | b'+'
            | b'-'
            | b'*'
            | b'/'
            | b'%'
            | b'!'
            | b'<'
            | b'>'
            | b'&'
            | b'|'
            | b'^'
            | b'~'
            | b'?'
    ) {
        return Some(pos + 1);
    }

    None
}

/// Scan punctuation. Returns end position or `None`.
pub fn scan_punctuation(b: &[u8], pos: usize) -> Option<usize> {
    if matches!(
        at(b, pos),
        b'(' | b')' | b'[' | b']' | b'{' | b'}' | b';' | b',' | b'.' | b':'
    ) {
        Some(pos + 1)
    } else {
        None
    }
}

/// Helper: check if an identifier is in a keyword set.
pub fn is_keyword(ident: &[u8], keywords: &[&[u8]]) -> bool {
    keywords.contains(&ident)
}

/// Check if previous non-whitespace token was a keyword (for avoiding `if(` as function).
/// `prev_kind` should be the kind of the last emitted token.
pub fn was_keyword(prev_kind: Option<TokenKind>) -> bool {
    matches!(prev_kind, Some(TokenKind::Keyword))
}
