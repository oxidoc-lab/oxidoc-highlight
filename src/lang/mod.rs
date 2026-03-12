mod rust;
mod javascript;
mod python;
mod bash;
mod html;
mod css;
mod json;
mod toml;
mod yaml;
mod go;
mod c;
mod java;
mod php;
mod sql;
mod markdown;
mod rdx;
mod xml;
mod diff;

use crate::scanner::Scanner;
use crate::token::Token;

/// Get a scanner for the given language name. Returns `None` for unknown languages.
pub fn get_scanner(lang: &str) -> Option<Box<dyn Scanner>> {
    match lang {
        "rust" | "rs" => Some(Box::new(rust::RustScanner)),
        "javascript" | "js" => Some(Box::new(javascript::JsScanner { typescript: false, jsx: false })),
        "jsx" => Some(Box::new(javascript::JsScanner { typescript: false, jsx: true })),
        "typescript" | "ts" => Some(Box::new(javascript::JsScanner { typescript: true, jsx: false })),
        "tsx" => Some(Box::new(javascript::JsScanner { typescript: true, jsx: true })),
        "python" | "py" => Some(Box::new(python::PythonScanner)),
        "bash" | "sh" | "shell" | "zsh" => Some(Box::new(bash::BashScanner)),
        "html" | "vue" | "svelte" => Some(Box::new(html::HtmlScanner)),
        "css" | "scss" | "less" => Some(Box::new(css::CssScanner)),
        "json" | "jsonc" | "json5" => Some(Box::new(json::JsonScanner)),
        "toml" => Some(Box::new(toml::TomlScanner)),
        "yaml" | "yml" => Some(Box::new(yaml::YamlScanner)),
        "go" | "golang" => Some(Box::new(go::GoScanner)),
        "c" | "h" => Some(Box::new(c::CScanner { cpp: false })),
        "cpp" | "c++" | "cxx" | "cc" | "hpp" | "hxx" => Some(Box::new(c::CScanner { cpp: true })),
        "java" | "kotlin" | "kt" => Some(Box::new(java::JavaScanner)),
        "php" => Some(Box::new(php::PhpScanner)),
        "sql" | "mysql" | "postgresql" | "sqlite" => Some(Box::new(sql::SqlScanner)),
        "markdown" | "md" => Some(Box::new(markdown::MarkdownScanner)),
        "rdx" => Some(Box::new(rdx::RdxScanner)),
        "xml" | "svg" | "xhtml" | "xsl" => Some(Box::new(xml::XmlScanner)),
        "diff" | "patch" => Some(Box::new(diff::DiffScanner)),
        _ => None,
    }
}

/// List of all supported language names.
pub fn supported() -> Vec<&'static str> {
    vec![
        "rust", "rs",
        "javascript", "js", "jsx",
        "typescript", "ts", "tsx",
        "python", "py",
        "bash", "sh", "shell", "zsh",
        "html", "vue", "svelte",
        "css", "scss", "less",
        "json", "jsonc", "json5",
        "toml",
        "yaml", "yml",
        "go", "golang",
        "c", "h", "cpp", "c++", "cxx", "cc", "hpp", "hxx",
        "java", "kotlin", "kt",
        "php",
        "sql", "mysql", "postgresql", "sqlite",
        "markdown", "md", "rdx",
        "xml", "svg", "xhtml", "xsl",
        "diff", "patch",
    ]
}

/// Scan code with the given language, or return empty tokens for unknown languages.
pub fn scan(code: &str, lang: &str) -> Vec<Token> {
    match get_scanner(lang) {
        Some(scanner) => scanner.scan(code),
        None => vec![],
    }
}
