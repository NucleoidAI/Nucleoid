use logos::Logos;

use crate::error::{Error, Result};

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\f\r]+")]
#[logos(skip r"#[^\n]*")]
enum Raw {
    #[token("\n")]
    Newline,

    #[regex(r"[A-Za-z_][A-Za-z0-9_]*", |lexer| lexer.slice().to_string())]
    Word(String),

    #[regex(r"\$[A-Za-z_][A-Za-z0-9_]*", |lexer| lexer.slice()[1..].to_string())]
    ClassRef(String),

    #[regex(r"[0-9]+(\.[0-9]+)?([eE][+-]?[0-9]+)?", |lexer| lexer.slice().parse::<f64>().ok())]
    #[regex(r"\.[0-9]+([eE][+-]?[0-9]+)?", |lexer| lexer.slice().parse::<f64>().ok(), priority = 4)]
    Number(f64),

    #[regex(r#""([^"\\]|\\.)*""#, |lexer| unescape(lexer.slice()))]
    #[regex(r"'([^'\\]|\\.)*'", |lexer| unescape(lexer.slice()))]
    Str(String),

    #[regex(r"`([^`\\]|\\.)*`", |lexer| lexer.slice()[1..lexer.slice().len() - 1].to_string())]
    Template(String),

    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[token("[")]
    LeftBracket,
    #[token("]")]
    RightBracket,
    #[token("{")]
    LeftBrace,
    #[token("}")]
    RightBrace,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token(";")]
    Semicolon,
    #[token(".")]
    Dot,
    #[token("=>")]
    Arrow,
    #[token("===")]
    StrictEqual,
    #[token("!==")]
    StrictNotEqual,
    #[token("==")]
    Equal,
    #[token("!=")]
    NotEqual,
    #[token("<=")]
    LessEqual,
    #[token(">=")]
    GreaterEqual,
    #[token("=")]
    Assign,
    #[token("<")]
    Less,
    #[token(">")]
    Greater,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("!")]
    Bang,
    #[token("&&")]
    AndAnd,
    #[token("||")]
    OrOr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    If,
    Else,
    For,
    Of,
    In,
    Class,
    Def,
    Function,
    Return,
    Throw,
    Delete,
    Try,
    Catch,
    Pass,
    Not,
    And,
    Or,
    True,
    False,
    Null,
    Super,
    This,
    TypeOf,
    New,
}

