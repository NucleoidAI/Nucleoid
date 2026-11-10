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
    /// The current instance, written as either `this` or `self`.
    This,
    TypeOf,
    New,
}

impl Keyword {
    #[must_use]
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

    pub(super) fn from_word(word: &str) -> Option<Keyword> {
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
    /// Exact decimal digits following `@`; never round a slot index via f64.
    VersionIndex(usize),
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
    /// `?`, which opens a conditional expression's consequent (NUC 59).
    Question,
    QuestionQuestion,
    Semicolon,
    Dot,
    At,
    Arrow,
    /// `->`, an optional spelling of [`Token::Arrow`] (NUC 57). It stays a
    /// separate token so that diagnostics name the arrow the source wrote.
    ThinArrow,

    Assign,
    ColonAssign,
    PlusAssign,
    MinusAssign,
    StarAssign,
    StarStarAssign,
    SlashAssign,
    PercentAssign,
    AndAndAssign,
    OrOrAssign,
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
    StarStar,
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
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Token::Identifier(name) => format!("identifier '{name}'"),
            Token::ClassRef(name) => format!("'${name}'"),
            Token::Number(number) => format!("number {}", crate::value::format_number(*number)),
            Token::VersionIndex(index) => format!("version {index}"),
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
            Token::Question => "'?'".to_string(),
            Token::QuestionQuestion => "'??'".to_string(),
            Token::Semicolon => "';'".to_string(),
            Token::Dot => "'.'".to_string(),
            Token::At => "'@'".to_string(),
            Token::Arrow => "'=>'".to_string(),
            Token::ThinArrow => "'->'".to_string(),
            Token::Assign => "'='".to_string(),
            Token::ColonAssign => "':='".to_string(),
            Token::PlusAssign => "'+='".to_string(),
            Token::MinusAssign => "'-='".to_string(),
            Token::StarAssign => "'*='".to_string(),
            Token::StarStarAssign => "'**='".to_string(),
            Token::SlashAssign => "'/='".to_string(),
            Token::PercentAssign => "'%='".to_string(),
            Token::AndAndAssign => "'&&='".to_string(),
            Token::OrOrAssign => "'||='".to_string(),
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
            Token::StarStar => "'**'".to_string(),
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

    /// Whether this token is an arrow: canonical `=>` or its `->` spelling.
    /// Both spellings introduce exactly the same parameters and bodies.
    #[must_use]
    pub fn is_arrow(&self) -> bool {
        matches!(self, Token::Arrow | Token::ThinArrow)
    }

    /// Whether a `/` following this token starts a regular expression literal
    /// rather than a division.
    pub(crate) fn precedes_regex(&self) -> bool {
        !matches!(
            self,
            Token::Identifier(_)
                | Token::ClassRef(_)
                | Token::Number(_)
                | Token::VersionIndex(_)
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
