//! The build script's statement splitter, shared with its integration tests.

/// The exporter's existing balanced/quoted argument scanner, shared by direct,
/// caught and scoped checks. This recognizes source spellings, not a new AST.
#[allow(dead_code)] // Splitter-only integration tests also include this module.
pub(crate) fn assertion(statement: &str) -> Option<(&str, &str)> {
    let statement = statement.trim();
    let tail = statement.strip_prefix("assert")?.trim_start();
    let arguments = if let Some(arguments) = tail.strip_prefix(':') {
        arguments
    } else {
        tail.strip_prefix('(')?.strip_suffix(')')?
    };
    let mut depth = 0;
    let mut quote = None;
    let mut escaped = false;

    for (index, character) in arguments.char_indices() {
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == delimiter {
                quote = None;
            }

            continue;
        }

        match character {
            '"' | '\'' | '`' => quote = Some(character),
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                let actual = arguments[..index].trim();
                let expected = arguments[index + character.len_utf8()..].trim();
                return Some((actual, expected));
            }
            _ => {}
        }
    }

    None
}

/// Match a complete ASCII Word keyword, not an ordinary identifier prefix.
/// This selects export boundaries; only the real parser validates the source.
pub(crate) fn starts_keyword(source: &str, keyword: &str) -> bool {
    source.strip_prefix(keyword).is_some_and(|tail| {
        !tail.starts_with(|character: char| character.is_ascii_alphanumeric() || character == '_')
    })
}

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
        // complete Else alias or Catch with its chain, not an identifier prefix.
        // Header validity remains the parser's job, not a second grammar here.
        let continuation = ["else", "elif", "catch"]
            .iter()
            .any(|keyword| starts_keyword(trimmed, keyword))
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
