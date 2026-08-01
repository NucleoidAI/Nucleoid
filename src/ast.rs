use std::fmt;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Assign {
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

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub type_name: Option<String>,
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
    List(Vec<Expr>),
    ObjectLiteral(Vec<(String, Expr)>),
    Function(Arc<Function>),
    Assign {
        target: Box<Expr>,
        value: Box<Expr>,
    },
    Delete(Box<Expr>),
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    StrictEqual,
    StrictNotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalOp {
    And,
    Or,
}

impl BinaryOp {
    pub fn as_str(self) -> &'static str {
        match self {
            BinaryOp::Add => "+",
            BinaryOp::Subtract => "-",
            BinaryOp::Multiply => "*",
            BinaryOp::Divide => "/",
            BinaryOp::Modulo => "%",
            BinaryOp::Equal => "==",
            BinaryOp::NotEqual => "!=",
            BinaryOp::StrictEqual => "===",
            BinaryOp::StrictNotEqual => "!==",
            BinaryOp::Less => "<",
            BinaryOp::LessEqual => "<=",
            BinaryOp::Greater => ">",
            BinaryOp::GreaterEqual => ">=",
        }
    }
}

impl Expr {
    /// The dotted path this expression denotes, when it is a plain identifier
    /// chain such as `a`, `person1.name` or `$Person.mortal`.
    pub fn path(&self) -> Option<String> {
        match self {
            Expr::Identifier(name) => Some(name.clone()),
            Expr::ClassRef(name) => Some(name.clone()),
            Expr::ObjectRef(id) => Some(id.clone()),
            Expr::Member { object, property } => {
                object.path().map(|base| format!("{base}.{property}"))
            }
            _ => None,
        }
    }

    /// The leftmost identifier of a path expression.
    pub fn root(&self) -> Option<&str> {
        match self {
            Expr::Identifier(name) => Some(name),
            Expr::ClassRef(name) => Some(name),
            Expr::ObjectRef(id) => Some(id),
            Expr::Member { object, .. } => object.root(),
            Expr::Index { object, .. } => object.root(),
            Expr::Slice { object, .. } => object.root(),
            Expr::Call { callee, .. } => callee.root(),
            _ => None,
        }
    }
}

impl fmt::Display for Expr {
    /// Renders the expression back to source. Statement keys in the dependency
    /// graph are derived from this, so two spellings of the same condition
    /// collapse onto one node.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Null => f.write_str("null"),
            Expr::Bool(bool) => write!(f, "{bool}"),
            Expr::Number(number) => f.write_str(&crate::value::format_number(*number)),
            Expr::String(string) => write!(f, "\"{string}\""),
            Expr::Template(parts) => {
                f.write_str("`")?;
                for part in parts {
                    match part {
                        TemplatePart::Literal(literal) => f.write_str(literal)?,
                        TemplatePart::Expression(expression) => write!(f, "${{{expression}}}")?,
                    }
                }
                f.write_str("`")
            }
            Expr::Regex(pattern) => write!(f, "/{pattern}/"),
            Expr::Identifier(name) => f.write_str(name),
            Expr::ClassRef(name) => write!(f, "${name}"),
            Expr::ObjectRef(id) => f.write_str(id),
            Expr::This => f.write_str("this"),
            Expr::Super(arguments) => {
                write!(f, "super({})", join(arguments))
            }
            Expr::Member { object, property } => write!(f, "{object}.{property}"),
            Expr::Index { object, index } => write!(f, "{object}[{index}]"),
            Expr::Slice { object, start, end } => {
                write!(f, "{object}[")?;
                if let Some(start) = start {
                    write!(f, "{start}")?;
                }
                f.write_str(":")?;
                if let Some(end) = end {
                    write!(f, "{end}")?;
                }
                f.write_str("]")
            }
            Expr::Call { callee, arguments } => write!(f, "{callee}({})", join(arguments)),
            Expr::Unary { operator, operand } => match operator {
                UnaryOp::Not => write!(f, "!{operand}"),
                UnaryOp::Negate => write!(f, "-{operand}"),
                UnaryOp::Plus => write!(f, "+{operand}"),
                UnaryOp::TypeOf => write!(f, "typeof {operand}"),
            },
            Expr::Binary {
                operator,
                left,
                right,
            } => write!(f, "{left}{}{right}", operator.as_str()),
            Expr::Logical {
                operator,
                left,
                right,
            } => {
                let operator = match operator {
                    LogicalOp::And => "&&",
                    LogicalOp::Or => "||",
                };
                write!(f, "{left}{operator}{right}")
            }
            Expr::List(items) => write!(f, "[{}]", join(items)),
            Expr::ObjectLiteral(entries) => {
                let rendered: Vec<String> = entries
                    .iter()
                    .map(|(key, value)| format!("\"{key}\":{value}"))
                    .collect();
                write!(f, "{{{}}}", rendered.join(","))
            }
            Expr::Function(function) => match &function.name {
                Some(name) => write!(f, "{name}()"),
                None => f.write_str("function()"),
            },
            Expr::Assign { target, value } => write!(f, "{target}={value}"),
            Expr::Delete(operand) => write!(f, "delete {operand}"),
        }
    }
}

fn join(expressions: &[Expr]) -> String {
    expressions
        .iter()
        .map(|expression| expression.to_string())
        .collect::<Vec<_>>()
        .join(",")
}
