use crate::error::{Error, Result};
use crate::graph::{NodeKey, ShapeKey};
use crate::lang::ast::Expr;
use crate::number;
use crate::reasoning::{Direction, Ranked, Selection, Stage};
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::{ObjectId, Value};

#[derive(Debug)]
pub struct Reason<'a> {
    pub node: &'a Expr,
}

impl<'a> Reason<'a> {
    #[must_use]
    pub fn new(node: &'a Expr) -> Self {
        Reason { node }
    }

    pub fn resolve(&self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Value> {
        match runtime.reason(self.node, scope)? {
            Some(selection) => Ok(runtime.materialize(&selection)),
            None => Ok(Value::Undefined),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum Link<'a> {
    Stage(Stage),
    Depth(&'a Expr),
    Path(&'a Expr),
}

impl Link<'_> {
    pub(crate) fn is_depth(self) -> bool {
        matches!(self, Link::Depth(_))
    }

    fn operation(self) -> Option<&'static str> {
        match self {
            Link::Stage(stage @ (Stage::Stated | Stage::Derived | Stage::Diff)) => {
                Some(stage.as_str())
            }
            Link::Stage(Stage::Why | Stage::Affects) => None,
            Link::Depth(_) => Some("depth"),
            Link::Path(_) => Some("path"),
        }
    }
}

pub(crate) fn pipeline(expression: &Expr) -> (Vec<Link<'_>>, &Expr) {
    let mut links = Vec::new();
    let mut base = expression;
    while let Some((link, source)) = split(base) {
        links.push(link);
        base = source;
    }
    (links, base)
}

fn split(expression: &Expr) -> Option<(Link<'_>, &Expr)> {
    match expression {
        Expr::Reason { stage, source } => Some((Link::Stage(*stage), source)),
        Expr::Depth { source, limit } => Some((Link::Depth(limit), source)),
        Expr::Path { source, target } => Some((Link::Path(target), source)),
        _ => None,
    }
}

impl Runtime {
    pub(crate) fn reason(
        &mut self,
        expression: &Expr,
        scope: &mut Scope,
    ) -> Result<Option<Selection>> {
        let (links, base) = pipeline(expression);
        for link in &links {
            self.check_operation(*link, scope)?;
        }

        let selection = self.base_selection(base, scope)?;
        if links.iter().copied().any(Link::is_depth) {
            return self.ranked(&links, selection, scope);
        }

        let mut selection = selection;
        let mut direction = Direction::Dependencies;
        for link in links.iter().rev() {
            match *link {
                Link::Stage(stage) => {
                    direction = stage.direction().unwrap_or(direction);
                    selection = self.apply_stage(stage, &selection)?;
                }
                Link::Path(target) => {
                    let target = self.selection_key(target, scope)?;
                    selection = self.apply_path(selection, &target, direction);
                }
                Link::Depth(_) => {}
            }
        }
        Ok(Some(selection))
    }

    fn ranked(
        &mut self,
        links: &[Link<'_>],
        selection: Selection,
        scope: &mut Scope,
    ) -> Result<Option<Selection>> {
        let mut ranked = Ranked::unwalked(selection);
        for link in links.iter().rev() {
            ranked = match *link {
                Link::Stage(stage) => self.apply_ranked(stage, ranked)?,
                Link::Depth(limit) => match self.depth_limit(limit, scope)? {
                    Some(limit) => ranked.within(limit),
                    None => return Ok(None),
                },
                Link::Path(target) => {
                    let target = self.selection_key(target, scope)?;
                    self.ranked_path(ranked, &target)
                }
            };
        }
        Ok(Some(ranked.into_selection()))
    }

    fn check_operation(&self, link: Link<'_>, scope: &Scope) -> Result<()> {
        match link.operation() {
            Some(name) if self.binds(name, scope) => {
                Err(Error::type_error(format!("{name} is not an operation")))
            }
            _ => Ok(()),
        }
    }

    fn binds(&self, name: &str, scope: &Scope) -> bool {
        scope.has(name)
            || self.state.has_variable(name)
            || self.state.has_function(name)
            || self.state.has_class(name)
    }

    fn depth_limit(&mut self, limit: &Expr, scope: &mut Scope) -> Result<Option<usize>> {
        match self.evaluate(limit, scope)? {
            Value::Null | Value::Undefined => Ok(None),
            Value::Number(value) => number::non_negative_integer(value)
                .map(Some)
                .ok_or_else(|| not_a_limit(limit)),
            Value::Bool(_)
            | Value::String(_)
            | Value::List(_)
            | Value::Object(_)
            | Value::Class(_)
            | Value::Date(_)
            | Value::Regex(_)
            | Value::Function(_) => Err(not_a_limit(limit)),
        }
    }

    fn base_selection(&mut self, base: &Expr, scope: &mut Scope) -> Result<Selection> {
        if matches!(base, Expr::Model) {
            self.track_shape(&ShapeKey::Model);
            let keys = self.model_keys();
            return Ok(self.selection_of(keys));
        }

        let key = self.selection_key(base, scope)?;
        Ok(self.selection_of(vec![key]))
    }

    fn selection_key(&mut self, expression: &Expr, scope: &mut Scope) -> Result<NodeKey> {
        match expression {
            Expr::Identifier(name) => {
                if let Some(Value::Object(id)) = scope.retrieve(name) {
                    return Ok(NodeKey::object(&id.clone()));
                }

                self.reference_key(name)
            }

            Expr::Member { object, property } => {
                let base = self.evaluate(object, scope)?;

                match base {
                    Value::Object(id) => Ok(NodeKey::property(&id, property)),
                    Value::Undefined | Value::Null => Err(Error::not_defined(object.describe())),
                    _ => Err(Error::type_error(format!("Cannot explain {expression}"))),
                }
            }

            Expr::ObjectRef(id) => Ok(NodeKey::object(&ObjectId::from(id.clone()))),

            other => Err(Error::type_error(format!("Cannot explain {other}"))),
        }
    }
}

fn not_a_limit(limit: &Expr) -> Error {
    Error::type_error(format!(
        "{} is not a non-negative integer",
        limit.describe()
    ))
}
