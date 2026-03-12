/// Token kind produced by scanners.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Keyword,
    String,
    Comment,
    Number,
    Function,
    Type,
    Operator,
    Punctuation,
    Property,
    Builtin,
    Attr,
    Variable,
    Plain,
}

impl TokenKind {
    /// CSS class name for this token kind, or `None` for `Plain`.
    pub fn class(self) -> Option<&'static str> {
        match self {
            Self::Keyword => Some("tok-keyword"),
            Self::String => Some("tok-string"),
            Self::Comment => Some("tok-comment"),
            Self::Number => Some("tok-number"),
            Self::Function => Some("tok-function"),
            Self::Type => Some("tok-type"),
            Self::Operator => Some("tok-operator"),
            Self::Punctuation => Some("tok-punctuation"),
            Self::Property => Some("tok-property"),
            Self::Builtin => Some("tok-builtin"),
            Self::Attr => Some("tok-attr"),
            Self::Variable => Some("tok-variable"),
            Self::Plain => None,
        }
    }
}

/// A token referencing a byte range in the source string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub start: usize,
    pub end: usize,
}

/// Render a token list to HTML. `code` is the original source.
pub fn render(code: &str, tokens: &[Token]) -> String {
    use crate::escape::escape_html;

    let mut out = String::with_capacity(code.len() * 2);
    let mut pos = 0;

    for tok in tokens {
        // Emit any gap as plain text
        if tok.start > pos {
            escape_html(&code[pos..tok.start], &mut out);
        }
        let text = &code[tok.start..tok.end];
        if let Some(cls) = tok.kind.class() {
            out.push_str("<span class=\"");
            out.push_str(cls);
            out.push_str("\">");
            escape_html(text, &mut out);
            out.push_str("</span>");
        } else {
            escape_html(text, &mut out);
        }
        pos = tok.end;
    }

    // Trailing plain text
    if pos < code.len() {
        escape_html(&code[pos..], &mut out);
    }

    out
}
