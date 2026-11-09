//! Source text, the syntax tree it parses to, and how each node is evaluated.
//!
//! [`lexer`] and [`parser`] turn source into [`Expr`] and [`Stmt`], while
//! [`generator`] renders that tree back to canonical source for graph keys.
//! [`Ast`] is a closed enum of expression evaluators, selected by
//! [`Ast::convert`].
//!
//! [`Ast::resolve`] produces a value directly. [`Ast::generate`] renders source
//! for graph keys, not for evaluation. Dependency reads are recorded as they
//! happen in [`crate::lang::evaluation`].

mod analysis;
pub mod array;
pub mod call;
pub mod function;
pub mod generator;
pub mod identifier;
pub mod lexer;
pub mod literal;
pub mod new;
pub mod object;
pub mod operator;
pub mod parameter;
pub mod parser;
pub mod quantified;
pub mod reason;
pub mod template;
mod typed;
pub mod version;

pub use parameter::ParameterDefault;
pub use parser::{Program, parse, parse_expression, parse_program};

use std::sync::Arc;

use crate::reasoning::Stage;
pub(crate) use analysis::{
    collect_assigned_names, collect_read_roots, find_class_reference_statement,
};
use lexer::{Keyword, Token};
pub use typed::Ast;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Assign {
        target: Expr,
        value: Expr,
    },
    ValueAssign {
        target: Expr,
        value: Expr,
    },
    Expression(Expr),
    If {
        condition: Expr,
        consequent: Vec<Stmt>,
        alternate: Option<Box<Stmt>>,
    },
    Block(Vec<Stmt>),
    Class(ClassDecl),
    Function(Arc<Function>),
    For {
        variable: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    ForEntries {
        key: String,
        value: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    Return(Option<Expr>),
    Throw(Expr),
    Delete(Expr),
    Try {
        body: Vec<Stmt>,
        parameter: String,
        catch: Vec<Stmt>,
    },
    /// A declaration with a type but no definition, such as `a: int`.
    Declaration {
        name: String,
        type_name: String,
    },
    Pass,
    Suppose {
        target: Option<Expr>,
        body: Vec<Stmt>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassDecl {
    pub name: String,
    pub parent: Option<String>,
    pub parameters: Vec<Parameter>,
    pub fields: Vec<Parameter>,
    pub constructor: Vec<Stmt>,
    pub methods: Vec<Arc<Function>>,
}

#[derive(Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub type_name: Option<String>,
    /// A scalar literal used only when the argument position/key is absent.
    pub default: Option<ParameterDefault>,
    /// The final parenthesized formal collects remaining positional values.
    pub rest: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: Option<String>,
    pub parameters: Vec<Parameter>,
    pub body: FunctionBody,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunctionBody {
    Expression(Box<Expr>),
    Block(Vec<Stmt>),
}

/// Literal, oldest-first selectors for an actual storage slot's history.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionSelector {
    Index(usize),
    First,
    Last,
    Previous,
}

impl VersionSelector {
    pub(crate) fn from_name(name: &str) -> Option<Self> {
        match name {
            "first" => Some(Self::First),
            "last" => Some(Self::Last),
            "previous" => Some(Self::Previous),
            _ => None,
        }
    }

    pub(crate) fn is_fixed(self) -> bool {
        matches!(self, Self::Index(_) | Self::First)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Template(Vec<TemplatePart>),
    Regex(String),
    Identifier(String),
    /// A class reference written as `$Person`.
    ClassRef(String),
    /// A resolved object, used when a statement is rebuilt for the graph so it
    /// re-runs against the same object rather than re-resolving a path.
    ObjectRef(String),
    This,
    Super(Vec<Expr>),
    Member {
        object: Box<Expr>,
        property: String,
    },
    Version {
        reference: Box<Expr>,
        selector: VersionSelector,
    },
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    },
    Slice {
        object: Box<Expr>,
        start: Option<Box<Expr>>,
        end: Option<Box<Expr>>,
    },
    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
    },
    Unary {
        operator: UnaryOp,
        operand: Box<Expr>,
    },
    Binary {
        operator: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Logical {
        operator: LogicalOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    /// `condition ? consequent : alternate`, also written
    /// `consequent if condition else alternate` (NUC 59). The condition is
    /// evaluated once, then exactly one branch.
    Conditional {
        condition: Box<Expr>,
        consequent: Box<Expr>,
        alternate: Box<Expr>,
    },
    List(Vec<Expr>),
    ObjectLiteral(Vec<(String, Expr)>),
    SpreadObject(Vec<ObjectEntry>),
    Function(Arc<Function>),
    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
    },
    Delete(Box<Expr>),
    Reason {
        stage: Stage,
        source: Box<Expr>,
    },
    Depth {
        source: Box<Expr>,
        limit: Box<Expr>,
    },
    Path {
        source: Box<Expr>,
        target: Box<Expr>,
    },
    Model,
    Quantified {
        quantifier: Quantifier,
        binder: String,
        collection: Box<Expr>,
        condition: Box<Expr>,
    },
    /// `NOUN of OPERAND` (NUC 75): a list query named by a noun, written as a
    /// prefix operator. It means exactly the method call `OPERAND.NOUN()` and
    /// evaluates through it; only its rendering keeps the written form.
    Of {
        noun: Noun,
        operand: Box<Expr>,
    },
    Enum(Vec<EnumMember>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumMember {
    pub name: String,
    pub value: ParameterDefault,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectEntry {
    Property(String, Expr),
    Spread(Expr),
}

impl ObjectEntry {
    #[must_use]
    pub fn value(&self) -> &Expr {
        match self {
            ObjectEntry::Property(_, value) | ObjectEntry::Spread(value) => value,
        }
    }
}

/// The list queries named by a noun, which `NOUN of OPERAND` names. Queries
/// named by a verb, such as `sort` or `push`, keep the method call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Noun {
    Sum,
    Max,
    Min,
    Mode,
}

impl Noun {
    /// The noun as written, which is also the name of the method that
    /// `NOUN of OPERAND` calls as `OPERAND.NOUN()`.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Noun::Sum => "sum",
            Noun::Max => "max",
            Noun::Min => "min",
            Noun::Mode => "mode",
        }
    }

    /// The noun `name` spells exactly, or `None` for every other name.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Noun> {
        match name {
            "sum" => Some(Noun::Sum),
            "max" => Some(Noun::Max),
            "min" => Some(Noun::Min),
            "mode" => Some(Noun::Mode),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quantifier {
    All,
    Any,
}

impl Quantifier {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Quantifier::All => "all",
            Quantifier::Any => "any",
        }
    }

    #[must_use]
    pub fn starting(word: &Token, binder: &Token, keyword: &Token) -> Option<Quantifier> {
        match (word, binder, keyword) {
            (Token::Identifier(word), Token::Identifier(_), Token::Keyword(Keyword::In)) => {
                match word.as_str() {
                    "all" | "every" => Some(Quantifier::All),
                    "any" => Some(Quantifier::Any),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TemplatePart {
    Literal(String),
    Expression(Expr),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Negate,
    Plus,
    TypeOf,
    /// Identity evaluation with explicit grouping in canonical source. Used
    /// only for assignment-shaped call arguments that were not bare labels.
    Positional,
    Word,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
    Equal,
    NotEqual,
    StrictEqual,
    StrictNotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Is,
    IsNot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalOp {
    And,
    Or,
    Coalesce,
}
