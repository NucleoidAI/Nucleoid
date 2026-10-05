use logos::Logos;

use crate::error::{Error, Position, Result};

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
    #[token("|>")]
    Pipe,
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
    pub fn as_str(self) -> &'static str {
        match self {
            Keyword::If => "if",
            Keyword::Else => "else",
            Keyword::For => "for",
            Keyword::Of => "of",
            Keyword::In => "in",
            Keyword::Class => "class",
            Keyword::Def => "def",
            Keyword::Function => "function",
            Keyword::Return => "return",
            Keyword::Throw => "throw",
            Keyword::Delete => "delete",
            Keyword::Try => "try",
            Keyword::Catch => "catch",
            Keyword::Pass => "pass",
            Keyword::Not => "not",
            Keyword::And => "and",
            Keyword::Or => "or",
            Keyword::True => "true",
            Keyword::False => "false",
            Keyword::Null => "null",
            Keyword::Super => "super",
            Keyword::This => "this",
            Keyword::TypeOf => "typeof",
            Keyword::New => "new",
        }
    }

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
    Pipe,

    Newline,
    Indent,
    Dedent,
    Eof,
}

impl Token {
    /// How this token is named in an error message.
    pub fn describe(&self) -> String {
        match self {
            Token::Identifier(name) => format!("identifier '{name}'"),
            Token::ClassRef(name) => format!("'${name}'"),
            Token::Number(number) => format!("number {}", crate::value::format_number(*number)),
            Token::Str(_) => "a string".to_string(),
            Token::Template(_) => "a template string".to_string(),
            Token::Regex(_) => "a regular expression".to_string(),
            Token::Keyword(keyword) => format!("keyword '{}'", keyword.as_str()),
            Token::LeftParen => "'('".to_string(),
            Token::RightParen => "')'".to_string(),
            Token::LeftBracket => "'['".to_string(),
            Token::RightBracket => "']'".to_string(),
            Token::LeftBrace => "'{'".to_string(),
            Token::RightBrace => "'}'".to_string(),
            Token::Comma => "','".to_string(),
            Token::Colon => "':'".to_string(),
            Token::Semicolon => "';'".to_string(),
            Token::Dot => "'.'".to_string(),
            Token::Arrow => "'=>'".to_string(),
            Token::Assign => "'='".to_string(),
            Token::Equal => "'=='".to_string(),
            Token::StrictEqual => "'==='".to_string(),
            Token::NotEqual => "'!='".to_string(),
            Token::StrictNotEqual => "'!=='".to_string(),
            Token::Less => "'<'".to_string(),
            Token::LessEqual => "'<='".to_string(),
            Token::Greater => "'>'".to_string(),
            Token::GreaterEqual => "'>='".to_string(),
            Token::Plus => "'+'".to_string(),
            Token::Minus => "'-'".to_string(),
            Token::Star => "'*'".to_string(),
            Token::Slash => "'/'".to_string(),
            Token::Percent => "'%'".to_string(),
            Token::Bang => "'!'".to_string(),
            Token::AndAnd => "'&&'".to_string(),
            Token::OrOr => "'||'".to_string(),
            Token::Pipe => "'|>'".to_string(),
            Token::Newline => "end of line".to_string(),
            Token::Indent => "an indented block".to_string(),
            Token::Dedent => "the end of a block".to_string(),
            Token::Eof => "end of input".to_string(),
        }
    }

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

/// A token and where it came from.
#[derive(Debug, Clone, PartialEq)]
pub struct Spanned {
    pub token: Token,
    pub position: Position,
}

pub fn tokenize(source: &str) -> Result<Vec<Spanned>> {
    let lines = Lines::new(source);
    let flat = scan(source, &lines)?;
    Ok(layout(flat, lines.end()))
}

/// Turns byte offsets into line and column numbers.
struct Lines<'a> {
    source: &'a str,
    starts: Vec<usize>,
}

impl<'a> Lines<'a> {
    fn new(source: &'a str) -> Self {
        let mut starts = vec![0];

        for (index, byte) in source.bytes().enumerate() {
            if byte == b'\n' {
                starts.push(index + 1);
            }
        }

        Lines { source, starts }
    }

    fn position(&self, offset: usize) -> Position {
        let offset = offset.min(self.source.len());

        let line = match self.starts.binary_search(&offset) {
            Ok(index) => index,
            Err(index) => index.saturating_sub(1),
        };

        let start = self.starts[line];
        let column = self.source[start..offset].chars().count() + 1;

        Position::new(line + 1, column)
    }

    fn end(&self) -> Position {
        self.position(self.source.len())
    }
}

struct Scanned {
    token: Token,
    position: Position,
    line_start: bool,
    indent: usize,
}

fn scan(source: &str, lines: &Lines<'_>) -> Result<Vec<Scanned>> {
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
            .at(lines.position(span.start))
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
            Raw::Pipe => Token::Pipe,
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
            position: lines.position(span.start),
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
fn layout(scanned: Vec<Scanned>, end: Position) -> Vec<Spanned> {
    let mut output: Vec<Spanned> = Vec::new();
    let mut indents: Vec<usize> = vec![0];
    let mut depth = 0usize;
    let mut pending_newline = false;
    // An end-of-line belongs to the line it ends, not to the one that follows.
    let mut previous = Position::new(1, 1);

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

        let position = item.position;

        if depth_before > 0 {
            previous = position;
            output.push(Spanned {
                token: item.token,
                position,
            });
            continue;
        }

        if item.line_start {
            if pending_newline && !output.is_empty() {
                output.push(Spanned {
                    token: Token::Newline,
                    position: previous,
                });
            }
            pending_newline = false;

            let current = *indents.last().unwrap_or(&0);

            if item.indent > current {
                indents.push(item.indent);
                output.push(Spanned {
                    token: Token::Indent,
                    position,
                });
            } else if item.indent < current {
                while indents.len() > 1 && *indents.last().unwrap() > item.indent {
                    indents.pop();
                    output.push(Spanned {
                        token: Token::Dedent,
                        position,
                    });
                }
            }
        } else if pending_newline {
            output.push(Spanned {
                token: Token::Newline,
                position: previous,
            });
            pending_newline = false;
        }

        previous = position;

        output.push(Spanned {
            token: item.token,
            position,
        });
    }

    if pending_newline {
        output.push(Spanned {
            token: Token::Newline,
            position: previous,
        });
    }

    while indents.len() > 1 {
        indents.pop();
        output.push(Spanned {
            token: Token::Dedent,
            position: end,
        });
    }

    output.push(Spanned {
        token: Token::Eof,
        position: end,
    });

    output
}
