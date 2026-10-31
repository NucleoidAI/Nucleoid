use std::collections::HashMap;

use crate::builtins;
use crate::error::{Error, Position, Result};
use crate::lang::ast::{Expr, Stmt, literal};
use crate::lang::evaluation::{Flow, TrackingMode};
use crate::nuc::Outcome;
use crate::nuc::expression::is_assertion;
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::{ObjectData, ObjectId, Value};

#[derive(Debug, Clone)]
pub struct Suppose {
    pub target: Option<Expr>,
    pub body: Vec<Stmt>,
}

impl Suppose {
    pub fn new(target: Option<Expr>, body: Vec<Stmt>) -> Self {
        Suppose { target, body }
    }

    pub fn run(&mut self, runtime: &mut Runtime, scope: &mut Scope) -> Result<Outcome> {
        let valued = has_value(&self.body);
        if self.target.is_some() && !valued {
            return Err(Error::type_error("suppose has no value"));
        }

        match (&self.target, runtime.suppose(&self.body, valued, scope)?) {
            (Some(target), Flow::Normal(value)) => {
                let assignment = Stmt::Assign {
                    target: target.clone(),
                    value: literal::constant(&value),
                };
                runtime.execute(&assignment, scope).map(Outcome::flow)
            }
            (_, flow) => Ok(Outcome::flow(flow)),
        }
    }
}

fn has_value(body: &[Stmt]) -> bool {
    match body.last() {
        Some(statement @ Stmt::Expression(_)) => !is_assertion(statement),
        Some(Stmt::Suppose { target: None, body }) => has_value(body),
        _ => false,
    }
}

enum Frozen {
    Value(Value),
    List(Vec<Frozen>),
    Object(usize),
}

#[derive(Default)]
struct Snapshot {
    objects: Vec<Vec<(String, Frozen)>>,
    indices: HashMap<ObjectId, usize>,
    pending: Vec<(usize, ObjectId)>,
}

impl Snapshot {
    fn freeze(&mut self, value: &Value) -> Frozen {
        match value {
            Value::List(items) => {
                Frozen::List(items.iter().map(|item| self.freeze(item)).collect())
            }
            Value::Object(id) => {
                if let Some(index) = self.indices.get(id) {
                    return Frozen::Object(*index);
                }
                let index = self.objects.len();
                self.objects.push(Vec::new());
                self.indices.insert(id.clone(), index);
                self.pending.push((index, id.clone()));
                Frozen::Object(index)
            }
            other => Frozen::Value(other.clone()),
        }
    }
}

struct Escaped {
    root: Frozen,
    objects: Vec<Vec<(String, Frozen)>>,
}

enum Escape {
    Normal(Escaped),
    Return(Escaped),
    Thrown(Escaped, Option<Position>),
    Failed(Error),
}

impl Runtime {
    fn suppose(&mut self, body: &[Stmt], valued: bool, scope: &mut Scope) -> Result<Flow> {
        let savepoint = self.savepoint();
        let locals = scope.clone();
        let null_read = std::mem::take(&mut self.null_read);
        let undefined_read = std::mem::take(&mut self.undefined_read);
        self.stack = self.stack.nested();

        let result = self.with_tracking(TrackingMode::Isolated, |runtime| {
            runtime.execute_all(body, scope)
        });
        let escape = match result {
            Ok((Flow::Normal(value), _)) if valued => Escape::Normal(self.escape(&value)),
            Ok((Flow::Normal(_), _)) => Escape::Normal(self.escape(&Value::Null)),
            Ok((Flow::Return(value), _)) => Escape::Return(self.escape(&value)),
            Err(error) => match error.thrown_value() {
                Some(value) => Escape::Thrown(self.escape(value), error.position()),
                None => Escape::Failed(error),
            },
        };

        self.rollback_to(savepoint);
        *scope = locals;
        self.null_read = null_read;
        self.undefined_read = undefined_read;

        match escape {
            Escape::Normal(escaped) => Ok(Flow::Normal(self.thaw(escaped))),
            Escape::Return(escaped) => Ok(Flow::Return(self.thaw(escaped))),
            Escape::Thrown(escaped, position) => {
                let error = Error::thrown(self.thaw(escaped));
                Err(match position {
                    Some(position) => error.at(position),
                    None => error,
                })
            }
            Escape::Failed(error) => Err(error),
        }
    }

    fn escape(&self, value: &Value) -> Escaped {
        let mut snapshot = Snapshot::default();
        let root = snapshot.freeze(value);

        while let Some((index, id)) = snapshot.pending.pop() {
            let properties = match self.state.object(&id) {
                Some(data) => data
                    .properties
                    .iter()
                    .map(|(name, value)| (name.clone(), snapshot.freeze(value)))
                    .collect(),
                None => Vec::new(),
            };
            snapshot.objects[index] = properties;
        }

        Escaped {
            root,
            objects: snapshot.objects,
        }
    }

    fn thaw(&mut self, escaped: Escaped) -> Value {
        let ids: Vec<ObjectId> = escaped
            .objects
            .iter()
            .map(|_| ObjectId::anonymous())
            .collect();

        for (id, properties) in ids.iter().zip(escaped.objects) {
            let mut data = ObjectData::new(None);
            for (name, frozen) in properties {
                data.properties.insert(name, self.thawed(frozen, &ids));
            }
            self.insert_object(id.clone(), data);
        }

        self.thawed(escaped.root, &ids)
    }

    fn thawed(&self, frozen: Frozen, ids: &[ObjectId]) -> Value {
        match frozen {
            Frozen::List(items) => Value::List(
                items
                    .into_iter()
                    .map(|item| self.thawed(item, ids))
                    .collect(),
            ),
            Frozen::Object(index) => Value::Object(ids[index].clone()),
            Frozen::Value(Value::Class(name))
                if !self.state.has_class(&name) && !builtins::is_global(&name) =>
            {
                Value::Null
            }
            Frozen::Value(value) => value,
        }
    }
}
