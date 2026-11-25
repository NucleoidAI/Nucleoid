use crate::error::{Error, Result};
use crate::graph::{NodeKey, ShapeKey};
use crate::lang::ast::{Expr, SetOperation};
use crate::number;
use crate::reasoning::{Direction, Projection, Ranked, Selection, Stage};
use crate::runtime::{MAX_DEPTH, Runtime};
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
        if let Expr::Reason { stage, source } = self.node {
            if let Some(projection) = stage.projection() {
                return runtime.projection_of(*stage, projection, source, scope);
            }
        }

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
    Combine(SetOperation, &'a Expr),
    Union(&'a Expr),
    Xor(&'a Expr),
}

impl Link<'_> {
    fn is_depth(self) -> bool {
        matches!(self, Link::Depth(_))
    }

    fn ends_pipeline(self) -> bool {
        matches!(self, Link::Stage(stage) if stage.projection().is_some())
    }

    pub(crate) fn freezes_argument(self) -> bool {
        matches!(
            self,
            Link::Depth(_) | Link::Combine(..) | Link::Union(_) | Link::Xor(_)
        )
    }

    fn operation(self) -> Option<&'static str> {
        match self {
            Link::Stage(
                stage @ (Stage::Stated
                | Stage::Derived
                | Stage::Diff
                | Stage::Nodes
                | Stage::Holds
                | Stage::Edges
                | Stage::Origins
                | Stage::Outcomes
                | Stage::Indegrees),
            ) => Some(stage.as_str()),
            Link::Stage(Stage::Why | Stage::Affects) => None,
            Link::Depth(_) => Some("depth"),
            Link::Path(_) => Some("path"),
            Link::Combine(operation, _) => Some(operation.as_str()),
            Link::Union(_) => Some("union"),
            Link::Xor(_) => Some("xor"),
        }
    }
}

pub(crate) fn pipeline(expression: &Expr) -> (Vec<Link<'_>>, &Expr) {
    let mut links = Vec::new();
    let mut base = expression;
    while let Some((link, source)) = split(base) {
        if link.ends_pipeline() {
            break;
        }
        links.push(link);
        base = source;
    }
    (links, base)
}

pub(crate) fn split(expression: &Expr) -> Option<(Link<'_>, &Expr)> {
    match expression {
        Expr::Reason { stage, source } => Some((Link::Stage(*stage), source)),
        Expr::Depth { source, limit } => Some((Link::Depth(limit), source)),
        Expr::Path { source, target } => Some((Link::Path(target), source)),
        Expr::Combine {
            operation,
            source,
            other,
        } => Some((Link::Combine(*operation, other), source)),
        Expr::Union { source, other } => Some((Link::Union(other), source)),
        Expr::Xor { source, other } => Some((Link::Xor(other), source)),
        _ => None,
    }
}

impl Runtime {
    pub(crate) fn reason(
        &mut self,
        expression: &Expr,
        scope: &mut Scope,
    ) -> Result<Option<Selection>> {
        let (links, selection) = self.selected(expression, scope)?;
        if links.iter().copied().any(Link::is_depth) {
            return Ok(self
                .ranked(&links, selection, scope)?
                .map(Ranked::into_selection));
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
                Link::Combine(operation, other) => match self.other_selection(other, scope)? {
                    Some(other) => selection = selection.combined(operation, &other),
                    None => return Ok(None),
                },
                Link::Union(other) => match self.other_selection(other, scope)? {
                    Some(other) => selection = selection.union(other),
                    None => return Ok(None),
                },
                Link::Xor(other) => match self.other_selection(other, scope)? {
                    Some(other) => selection = selection.xor(other),
                    None => return Ok(None),
                },
                Link::Depth(_) => {}
            }
        }
        Ok(Some(selection))
    }

    fn selected<'e>(
        &mut self,
        expression: &'e Expr,
        scope: &mut Scope,
    ) -> Result<(Vec<Link<'e>>, Selection)> {
        let (links, base) = pipeline(expression);
        for link in &links {
            self.check_operation(*link, scope)?;
        }
        let selection = self.base_selection(base, scope)?;
        Ok((links, selection))
    }

    fn projection_of(
        &mut self,
        stage: Stage,
        projection: Projection,
        source: &Expr,
        scope: &mut Scope,
    ) -> Result<Value> {
        self.check_operation(Link::Stage(stage), scope)?;
        Ok(match self.reason(source, scope)? {
            Some(selection) => self.projected(&selection, projection),
            None => Value::Undefined,
        })
    }

    fn other_selection(&mut self, other: &Expr, scope: &mut Scope) -> Result<Option<Selection>> {
        self.nested(|runtime| runtime.reason(other, scope))
    }

    fn other_ranked(&mut self, other: &Expr, scope: &mut Scope) -> Result<Option<Ranked>> {
        self.nested(|runtime| {
            let (links, selection) = runtime.selected(other, scope)?;
            runtime.ranked(&links, selection, scope)
        })
    }

    fn nested<T>(&mut self, evaluate: impl FnOnce(&mut Self) -> Result<T>) -> Result<T> {
        self.depth += 1;

        if self.depth > MAX_DEPTH {
            self.depth -= 1;
            return Err(Error::type_error("Maximum expression depth exceeded"));
        }

        let depth = self.depth;
        let result = evaluate(self);
        debug_assert_eq!(self.depth, depth);
        self.depth -= 1;
        result
    }

    fn ranked(
        &mut self,
        links: &[Link<'_>],
        selection: Selection,
        scope: &mut Scope,
    ) -> Result<Option<Ranked>> {
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
                Link::Combine(operation, other) => match self.other_selection(other, scope)? {
                    Some(other) => ranked.combined(operation, &other),
                    None => return Ok(None),
                },
                Link::Union(other) => match self.other_ranked(other, scope)? {
                    Some(other) => ranked.union(other),
                    None => return Ok(None),
                },
                Link::Xor(other) => match self.other_ranked(other, scope)? {
                    Some(other) => ranked.xor(other),
                    None => return Ok(None),
                },
            };
        }
        Ok(Some(ranked))
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
                    return Ok(NodeKey::object(id));
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
