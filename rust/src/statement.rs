use regex::Regex;
use std::sync::OnceLock;

/// The original parses source into an ESTree program via acorn and walks
/// individual top-level statements (`src/statement.js`, `src/stack.js`).
/// We don't have an ESTree walker in Rust, so this splits on a
/// brace/string-aware scan instead: good enough for sequential statements,
/// `if`/`for`/`function`/`class` blocks, but not a full JS parser.
pub fn split(source: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut buf = String::new();
    let mut depth: i32 = 0;
    let mut quote: Option<char> = None;
    let mut escaped = false;

    for c in source.chars() {
        buf.push(c);

        if let Some(q) = quote {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == q {
                quote = None;
            }
            continue;
        }

        match c {
            '\'' | '"' | '`' => quote = Some(c),
            '{' | '(' | '[' => depth += 1,
            '}' if depth == 1 => {
                depth = 0;
                // A block-shaped statement (function/if/for/class/{...}) is
                // complete as soon as its outermost brace closes — it won't
                // necessarily be followed by a `;`.
                let trimmed = buf.trim();
                if !trimmed.is_empty() {
                    statements.push(trimmed.to_string());
                }
                buf.clear();
            }
            '}' | ')' | ']' => depth -= 1,
            ';' if depth == 0 => {
                let trimmed = buf.trim_end_matches(';').trim();
                if !trimmed.is_empty() {
                    statements.push(trimmed.to_string());
                }
                buf.clear();
            }
            _ => {}
        }
    }

    let trimmed = buf.trim();
    if !trimmed.is_empty() {
        statements.push(trimmed.to_string());
    }

    statements
}

/// Result of classifying one statement for dependency-graph purposes.
/// Everything still gets executed as real JS via `Engine::eval` regardless
/// of how it's classified here — this only decides whether we also track it
/// as a reactive variable (`src/lang/$nuc/$LET.js`, `$ASSIGNMENT.js`).
pub enum Parsed<'a> {
    Assignment {
        declared: bool,
        name: String,
        rhs: &'a str,
    },
    Other,
}

pub fn parse(statement: &str) -> Parsed<'_> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        Regex::new(r"^(?:(let|const|var)\s+)?([A-Za-z_$][A-Za-z0-9_$]*)\s*=\s*([^=].*)$")
            .expect("valid regex")
    });

    match re.captures(statement) {
        Some(caps) => Parsed::Assignment {
            declared: caps.get(1).is_some(),
            name: caps[2].to_string(),
            rhs: caps.get(3).map(|m| m.as_str()).unwrap_or_default(),
        },
        None => Parsed::Other,
    }
}

/// Identifiers referenced in an expression that are also known state
/// variables — a heuristic stand-in for the original's ESTree-based
/// dependency graph (`src/graph.js`, `src/Expression.js`).
pub fn referenced_vars(expr: &str, known: &std::collections::HashSet<String>) -> Vec<String> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"[A-Za-z_$][A-Za-z0-9_$]*").expect("valid regex"));

    let mut found = Vec::new();
    for m in re.find_iter(expr) {
        let ident = m.as_str();
        if known.contains(ident) && !found.contains(&ident.to_string()) {
            found.push(ident.to_string());
        }
    }
    found
}
