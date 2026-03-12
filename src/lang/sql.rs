use crate::scanner::*;
use crate::token::{Token, TokenKind};

pub struct SqlScanner;

const KEYWORDS: &[&[u8]] = &[
    b"SELECT", b"FROM", b"WHERE", b"INSERT", b"INTO", b"UPDATE", b"DELETE", b"CREATE",
    b"DROP", b"ALTER", b"TABLE", b"INDEX", b"VIEW", b"JOIN", b"INNER", b"LEFT",
    b"RIGHT", b"OUTER", b"FULL", b"CROSS", b"ON", b"AND", b"OR", b"NOT", b"IN",
    b"EXISTS", b"BETWEEN", b"LIKE", b"IS", b"NULL", b"AS", b"ORDER", b"BY",
    b"GROUP", b"HAVING", b"LIMIT", b"OFFSET", b"UNION", b"ALL", b"DISTINCT",
    b"SET", b"VALUES", b"PRIMARY", b"KEY", b"FOREIGN", b"REFERENCES", b"CASCADE",
    b"CONSTRAINT", b"DEFAULT", b"CHECK", b"UNIQUE", b"IF", b"ELSE", b"THEN",
    b"END", b"CASE", b"WHEN", b"BEGIN", b"COMMIT", b"ROLLBACK", b"TRANSACTION",
    b"ASC", b"DESC", b"TRUE", b"FALSE", b"WITH", b"RECURSIVE", b"RETURNING",
    // Also lowercase
    b"select", b"from", b"where", b"insert", b"into", b"update", b"delete", b"create",
    b"drop", b"alter", b"table", b"index", b"view", b"join", b"inner", b"left",
    b"right", b"outer", b"full", b"cross", b"on", b"and", b"or", b"not", b"in",
    b"exists", b"between", b"like", b"is", b"null", b"as", b"order", b"by",
    b"group", b"having", b"limit", b"offset", b"union", b"all", b"distinct",
    b"set", b"values", b"primary", b"key", b"foreign", b"references", b"cascade",
    b"constraint", b"default", b"check", b"unique", b"if", b"else", b"then",
    b"end", b"case", b"when", b"begin", b"commit", b"rollback", b"transaction",
    b"asc", b"desc", b"true", b"false", b"with", b"recursive", b"returning",
];

const TYPES: &[&[u8]] = &[
    b"INT", b"INTEGER", b"BIGINT", b"SMALLINT", b"TINYINT", b"SERIAL", b"BIGSERIAL",
    b"FLOAT", b"REAL", b"DOUBLE", b"DECIMAL", b"NUMERIC", b"BOOLEAN", b"BOOL",
    b"VARCHAR", b"CHAR", b"TEXT", b"BLOB", b"DATE", b"TIME", b"TIMESTAMP",
    b"DATETIME", b"JSON", b"JSONB", b"UUID", b"BYTEA", b"ARRAY",
    b"int", b"integer", b"bigint", b"smallint", b"tinyint", b"serial", b"bigserial",
    b"float", b"real", b"double", b"decimal", b"numeric", b"boolean", b"bool",
    b"varchar", b"char", b"text", b"blob", b"date", b"time", b"timestamp",
    b"datetime", b"json", b"jsonb", b"uuid", b"bytea", b"array",
];

const BUILTINS: &[&[u8]] = &[
    b"COUNT", b"SUM", b"AVG", b"MIN", b"MAX", b"COALESCE", b"NULLIF",
    b"CAST", b"CONVERT", b"CONCAT", b"LENGTH", b"UPPER", b"LOWER", b"TRIM",
    b"SUBSTRING", b"REPLACE", b"NOW", b"CURRENT_TIMESTAMP", b"EXTRACT",
    b"count", b"sum", b"avg", b"min", b"max", b"coalesce", b"nullif",
    b"cast", b"convert", b"concat", b"length", b"upper", b"lower", b"trim",
    b"substring", b"replace", b"now", b"current_timestamp", b"extract",
];

fn at(b: &[u8], i: usize) -> u8 {
    if i < b.len() { b[i] } else { 0 }
}

impl Scanner for SqlScanner {
    fn scan(&self, code: &str) -> Vec<Token> {
        let b = code.as_bytes();
        let mut tokens = Vec::new();
        let mut i = 0;

        while i < b.len() {
            let c = b[i];
            if c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' { i += 1; continue; }

            // Line comments --
            if c == b'-' && at(b, i + 1) == b'-' {
                if let Some(end) = scan_line_comment(b, i, b"--") {
                    tokens.push(Token { kind: TokenKind::Comment, start: i, end });
                    i = end;
                    continue;
                }
            }
            if let Some(end) = scan_block_comment(b, i) { tokens.push(Token { kind: TokenKind::Comment, start: i, end }); i = end; continue; }

            if c == b'\'' { if let Some(end) = scan_single_string(b, i) { tokens.push(Token { kind: TokenKind::String, start: i, end }); i = end; continue; } }
            if c == b'"' { if let Some(end) = scan_double_string(b, i) { tokens.push(Token { kind: TokenKind::String, start: i, end }); i = end; continue; } }

            if let Some(end) = scan_number(b, i) { tokens.push(Token { kind: TokenKind::Number, start: i, end }); i = end; continue; }

            if let Some((end, ident)) = scan_ident(b, i) {
                let kind = if is_keyword(ident, KEYWORDS) { TokenKind::Keyword }
                    else if is_keyword(ident, TYPES) { TokenKind::Type }
                    else if is_keyword(ident, BUILTINS) && is_function_call(b, end) { TokenKind::Builtin }
                    else if is_function_call(b, end) { TokenKind::Function }
                    else { TokenKind::Plain };
                tokens.push(Token { kind, start: i, end });
                i = end;
                continue;
            }

            if let Some(end) = scan_operator(b, i) { tokens.push(Token { kind: TokenKind::Operator, start: i, end }); i = end; continue; }
            if let Some(end) = scan_punctuation(b, i) { tokens.push(Token { kind: TokenKind::Punctuation, start: i, end }); i = end; continue; }
            i += 1;
        }
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::render;
    fn hl(code: &str) -> String { render(code, &SqlScanner.scan(code)) }

    #[test]
    fn select() { assert!(hl("SELECT * FROM users").contains("tok-keyword")); }
    #[test]
    fn string() { assert!(hl("WHERE name = 'foo'").contains("tok-string")); }
    #[test]
    fn comment() { assert!(hl("-- comment").contains("tok-comment")); }
    #[test]
    fn number() { assert!(hl("LIMIT 10").contains("tok-number")); }
    #[test]
    fn builtin() { assert!(hl("COUNT(*)").contains("tok-builtin")); }
}
