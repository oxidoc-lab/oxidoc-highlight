use crate::scanner::*;
use crate::token::{Token, TokenKind};

pub struct RdxScanner;

fn at(b: &[u8], i: usize) -> u8 {
    if i < b.len() { b[i] } else { 0 }
}

/// Check if byte is uppercase ASCII letter.
fn is_upper(c: u8) -> bool {
    c.is_ascii_uppercase()
}

impl Scanner for RdxScanner {
    fn scan(&self, code: &str) -> Vec<Token> {
        let b = code.as_bytes();
        let mut tokens = Vec::new();
        let mut i = 0;
        let mut line_start = true;
        let mut in_frontmatter = false;
        let mut frontmatter_started = false;
        let mut in_code_fence = false;
        let mut code_fence_len: usize = 0;

        while i < b.len() {
            let c = b[i];

            if c == b'\n' {
                line_start = true;
                i += 1;
                continue;
            }

            // ── Frontmatter (--- at line 1) ──────────────────────────
            if line_start && !frontmatter_started && i == 0 && b[i..].starts_with(b"---") {
                let start = i;
                i += 3;
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
                tokens.push(Token {
                    kind: TokenKind::Keyword,
                    start,
                    end: i,
                });
                in_frontmatter = true;
                frontmatter_started = true;
                line_start = true;
                if i < b.len() {
                    i += 1;
                }
                continue;
            }

            if in_frontmatter {
                if line_start && b[i..].starts_with(b"---") {
                    let start = i;
                    i += 3;
                    while i < b.len() && b[i] != b'\n' {
                        i += 1;
                    }
                    tokens.push(Token {
                        kind: TokenKind::Keyword,
                        start,
                        end: i,
                    });
                    in_frontmatter = false;
                    line_start = true;
                    if i < b.len() {
                        i += 1;
                    }
                    continue;
                }
                // Inside frontmatter: scan as YAML key: value
                let start = i;
                scan_frontmatter_line(b, &mut i, &mut tokens);
                if i == start {
                    i += 1;
                } // safety
                line_start = true;
                continue;
            }

            // Track code fences — content inside is NOT tokenized
            if line_start && !in_code_fence {
                // Skip leading whitespace
                let mut ws = i;
                while ws < b.len() && (b[ws] == b' ' || b[ws] == b'\t') {
                    ws += 1;
                }

                if at(b, ws) == b'`' {
                    let mut fl = 0;
                    while ws < b.len() && b[ws] == b'`' {
                        ws += 1;
                        fl += 1;
                    }
                    if fl >= 3 {
                        // Code fence opening — include lang tag
                        let start = i;
                        while i < b.len() && b[i] != b'\n' {
                            i += 1;
                        }
                        tokens.push(Token {
                            kind: TokenKind::String,
                            start,
                            end: i,
                        });
                        in_code_fence = true;
                        code_fence_len = fl;
                        line_start = true;
                        if i < b.len() {
                            i += 1;
                        }
                        continue;
                    }
                }
            }

            if in_code_fence {
                // Check for closing fence
                let mut ws = i;
                while ws < b.len() && (b[ws] == b' ' || b[ws] == b'\t') {
                    ws += 1;
                }
                let mut fl = 0;
                while ws < b.len() && b[ws] == b'`' {
                    ws += 1;
                    fl += 1;
                }
                if fl >= code_fence_len {
                    // Check rest of line is whitespace
                    let mut rest = ws;
                    while rest < b.len() && (b[rest] == b' ' || b[rest] == b'\t') {
                        rest += 1;
                    }
                    if rest >= b.len() || b[rest] == b'\n' {
                        let start = i;
                        i = rest;
                        tokens.push(Token {
                            kind: TokenKind::String,
                            start,
                            end: i,
                        });
                        in_code_fence = false;
                        line_start = true;
                        continue;
                    }
                }
                // Code fence content — pass through as plain
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
                line_start = true;
                if i < b.len() {
                    i += 1;
                }
                continue;
            }

            if c == b' ' || c == b'\t' || c == b'\r' {
                i += 1;
                continue;
            }

            frontmatter_started = true;
            let was_line_start = line_start;
            line_start = false;

            // ── HTML comments <!-- --> ────────────────────────────────
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

            // ── Escape sequences \{, \}, \\, \{{ ─────────────────────
            if c == b'\\' && i + 1 < b.len() {
                let next = b[i + 1];
                if matches!(next, b'{' | b'}' | b'\\') {
                    tokens.push(Token {
                        kind: TokenKind::Operator,
                        start: i,
                        end: i + 2,
                    });
                    i += 2;
                    continue;
                }
            }

            // ── Context variables {$path.to.var} ─────────────────────
            if c == b'{' && at(b, i + 1) == b'$' {
                let start = i;
                i += 2;
                // Scan variable path: [a-zA-Z_][a-zA-Z0-9_]*(\.[a-zA-Z_][a-zA-Z0-9_]*)*
                while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_' || b[i] == b'.')
                {
                    i += 1;
                }
                if at(b, i) == b'}' {
                    i += 1;
                    tokens.push(Token {
                        kind: TokenKind::Variable,
                        start,
                        end: i,
                    });
                } else {
                    // Malformed variable — still mark it
                    tokens.push(Token {
                        kind: TokenKind::Variable,
                        start,
                        end: i,
                    });
                }
                continue;
            }

            // ── JSON attributes {{ ... }} ────────────────────────────
            if c == b'{' && at(b, i + 1) == b'{' {
                let start = i;
                i += 2;
                let mut depth = 1u32;
                while i < b.len() && depth > 0 {
                    if b[i] == b'{' && at(b, i + 1) == b'{' {
                        depth += 1;
                        i += 2;
                    } else if b[i] == b'}' && at(b, i + 1) == b'}' {
                        depth -= 1;
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                tokens.push(Token {
                    kind: TokenKind::String,
                    start,
                    end: i,
                });
                continue;
            }

            // ── Component tags <ComponentName ... /> or <ComponentName>...</ComponentName> ──
            if c == b'<' {
                let tag_start = i;
                let is_closing = at(b, i + 1) == b'/';
                let name_pos = if is_closing { i + 2 } else { i + 1 };

                // RDX components start with uppercase
                if is_upper(at(b, name_pos)) {
                    i = name_pos;
                    // Scan tag name: [A-Z][a-zA-Z0-9_]*
                    while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                        i += 1;
                    }
                    tokens.push(Token {
                        kind: TokenKind::Keyword,
                        start: tag_start,
                        end: i,
                    });

                    // For closing tags, just find >
                    if is_closing {
                        while i < b.len() && b[i] != b'>' {
                            i += 1;
                        }
                        if i < b.len() {
                            tokens.push(Token {
                                kind: TokenKind::Punctuation,
                                start: i,
                                end: i + 1,
                            });
                            i += 1;
                        }
                        continue;
                    }

                    // Scan attributes
                    scan_component_attrs(b, &mut i, &mut tokens);
                    continue;
                }

                // Lowercase HTML tags — same treatment as HTML scanner
                if at(b, name_pos).is_ascii_lowercase() {
                    i = name_pos;
                    while i < b.len()
                        && (b[i].is_ascii_alphanumeric() || b[i] == b'-' || b[i] == b'_')
                    {
                        i += 1;
                    }
                    tokens.push(Token {
                        kind: TokenKind::Attr,
                        start: tag_start,
                        end: i,
                    });

                    // Scan HTML attrs until >
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
                        if b[i].is_ascii_alphabetic()
                            || b[i] == b'-'
                            || b[i] == b'_'
                            || b[i] == b':'
                            || b[i] == b'@'
                        {
                            let as_ = i;
                            while i < b.len()
                                && (b[i].is_ascii_alphanumeric()
                                    || b[i] == b'-'
                                    || b[i] == b'_'
                                    || b[i] == b':'
                                    || b[i] == b'@')
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

            // ── Display math $$ ... $$ ───────────────────────────────
            if c == b'$' && at(b, i + 1) == b'$' {
                let start = i;
                i += 2;
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
                // Find closing $$
                if i < b.len() {
                    i += 1;
                }
                while i < b.len() {
                    let ls = i;
                    let mut ws = i;
                    while ws < b.len() && (b[ws] == b' ' || b[ws] == b'\t') {
                        ws += 1;
                    }
                    if at(b, ws) == b'$' && at(b, ws + 1) == b'$' {
                        i = ws + 2;
                        while i < b.len() && b[i] != b'\n' {
                            i += 1;
                        }
                        break;
                    }
                    while i < b.len() && b[i] != b'\n' {
                        i += 1;
                    }
                    if i < b.len() {
                        i += 1;
                    }
                    if i == ls {
                        break;
                    }
                }
                tokens.push(Token {
                    kind: TokenKind::String,
                    start,
                    end: i,
                });
                continue;
            }

            // ── Inline math $...$ ────────────────────────────────────
            if c == b'$' && at(b, i + 1) != b' ' && at(b, i + 1) != b'$' && at(b, i + 1) != 0 {
                let start = i;
                i += 1;
                while i < b.len() && b[i] != b'$' && b[i] != b'\n' {
                    i += 1;
                }
                if i < b.len() && b[i] == b'$' {
                    i += 1;
                    tokens.push(Token {
                        kind: TokenKind::String,
                        start,
                        end: i,
                    });
                    continue;
                }
                // Not a math expression, backtrack
                i = start;
            }

            // ── Markdown headings ────────────────────────────────────
            if was_line_start && c == b'#' {
                let start = i;
                while i < b.len() && b[i] == b'#' {
                    i += 1;
                }
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
                tokens.push(Token {
                    kind: TokenKind::Keyword,
                    start,
                    end: i,
                });
                line_start = true;
                continue;
            }

            // ── Inline code `...` ────────────────────────────────────
            if c == b'`' {
                let start = i;
                i += 1;
                while i < b.len() && b[i] != b'`' && b[i] != b'\n' {
                    i += 1;
                }
                if i < b.len() && b[i] == b'`' {
                    i += 1;
                }
                tokens.push(Token {
                    kind: TokenKind::String,
                    start,
                    end: i,
                });
                continue;
            }

            // ── Bold **...** / __...__  ──────────────────────────────
            if (c == b'*' && at(b, i + 1) == b'*') || (c == b'_' && at(b, i + 1) == b'_') {
                let delim = c;
                let start = i;
                i += 2;
                while i + 1 < b.len() && !(b[i] == delim && b[i + 1] == delim) {
                    if b[i] == b'\n' {
                        break;
                    }
                    i += 1;
                }
                if i + 1 < b.len() && b[i] == delim && b[i + 1] == delim {
                    i += 2;
                }
                tokens.push(Token {
                    kind: TokenKind::Keyword,
                    start,
                    end: i,
                });
                continue;
            }

            // ── Strikethrough ~~...~~ ────────────────────────────────
            if c == b'~' && at(b, i + 1) == b'~' {
                let start = i;
                i += 2;
                while i + 1 < b.len() && !(b[i] == b'~' && b[i + 1] == b'~') {
                    if b[i] == b'\n' {
                        break;
                    }
                    i += 1;
                }
                if i + 1 < b.len() && b[i] == b'~' && b[i + 1] == b'~' {
                    i += 2;
                }
                tokens.push(Token {
                    kind: TokenKind::Keyword,
                    start,
                    end: i,
                });
                continue;
            }

            // ── Italic *...* / _..._ ─────────────────────────────────
            if (c == b'*' || c == b'_') && at(b, i + 1) != b' ' {
                let delim = c;
                let start = i;
                i += 1;
                while i < b.len() && b[i] != delim && b[i] != b'\n' {
                    i += 1;
                }
                if i < b.len() && b[i] == delim {
                    i += 1;
                }
                tokens.push(Token {
                    kind: TokenKind::Keyword,
                    start,
                    end: i,
                });
                continue;
            }

            // ── Links [text](url) ────────────────────────────────────
            if c == b'[' {
                let start = i;
                i += 1;
                while i < b.len() && b[i] != b']' && b[i] != b'\n' {
                    i += 1;
                }
                if at(b, i) == b']' && at(b, i + 1) == b'(' {
                    i += 2;
                    while i < b.len() && b[i] != b')' && b[i] != b'\n' {
                        i += 1;
                    }
                    if at(b, i) == b')' {
                        i += 1;
                    }
                    tokens.push(Token {
                        kind: TokenKind::String,
                        start,
                        end: i,
                    });
                    continue;
                }
                // Footnote references [^1]
                if at(b, i) == b']' {
                    i += 1;
                    tokens.push(Token {
                        kind: TokenKind::Attr,
                        start,
                        end: i,
                    });
                    continue;
                }
            }

            // ── Images ![alt](url) ───────────────────────────────────
            if c == b'!' && at(b, i + 1) == b'[' {
                let start = i;
                i += 2;
                while i < b.len() && b[i] != b']' && b[i] != b'\n' {
                    i += 1;
                }
                if at(b, i) == b']' && at(b, i + 1) == b'(' {
                    i += 2;
                    while i < b.len() && b[i] != b')' && b[i] != b'\n' {
                        i += 1;
                    }
                    if at(b, i) == b')' {
                        i += 1;
                    }
                    tokens.push(Token {
                        kind: TokenKind::String,
                        start,
                        end: i,
                    });
                    continue;
                }
            }

            // ── Blockquote > ─────────────────────────────────────────
            if was_line_start && c == b'>' {
                tokens.push(Token {
                    kind: TokenKind::Keyword,
                    start: i,
                    end: i + 1,
                });
                i += 1;
                continue;
            }

            // ── Thematic break --- / *** / ___ ───────────────────────
            if was_line_start
                && matches!(c, b'-' | b'*' | b'_')
                && at(b, i + 1) == c
                && at(b, i + 2) == c
            {
                let start = i;
                while i < b.len() && (b[i] == c || b[i] == b' ') {
                    i += 1;
                }
                if i >= b.len() || b[i] == b'\n' {
                    tokens.push(Token {
                        kind: TokenKind::Keyword,
                        start,
                        end: i,
                    });
                    line_start = true;
                    continue;
                }
                i = start; // not a break
            }

            // ── Task list markers [ ] [x] ────────────────────────────
            // (these appear inside list items, handled by markdown rendering)

            i += 1;
        }

        tokens
    }
}

/// Scan a frontmatter line as YAML key: value.
fn scan_frontmatter_line(b: &[u8], i: &mut usize, tokens: &mut Vec<Token>) {
    // Skip leading whitespace
    while *i < b.len() && (b[*i] == b' ' || b[*i] == b'\t') {
        *i += 1;
    }

    if *i >= b.len() || b[*i] == b'\n' {
        if *i < b.len() {
            *i += 1;
        }
        return;
    }

    // Comment
    if b[*i] == b'#' {
        let start = *i;
        while *i < b.len() && b[*i] != b'\n' {
            *i += 1;
        }
        tokens.push(Token {
            kind: TokenKind::Comment,
            start,
            end: *i,
        });
        if *i < b.len() {
            *i += 1;
        }
        return;
    }

    // List item marker
    if b[*i] == b'-' && at(b, *i + 1) == b' ' {
        tokens.push(Token {
            kind: TokenKind::Punctuation,
            start: *i,
            end: *i + 1,
        });
        *i += 1;
        // Rest of line is a value
        while *i < b.len() && b[*i] == b' ' {
            *i += 1;
        }
        let val_start = *i;
        while *i < b.len() && b[*i] != b'\n' {
            *i += 1;
        }
        if *i > val_start {
            scan_yaml_value(b, val_start, *i, tokens);
        }
        if *i < b.len() {
            *i += 1;
        }
        return;
    }

    // Key: value
    let key_start = *i;
    while *i < b.len() && b[*i] != b':' && b[*i] != b'\n' {
        *i += 1;
    }
    if *i < b.len() && b[*i] == b':' {
        tokens.push(Token {
            kind: TokenKind::Property,
            start: key_start,
            end: *i,
        });
        tokens.push(Token {
            kind: TokenKind::Punctuation,
            start: *i,
            end: *i + 1,
        });
        *i += 1;
        while *i < b.len() && b[*i] == b' ' {
            *i += 1;
        }
        let val_start = *i;
        while *i < b.len() && b[*i] != b'\n' {
            *i += 1;
        }
        if *i > val_start {
            scan_yaml_value(b, val_start, *i, tokens);
        }
    } else {
        // Plain text line
        while *i < b.len() && b[*i] != b'\n' {
            *i += 1;
        }
    }
    if *i < b.len() {
        *i += 1;
    }
}

/// Classify a YAML value span.
fn scan_yaml_value(b: &[u8], start: usize, end: usize, tokens: &mut Vec<Token>) {
    let val = &b[start..end];
    // Trim trailing whitespace
    let mut e = val.len();
    while e > 0 && (val[e - 1] == b' ' || val[e - 1] == b'\t') {
        e -= 1;
    }
    let trimmed = &val[..e];

    if trimmed.is_empty() {
        return;
    }

    // Quoted string
    if (trimmed[0] == b'"' || trimmed[0] == b'\'') && e >= 2 && trimmed[e - 1] == trimmed[0] {
        tokens.push(Token {
            kind: TokenKind::String,
            start,
            end: start + e,
        });
        return;
    }

    // Boolean / null
    if matches!(
        trimmed,
        b"true"
            | b"false"
            | b"yes"
            | b"no"
            | b"null"
            | b"True"
            | b"False"
            | b"Yes"
            | b"No"
            | b"Null"
            | b"~"
    ) {
        tokens.push(Token {
            kind: TokenKind::Keyword,
            start,
            end: start + e,
        });
        return;
    }

    // Number
    if trimmed.iter().all(|&c| {
        c.is_ascii_digit()
            || c == b'.'
            || c == b'-'
            || c == b'+'
            || c == b'e'
            || c == b'E'
            || c == b'_'
    }) && trimmed.iter().any(|&c| c.is_ascii_digit())
    {
        tokens.push(Token {
            kind: TokenKind::Number,
            start,
            end: start + e,
        });
        return;
    }

    // Plain value
    tokens.push(Token {
        kind: TokenKind::Plain,
        start,
        end: start + e,
    });
}

/// Scan component attributes until `>` or `/>`.
fn scan_component_attrs(b: &[u8], i: &mut usize, tokens: &mut Vec<Token>) {
    while *i < b.len() && b[*i] != b'>' {
        if b[*i] == b' ' || b[*i] == b'\t' || b[*i] == b'\n' || b[*i] == b'\r' {
            *i += 1;
            continue;
        }

        // Self-closing />
        if b[*i] == b'/' && at(b, *i + 1) == b'>' {
            tokens.push(Token {
                kind: TokenKind::Punctuation,
                start: *i,
                end: *i + 2,
            });
            *i += 2;
            return;
        }

        // Attribute name
        if b[*i].is_ascii_alphabetic() || b[*i] == b'_' {
            let attr_start = *i;
            while *i < b.len() && (b[*i].is_ascii_alphanumeric() || b[*i] == b'_' || b[*i] == b'-')
            {
                *i += 1;
            }
            let attr_end = *i;

            // Check for = (no whitespace allowed around = in RDX)
            if at(b, *i) == b'=' {
                tokens.push(Token {
                    kind: TokenKind::Attr,
                    start: attr_start,
                    end: attr_end,
                });
                tokens.push(Token {
                    kind: TokenKind::Operator,
                    start: *i,
                    end: *i + 1,
                });
                *i += 1;

                // Value
                let vc = at(b, *i);

                // String value "..." or '...'
                if vc == b'"' || vc == b'\'' {
                    let vs = *i;
                    if let Some(end) = scan_quoted_string(b, *i, vc) {
                        *i = end;
                        tokens.push(Token {
                            kind: TokenKind::String,
                            start: vs,
                            end: *i,
                        });
                    }
                }
                // JSON attribute {{ ... }}
                else if vc == b'{' && at(b, *i + 1) == b'{' {
                    let vs = *i;
                    *i += 2;
                    let mut depth = 1u32;
                    while *i < b.len() && depth > 0 {
                        if b[*i] == b'{' && at(b, *i + 1) == b'{' {
                            depth += 1;
                            *i += 2;
                        } else if b[*i] == b'}' && at(b, *i + 1) == b'}' {
                            depth -= 1;
                            *i += 2;
                        } else {
                            *i += 1;
                        }
                    }
                    tokens.push(Token {
                        kind: TokenKind::String,
                        start: vs,
                        end: *i,
                    });
                }
                // Variable attribute {$var}
                else if vc == b'{' && at(b, *i + 1) == b'$' {
                    let vs = *i;
                    *i += 2;
                    while *i < b.len()
                        && (b[*i].is_ascii_alphanumeric() || b[*i] == b'_' || b[*i] == b'.')
                    {
                        *i += 1;
                    }
                    if at(b, *i) == b'}' {
                        *i += 1;
                    }
                    tokens.push(Token {
                        kind: TokenKind::Variable,
                        start: vs,
                        end: *i,
                    });
                }
                // Primitive literal {true}, {false}, {null}, {42}, {-2.5}
                else if vc == b'{' {
                    let vs = *i;
                    *i += 1;
                    while *i < b.len() && b[*i] != b'}' && b[*i] != b'\n' {
                        *i += 1;
                    }
                    if at(b, *i) == b'}' {
                        *i += 1;
                    }
                    // Classify the content
                    let content = &b[vs + 1..*i - 1];
                    let trimmed = content
                        .iter()
                        .copied()
                        .skip_while(|&c| c == b' ')
                        .collect::<Vec<_>>();
                    let kind = if matches!(trimmed.as_slice(), b"true" | b"false" | b"null") {
                        TokenKind::Keyword
                    } else {
                        TokenKind::Number
                    };
                    tokens.push(Token {
                        kind,
                        start: vs,
                        end: *i,
                    });
                }
            } else {
                // Boolean shorthand (attribute without value)
                tokens.push(Token {
                    kind: TokenKind::Attr,
                    start: attr_start,
                    end: attr_end,
                });
            }
            continue;
        }

        *i += 1;
    }

    // Closing >
    if *i < b.len() && b[*i] == b'>' {
        tokens.push(Token {
            kind: TokenKind::Punctuation,
            start: *i,
            end: *i + 1,
        });
        *i += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::render;

    fn hl(code: &str) -> String {
        render(code, &RdxScanner.scan(code))
    }

    #[test]
    fn frontmatter() {
        let code = "---\ntitle: Hello\nversion: 1.0\n---\n";
        let out = hl(code);
        assert!(out.contains("tok-keyword")); // ---
        assert!(out.contains("tok-property")); // title, version
        assert!(out.contains("tok-number")); // 1.0
    }

    #[test]
    fn component_self_closing() {
        let out = hl(r#"<Button label="Click" />"#);
        assert!(out.contains("tok-keyword")); // <Button
        assert!(out.contains("tok-attr")); // label
        assert!(out.contains("tok-string")); // "Click"
    }

    #[test]
    fn component_block() {
        let out = hl("<Notice type=\"warning\">\nContent\n</Notice>");
        assert_eq!(out.matches("tok-keyword").count(), 2); // open + close tags
    }

    #[test]
    fn variable_interpolation() {
        let out = hl("Hello {$user.name}!");
        assert!(out.contains("tok-variable"));
    }

    #[test]
    fn variable_in_attr() {
        let out = hl(r#"<Comp data={$frontmatter.title} />"#);
        assert!(out.contains("tok-variable"));
    }

    #[test]
    fn json_attr() {
        let out = hl(r#"<Chart config={{"type": "bar"}} />"#);
        assert!(out.contains("tok-string")); // JSON block
    }

    #[test]
    fn primitive_attr() {
        let out = hl("<Slider min={0} max={100} active={true} />");
        assert!(out.contains("tok-number")); // {0}, {100}
        assert!(out.contains("tok-keyword")); // {true}
    }

    #[test]
    fn boolean_shorthand() {
        let out = hl("<Input disabled />");
        assert!(out.contains("tok-attr")); // disabled
    }

    #[test]
    fn inline_math() {
        let out = hl("The equation $x^2 + y^2$ is well known.");
        assert!(out.contains("tok-string"));
    }

    #[test]
    fn display_math() {
        let out = hl("$$\nE = mc^2\n$$");
        assert!(out.contains("tok-string"));
    }

    #[test]
    fn html_comment() {
        let out = hl("<!-- This is a comment -->");
        assert!(out.contains("tok-comment"));
    }

    #[test]
    fn escape_sequences() {
        let out = hl(r"\{$not_a_var}");
        assert!(out.contains("tok-operator")); // \{
        assert!(!out.contains("tok-variable"));
    }

    #[test]
    fn heading() {
        let out = hl("# Hello World");
        assert!(out.contains("tok-keyword"));
    }

    #[test]
    fn code_fence_not_tokenized() {
        let code = "```rust\nlet x = 1;\n```";
        let out = hl(code);
        // Code content should NOT be tokenized with Rust tokens
        assert!(!out.contains("tok-keyword"));
        assert!(out.contains("tok-string")); // fence markers
    }

    #[test]
    fn inline_code() {
        let out = hl("`{$title}` is literal");
        assert!(out.contains("tok-string"));
        assert!(!out.contains("tok-variable")); // not interpolated in code
    }

    #[test]
    fn bold_and_italic() {
        assert!(hl("**bold**").contains("tok-keyword"));
        assert!(hl("*italic*").contains("tok-keyword"));
    }

    #[test]
    fn strikethrough() {
        assert!(hl("~~deleted~~").contains("tok-keyword"));
    }

    #[test]
    fn link() {
        assert!(hl("[text](url)").contains("tok-string"));
    }

    #[test]
    fn html_tag() {
        let out = hl("<div class=\"main\">");
        assert!(out.contains("tok-attr")); // tag + attribute
    }

    #[test]
    fn full_document() {
        let code = r#"---
title: Test Doc
version: 1.0
---

# {$title}

<Notice type="warning">
  **Important:** This is a notice.
</Notice>

The equation $E = mc^2$ is famous.

```rust
fn main() {}
```

<DataTable
  title="Users"
  page={1}
  sortable
  columns={{["Name", "Email"]}}
  data={$table_data}
/>

Escaped: \{$not_a_var}
"#;
        let out = hl(code);
        assert!(out.contains("tok-keyword")); // ---, headings, components
        assert!(out.contains("tok-property")); // frontmatter keys
        assert!(out.contains("tok-variable")); // {$title}, {$table_data}
        assert!(out.contains("tok-attr")); // component attrs
        assert!(out.contains("tok-string")); // strings, math, code fence
        assert!(out.contains("tok-number")); // 1.0, {1}
        assert!(out.contains("tok-operator")); // escape \{
    }

    #[test]
    fn no_panic_on_garbage() {
        let _ = hl("}{][)(///**/\"\"'''```\n\r\t\x00🎉{$}{$.");
    }
}
