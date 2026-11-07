use logos::Logos;

use super::token::{Keyword, Token};
use super::{delimited, unescape};

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\f\r]+")]
#[logos(skip r"#[^\n]*")]
pub(super) enum Raw {
    #[token("\n")]
    Newline,

    #[regex(r"[A-Za-z_][A-Za-z0-9_]*", |lexer| lexer.slice().to_string())]
    Word(String),

    #[regex(r"\$[A-Za-z_][A-Za-z0-9_]*", |lexer| lexer.slice().trim_start_matches('$').to_string())]
    ClassRef(String),

    #[regex(r"[0-9]+(\.[0-9]+)?([eE][+-]?[0-9]+)?", |lexer| lexer.slice().parse::<f64>().ok())]
    #[regex(r"\.[0-9]+([eE][+-]?[0-9]+)?", |lexer| lexer.slice().parse::<f64>().ok(), priority = 4)]
    Number(f64),

    #[regex(r#""([^"\\]|\\.)*""#, |lexer| unescape(lexer.slice()))]
    #[regex(r"'([^'\\]|\\.)*'", |lexer| unescape(lexer.slice()))]
    Str(String),

    #[regex(r"`([^`\\]|\\.)*`", |lexer| delimited(lexer.slice()).to_string())]
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
    #[token("?")]
    Question,
    #[token("??")]
    QuestionQuestion,
    #[token(";")]
    Semicolon,
    #[token(".")]
    Dot,
    #[token("@")]
    At,
    #[token("=>")]
    Arrow,
    #[token("->")]
    ThinArrow,
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
    #[token(":=")]
    ColonAssign,
    #[token("+=")]
    PlusAssign,
    #[token("-=")]
    MinusAssign,
    #[token("*=")]
    StarAssign,
    #[token("**=")]
    StarStarAssign,
    #[token("/=")]
    SlashAssign,
    #[token("%=")]
    PercentAssign,
    #[token("&&=")]
    AndAndAssign,
    #[token("||=")]
    OrOrAssign,
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
    #[token("**")]
    StarStar,
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

impl Raw {
    pub(super) fn plain(self) -> Token {
        match self {
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
            Raw::Question => Token::Question,
            Raw::QuestionQuestion => Token::QuestionQuestion,
            Raw::Semicolon => Token::Semicolon,
            Raw::Dot => Token::Dot,
            Raw::At => Token::At,
            Raw::Arrow => Token::Arrow,
            Raw::ThinArrow => Token::ThinArrow,
            Raw::Assign => Token::Assign,
            Raw::ColonAssign => Token::ColonAssign,
            Raw::PlusAssign => Token::PlusAssign,
            Raw::MinusAssign => Token::MinusAssign,
            Raw::StarAssign => Token::StarAssign,
            Raw::StarStarAssign => Token::StarStarAssign,
            Raw::PercentAssign => Token::PercentAssign,
            Raw::AndAndAssign => Token::AndAndAssign,
            Raw::OrOrAssign => Token::OrOrAssign,
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
            Raw::StarStar => Token::StarStar,
            Raw::Percent => Token::Percent,
            Raw::Bang => Token::Bang,
            Raw::AndAnd => Token::AndAnd,
            Raw::OrOr => Token::OrOr,
            Raw::Pipe => Token::Pipe,
            Raw::Slash => Token::Slash,
            Raw::SlashAssign => Token::SlashAssign,
        }
    }
}
