pub mod escape;
pub mod token;
pub mod scanner;
mod lang;

use token::render;

/// Highlight `code` in the given language, returning HTML with `<span class="tok-*">` tokens.
///
/// Unknown languages return HTML-escaped plain text (no spans).
/// Empty input returns an empty string.
pub fn highlight(code: &str, lang: &str) -> String {
    if code.is_empty() {
        return String::new();
    }
    let tokens = lang::scan(code, lang);
    render(code, &tokens)
}

/// List all supported language identifiers.
pub fn supported_languages() -> Vec<&'static str> {
    lang::supported()
}

/// Check if a language is supported.
pub fn is_supported(lang: &str) -> bool {
    lang::get_scanner(lang).is_some()
}

// ── Wasm bindings ────────────────────────────────────────────────────

#[cfg(feature = "wasm")]
mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(js_name = "highlight")]
    pub fn highlight_wasm(code: &str, lang: &str) -> String {
        crate::highlight(code, lang)
    }

    #[wasm_bindgen(js_name = "supportedLanguages")]
    pub fn supported_languages_wasm() -> Vec<String> {
        crate::supported_languages().into_iter().map(String::from).collect()
    }

    #[wasm_bindgen(js_name = "isSupported")]
    pub fn is_supported_wasm(lang: &str) -> bool {
        crate::is_supported(lang)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input() {
        assert_eq!(highlight("", "rust"), "");
    }

    #[test]
    fn unknown_language_escapes_html() {
        assert_eq!(highlight("<b>hi</b>", "unknown"), "&lt;b&gt;hi&lt;/b&gt;");
    }

    #[test]
    fn round_trip_strip_spans() {
        let code = "let x = 42; // test\nfn foo() {}";
        let html = highlight(code, "rust");
        // Strip all spans
        let stripped = html
            .replace(|_: char| false, "") // no-op
            .split("<span")
            .map(|s| {
                if let Some(pos) = s.find('>') {
                    &s[pos + 1..]
                } else {
                    s
                }
            })
            .collect::<Vec<_>>()
            .join("")
            .replace("</span>", "");
        // The stripped text should equal the HTML-escaped original
        let mut expected = String::new();
        crate::escape::escape_html(code, &mut expected);
        assert_eq!(stripped, expected);
    }

    #[test]
    fn supported_languages_nonempty() {
        assert!(supported_languages().len() > 10);
    }

    #[test]
    fn is_supported_works() {
        assert!(is_supported("rust"));
        assert!(is_supported("rs"));
        assert!(is_supported("javascript"));
        assert!(!is_supported("brainfuck"));
    }

    #[test]
    fn unicode_no_panic() {
        // Should not panic on unicode input
        let _ = highlight("let 变量 = \"你好世界\";", "rust");
        let _ = highlight("const 🎉 = true;", "javascript");
    }

    #[test]
    fn crlf_handling() {
        let code = "let x = 1;\r\nlet y = 2;\r\n";
        let html = highlight(code, "rust");
        assert!(html.contains("tok-keyword"));
        assert!(html.contains("\r\n"));
    }

    #[test]
    fn spec_example() {
        let out = highlight("let x = 42;", "rust");
        assert!(out.contains("<span class=\"tok-keyword\">let</span>"));
        assert!(out.contains("<span class=\"tok-operator\">=</span>"));
        assert!(out.contains("<span class=\"tok-number\">42</span>"));
        assert!(out.contains("<span class=\"tok-punctuation\">;</span>"));
    }
}
