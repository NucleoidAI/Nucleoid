//! The build script's statement splitter, shared with its integration tests.

/// Splits a case into top-level statements while retaining comments and blocks.
pub(crate) fn statements(source: &str) -> Vec<String> {
    let source = source.replace("\r\n", "\n");
    let mut statements = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    let mut pending: Vec<&str> = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim();
        let top_level = line.len() == line.trim_start().len();
        let comment_or_blank = trimmed.is_empty() || trimmed.starts_with('#');
        // Raw::Word in the lexer is [A-Za-z_][A-Za-z0-9_]*. Keep either
        // complete Else alias with its chain, not an identifier prefix.
        // Header validity remains the parser's job, not a second grammar here.
        let continuation = ["else", "elif"].iter().any(|keyword| {
            trimmed.strip_prefix(keyword).is_some_and(|tail| {
                !tail.starts_with(|character: char| {
                    character.is_ascii_alphanumeric() || character == '_'
                })
            })
        }) || trimmed.starts_with("catch ")
            || trimmed.starts_with("finally:")
            || trimmed.starts_with([')', ']', '}']);

        if comment_or_blank {
            pending.push(line);
        } else if top_level && !continuation {
            if !current.is_empty() {
                statements.push(current.join("\n").trim().to_string());
                current.clear();
            }

            current.append(&mut pending);
            current.push(line);
        } else {
            current.append(&mut pending);
            current.push(line);
        }
    }

    if !current.is_empty() {
        statements.push(current.join("\n").trim().to_string());
    }

    assert!(!statements.is_empty(), "a dataset record has no statements");
    statements
}