impl Keyword {
    fn from_word(word: &str) -> Option<Keyword> {
        Some(match word {
            "if" => Keyword::If,
            "else" | "elif" => Keyword::Else,
            "for" => Keyword::For,
            "of" => Keyword::Of,
            "in" => Keyword::In,
            "class" => Keyword::Class,
            "def" => Keyword::Def,
            "function" => Keyword::Function,
            "return" => Keyword::Return,
            "throw" => Keyword::Throw,
            "delete" => Keyword::Delete,
            "try" => Keyword::Try,
            "catch" => Keyword::Catch,
            "pass" => Keyword::Pass,
            "not" => Keyword::Not,
            "and" => Keyword::And,
            "or" => Keyword::Or,
            "true" => Keyword::True,
            "false" => Keyword::False,
            "null" | "None" | "undefined" => Keyword::Null,
            "super" => Keyword::Super,
            "this" | "self" => Keyword::This,
            "typeof" => Keyword::TypeOf,
            "new" => Keyword::New,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Identifier(String),
    ClassRef(String),
    Number(f64),
    Str(String),
    Template(String),
    Regex(String),
    Keyword(Keyword),

    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Comma,
    Colon,
    Semicolon,
    Dot,
    Arrow,

    Assign,
    Equal,
    StrictEqual,
    NotEqual,
    StrictNotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Bang,
    AndAnd,
    OrOr,

    Newline,
    Indent,
    Dedent,
    Eof,
}

impl Token {
    /// Whether a `/` following this token starts a regular expression literal
    /// rather than a division.
    fn precedes_regex(&self) -> bool {
        !matches!(
            self,
            Token::Identifier(_)
                | Token::ClassRef(_)
                | Token::Number(_)
                | Token::Str(_)
                | Token::Template(_)
                | Token::Regex(_)
                | Token::RightParen
                | Token::RightBracket
                | Token::RightBrace
                | Token::Keyword(Keyword::This)
        )
    }
}

fn unescape(slice: &str) -> String {
    let inner = &slice[1..slice.len() - 1];
    let mut output = String::with_capacity(inner.len());
    let mut characters = inner.chars();

    while let Some(character) = characters.next() {
        if character != '\\' {
            output.push(character);
            continue;
        }

        match characters.next() {
            Some('n') => output.push('\n'),
            Some('t') => output.push('\t'),
            Some('r') => output.push('\r'),
            Some('0') => output.push('\0'),
            Some(other) => output.push(other),
            None => output.push('\\'),
        }
    }

    output
}

pub fn tokenize(source: &str) -> Result<Vec<Token>> {
    let flat = scan(source)?;
    Ok(layout(flat))
}

struct Scanned {
    token: Token,
    start: usize,
    line_start: bool,
    indent: usize,
}

fn scan(source: &str) -> Result<Vec<Scanned>> {
    let mut lexer = Raw::lexer(source);
    let mut scanned: Vec<Scanned> = Vec::new();
    let mut previous: Option<Token> = None;

    while let Some(next) = lexer.next() {
        let span = lexer.span();

        let raw = next.map_err(|()| {
            Error::syntax(format!(
                "Unexpected character '{}'",
                &source[span.start..span.end]
            ))
        })?;

        let token = match raw {
            Raw::Newline => Token::Newline,
            Raw::Word(word) => match Keyword::from_word(&word) {
                Some(keyword) => Token::Keyword(keyword),
                None => Token::Identifier(word),
            },
            Raw::ClassRef(name) => Token::ClassRef(name),
            Raw::Number(number) => Token::Number(number),
            Raw::Str(string) => Token::Str(string),
            Raw::Template(template) => Token::Template(template),
            Raw::LeftParen => Token::LeftParen,
            Raw::RightParen => Token::RightParen,
            Raw::LeftBracket => Token::LeftBracket,
            Raw::RightBracket => Token::RightBracket,
            Raw::LeftBrace => Token::LeftBrace,
            Raw::RightBrace => Token::RightBrace,
            Raw::Comma => Token::Comma,
            Raw::Colon => Token::Colon,
            Raw::Semicolon => Token::Semicolon,
            Raw::Dot => Token::Dot,
            Raw::Arrow => Token::Arrow,
            Raw::Assign => Token::Assign,
            Raw::Equal => Token::Equal,
            Raw::StrictEqual => Token::StrictEqual,
            Raw::NotEqual => Token::NotEqual,
            Raw::StrictNotEqual => Token::StrictNotEqual,
            Raw::Less => Token::Less,
            Raw::LessEqual => Token::LessEqual,
            Raw::Greater => Token::Greater,
            Raw::GreaterEqual => Token::GreaterEqual,
            Raw::Plus => Token::Plus,
            Raw::Minus => Token::Minus,
            Raw::Star => Token::Star,
            Raw::Percent => Token::Percent,
            Raw::Bang => Token::Bang,
            Raw::AndAnd => Token::AndAnd,
            Raw::OrOr => Token::OrOr,
            Raw::Slash => {
                let starts_regex = previous.as_ref().is_none_or(Token::precedes_regex);

                if starts_regex {
                    match read_regex(lexer.remainder()) {
                        Some((pattern, consumed)) => {
                            lexer.bump(consumed);
                            Token::Regex(pattern)
                        }
                        None => Token::Slash,
                    }
                } else {
                    Token::Slash
                }
            }
        };

        let (line_start, indent) = line_position(source, span.start);
        previous = Some(token.clone());
        scanned.push(Scanned {
            token,
            start: span.start,
            line_start,
            indent,
        });
    }

    Ok(scanned)
}

/// Reads the body of a regular expression literal, given the source that
/// follows its opening slash. Returns the pattern and the bytes consumed.
fn read_regex(remainder: &str) -> Option<(String, usize)> {
    let mut pattern = String::new();
    let mut characters = remainder.char_indices();
    let mut in_class = false;

    while let Some((index, character)) = characters.next() {
        match character {
            '\\' => {
                pattern.push(character);
                let (_, escaped) = characters.next()?;
                pattern.push(escaped);
            }
            '\n' => return None,
            '[' => {
                in_class = true;
                pattern.push(character);
            }
            ']' => {
                in_class = false;
                pattern.push(character);
            }
            '/' if !in_class => {
                let mut consumed = index + character.len_utf8();

                for flag in remainder[consumed..].chars() {
                    if flag.is_ascii_alphabetic() {
                        consumed += flag.len_utf8();
                    } else {
                        break;
                    }
                }

                return Some((pattern, consumed));
            }
            _ => pattern.push(character),
        }
    }

    None
}

fn line_position(source: &str, start: usize) -> (bool, usize) {
    let before = &source[..start];
    let line = match before.rfind('\n') {
        Some(index) => &before[index + 1..],
        None => before,
    };

    let line_start = line.chars().all(|character| character.is_whitespace());
    let indent = line
        .chars()
        .map(|character| if character == '\t' { 4 } else { 1 })
        .sum();

    (line_start, indent)
}

/// Turns the flat token stream into a block-structured one by inserting
/// `Indent`, `Dedent` and `Newline` tokens. Layout is suppressed inside `(` and
/// `[`, where line breaks carry no meaning.
fn layout(scanned: Vec<Scanned>) -> Vec<Token> {
    let mut output: Vec<Token> = Vec::new();
    let mut indents: Vec<usize> = vec![0];
    let mut depth = 0usize;
    let mut pending_newline = false;

    for item in scanned {
        if item.token == Token::Newline {
            if depth == 0 {
                pending_newline = true;
            }
            continue;
        }

        let depth_before = depth;

        match item.token {
            Token::LeftParen | Token::LeftBracket => depth += 1,
            Token::RightParen | Token::RightBracket => depth = depth.saturating_sub(1),
            _ => {}
        }

        if depth_before > 0 {
            output.push(item.token);
            continue;
        }

        if item.line_start {
            if pending_newline && !output.is_empty() {
                output.push(Token::Newline);
            }
            pending_newline = false;

            let current = *indents.last().unwrap_or(&0);

            if item.indent > current {
                indents.push(item.indent);
                output.push(Token::Indent);
            } else if item.indent < current {
                while indents.len() > 1 && *indents.last().unwrap() > item.indent {
                    indents.pop();
                    output.push(Token::Dedent);
                }
            }
        } else if pending_newline {
            output.push(Token::Newline);
            pending_newline = false;
        }

        let _ = item.start;
        output.push(item.token);
    }

    if pending_newline {
        output.push(Token::Newline);
    }

    while indents.len() > 1 {
        indents.pop();
        output.push(Token::Dedent);
    }

    output.push(Token::Eof);
    output
}
