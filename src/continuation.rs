use nucleoid::lang::ast::Quantifier;
use nucleoid::lang::ast::lexer::{Keyword, Spanned, Token, tokenize};
use nucleoid::{ErrorKind, Position};

use super::INDENT;

#[derive(Default)]
pub(super) struct Pending {
    pub(super) source: String,
    multiline: bool,
    pub(super) indentation: String,
}

impl Pending {
    pub(super) fn accept(&mut self, line: &str) -> Option<String> {
        if line.trim().is_empty() {
            if unterminated_block_comment(&self.source) || opens_function_body(&self.source) {
                self.source.push_str(line);
                self.source.push('\n');
                return None;
            }
            return (!self.source.is_empty()).then(|| self.take());
        }

        // A block comment may span physical reads. Inspect the complete pending
        // source when it closes, rather than treating its contents as headers.
        let continued;
        let header = if unterminated_block_comment(&self.source) {
            continued = format!("{}{line}", self.source);
            continued.as_str()
        } else {
            line
        };
        let opens_body = indented_body_header(header);
        self.multiline |= opens_body;
        let indentation_hint = next_indentation(header, opens_body);
        self.source.push_str(line);
        self.source.push('\n');

        if unterminated_block_comment(&self.source) {
            self.indentation.clear();
            return None;
        }
        if self.multiline || opens_function_body(&self.source) {
            self.indentation = indentation_hint;
            return None;
        }
        Some(self.take())
    }

    fn take(&mut self) -> String {
        self.multiline = false;
        self.indentation.clear();
        std::mem::take(&mut self.source)
    }
}

pub(super) fn unterminated_block_comment(source: &str) -> bool {
    tokenize(source).is_err_and(|error| {
        error.kind() == ErrorKind::Syntax && error.message() == "Unterminated block comment"
    })
}

fn opens_function_body(source: &str) -> bool {
    let Ok(tokens) = tokenize(source) else {
        return false;
    };
    let mut tokens = header_tokens(&tokens).peekable();
    let mut previous = None;
    let mut declares = false;
    let mut opened = 0usize;
    let mut closed = 0usize;
    while let Some(token) = tokens.next() {
        match token {
            Token::Keyword(Keyword::Function) if previous != Some(&Token::Dot) => {
                declares |= matches!(tokens.peek(), Some(Token::Identifier(_)));
            }
            Token::LeftBrace => opened += 1,
            Token::RightBrace => closed += 1,
            _ => {}
        }
        previous = Some(token);
    }
    declares && opened > closed
}

fn is_layout(token: &Token) -> bool {
    matches!(
        token,
        Token::Newline | Token::Indent | Token::Dedent | Token::Eof
    )
}

/// Use the last logical code line, not text inside a trailing multiline
/// comment.
fn lex_header(source: &str) -> nucleoid::Result<Vec<Spanned>> {
    let mut tokens = tokenize(source)?;
    if let Some(last) = tokens
        .iter()
        .rposition(|spanned| !is_layout(&spanned.token))
    {
        let mut start = tokens
            .iter()
            .take(last)
            .rposition(|spanned| spanned.token == Token::Newline)
            .map_or(0, |index| index + 1);
        while start < last
            && tokens
                .get(start)
                .is_some_and(|spanned| is_layout(&spanned.token))
        {
            start += 1;
        }
        tokens.drain(..start);
    }
    Ok(tokens)
}

pub(super) fn indented_body_header(line: &str) -> bool {
    if indented_named_arrow_header(line)
        || indented_try_header(line)
        || indented_condition_header(line, Keyword::If)
    {
        return true;
    }
    let Ok(tokens) = lex_header(line) else {
        return false;
    };
    let mut tokens = header_tokens(&tokens);
    let first = tokens.next();
    let last = tokens.next_back();
    matches!(last, Some(Token::Colon))
        || (matches!(first, Some(Token::Keyword(Keyword::Class)))
            && matches!(last, Some(Token::Identifier(_)))
            && matches!(tokens.next_back(), Some(Token::Colon)))
}

pub(super) fn next_indentation(source: &str, opens_body: bool) -> String {
    let code_position = lex_header(source).ok().and_then(|tokens| {
        let last = tokens
            .iter()
            .rev()
            .find(|spanned| !is_layout(&spanned.token))?;
        tokens
            .iter()
            .find(|spanned| spanned.position.line == last.position.line)
            .map(|spanned| spanned.position)
    });
    let mut indentation: String = match code_position {
        Some(position) => comment_free_indentation(source, position),
        None => source
            .lines()
            .next_back()
            .unwrap_or_default()
            .chars()
            .take_while(|character| matches!(character, ' ' | '\t'))
            .collect(),
    };
    // A pasted, balanced conditional header may span several physical lines.
    if opens_body || indented_alternative_header(source) {
        indentation.push_str(INDENT);
    }
    indentation
}

/// Before the first code token, only whitespace and comments can contribute
/// layout. Copy real indentation, never the columns occupied by a comment.
fn comment_free_indentation(source: &str, position: Position) -> String {
    let start: usize = source
        .split_inclusive('\n')
        .take(position.line - 1)
        .map(str::len)
        .sum();
    let prefix: String = source
        .get(start..)
        .unwrap_or_default()
        .chars()
        .take(position.column - 1)
        .collect();
    let mut remainder = prefix.as_str();
    if unterminated_block_comment(source.get(..start).unwrap_or_default()) {
        remainder = remainder.split_once("*/").map_or("", |(_, tail)| tail);
    }
    let mut indentation = String::new();
    while !remainder.is_empty() {
        if let Some(body) = remainder.strip_prefix("/*") {
            remainder = body.split_once("*/").map_or("", |(_, tail)| tail);
        } else {
            let mut characters = remainder.chars();
            let Some(character) = characters.next() else {
                break;
            };
            if !matches!(character, ' ' | '\t' | '\r' | '\u{c}') {
                break;
            }
            if matches!(character, ' ' | '\t') {
                indentation.push(character);
            }
            remainder = characters.as_str();
        }
    }
    indentation
}

fn indented_alternative_header(line: &str) -> bool {
    if indented_condition_header(line, Keyword::Else) {
        return true;
    }
    let Ok(tokens) = lex_header(line) else {
        return false;
    };
    let mut tokens = header_tokens(&tokens);
    match tokens.next() {
        Some(Token::Keyword(Keyword::Else)) => tokens.next().is_none(),
        Some(Token::Keyword(Keyword::Catch)) => match tokens.next() {
            None => true,
            Some(Token::Identifier(_)) => tokens.next().is_none(),
            Some(Token::LeftParen) => {
                matches!(tokens.next(), Some(Token::Identifier(_)))
                    && matches!(tokens.next(), Some(Token::RightParen))
                    && tokens.next().is_none()
            }
            _ => false,
        },
        _ => false,
    }
}

/// Buffer only a lexical, nonempty balanced condition without a suite delimiter.
/// A top-level conditional expression's colon pairs with its `?`; it is not a
/// suite delimiter. Alternatives only adjust indentation inside an existing
/// submission; they do not start one. This is not expression validation:
/// commit/EOF still runs the unchanged real parser, including its body
/// requirements and resource bounds.
fn indented_condition_header(line: &str, keyword: Keyword) -> bool {
    let Ok(tokens) = lex_header(line) else {
        return false;
    };
    let mut tokens = tokens.iter().map(|spanned| &spanned.token).peekable();
    if tokens.next() != Some(&Token::Keyword(keyword)) {
        return false;
    }
    if keyword == Keyword::Else && tokens.peek() == Some(&&Token::Keyword(Keyword::If)) {
        tokens.next();
    }
    let mut nonempty = false;
    let mut delimiters = Vec::new();
    let mut conditionals = 0usize;
    while let Some(token) = tokens.next() {
        match token {
            Token::At => match tokens.next() {
                Some(Token::VersionIndex(_)) => {}
                Some(Token::Identifier(name))
                    if matches!(name.as_str(), "first" | "last" | "previous") => {}
                _ => return false,
            },
            Token::Eof => return nonempty && delimiters.is_empty() && conditionals == 0,
            Token::Newline | Token::Dedent if delimiters.is_empty() => {
                return nonempty
                    && conditionals == 0
                    && tokens
                        .all(|rest| matches!(rest, Token::Newline | Token::Dedent | Token::Eof));
            }
            Token::Question if delimiters.is_empty() => conditionals += 1,
            Token::Identifier(_)
                if delimiters.is_empty() && starts_quantifier(token, tokens.clone()) =>
            {
                conditionals += 1;
            }
            Token::Colon if delimiters.is_empty() && conditionals > 0 => conditionals -= 1,
            Token::Colon | Token::ColonAssign | Token::Semicolon | Token::Indent
                if delimiters.is_empty() =>
            {
                return false;
            }
            Token::LeftParen | Token::LeftBracket | Token::LeftBrace => {
                delimiters.push(token);
            }
            Token::RightParen | Token::RightBracket | Token::RightBrace
                if !closes(delimiters.pop(), token) =>
            {
                return false;
            }
            _ => {}
        }
        nonempty = true;
    }
    false
}

fn closes(open: Option<&Token>, close: &Token) -> bool {
    matches!(
        (open, close),
        (Some(Token::LeftParen), Token::RightParen)
            | (Some(Token::LeftBracket), Token::RightBracket)
            | (Some(Token::LeftBrace), Token::RightBrace)
    )
}

fn starts_quantifier<'a>(word: &Token, mut rest: impl Iterator<Item = &'a Token>) -> bool {
    match (rest.next(), rest.next()) {
        (Some(binder), Some(keyword)) => Quantifier::starting(word, binder, keyword).is_some(),
        _ => false,
    }
}

/// Buffer a lexical named-arrow header, not anonymous arrows or arrow text in
/// literals/comments. Either spelling, `=>` or `->`, ends the header. The
/// parser still owns parameter validation and the real indentation
/// requirement when the input is committed.
fn indented_named_arrow_header(line: &str) -> bool {
    let Ok(tokens) = lex_header(line) else {
        return false;
    };
    let mut tokens = header_tokens(&tokens);
    matches!(tokens.next(), Some(Token::Keyword(Keyword::Def)))
        && matches!(tokens.next(), Some(Token::Identifier(_)))
        && tokens.next_back().is_some_and(Token::is_arrow)
        && !tokens.any(Token::is_arrow)
}

/// A bare try starts the existing buffer; catch stays in that same submission.
/// Lexing the whole line rejects malformed suffixes rather than hiding errors.
/// The parser still validates both suites when blank-line commit or EOF runs it.
fn indented_try_header(line: &str) -> bool {
    let Ok(tokens) = lex_header(line) else {
        return false;
    };
    let mut tokens = header_tokens(&tokens);
    matches!(tokens.next(), Some(Token::Keyword(Keyword::Try))) && tokens.next().is_none()
}

fn header_tokens(tokens: &[Spanned]) -> impl DoubleEndedIterator<Item = &Token> {
    tokens
        .iter()
        .map(|spanned| &spanned.token)
        .filter(|token| !is_layout(token))
}

#[cfg(test)]
mod tests;
