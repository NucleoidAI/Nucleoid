use indexmap::IndexMap;
use std::sync::Arc;

use crate::lang::ast::{Function, Parameter, Stmt};
use crate::runtime::Runtime;
use crate::value::{ObjectData, ObjectId, Value};

/// A class-level statement kept as a template and re-applied to every instance.
#[derive(Debug, Clone)]
pub struct Declaration {
    pub key: String,
    pub statement: Stmt,
    pub sequence: u64,
}

#[derive(Debug, Clone)]
pub struct ClassData {
    pub name: String,
    pub parent: Option<String>,
    pub parameters: Vec<Parameter>,
    pub constructor: Vec<Stmt>,
    pub methods: IndexMap<String, Arc<Function>>,
    pub instances: Vec<ObjectId>,
    pub declarations: IndexMap<String, Declaration>,
}

impl ClassData {
    pub fn new(name: impl Into<String>) -> Self {
        ClassData {
            name: name.into(),
            parent: None,
            parameters: Vec::new(),
            constructor: Vec::new(),
            methods: IndexMap::new(),
            instances: Vec::new(),
            declarations: IndexMap::new(),
        }
    }

    pub fn declarations_in_order(&self) -> Vec<Declaration> {
        let mut declarations: Vec<Declaration> = self.declarations.values().cloned().collect();
        declarations.sort_by_key(|declaration| declaration.sequence);
        declarations
    }
}

/// Everything the runtime holds: top-level variables, objects and classes.
#[derive(Debug, Clone, Default)]
pub struct State {
    pub variables: IndexMap<String, Value>,
    pub objects: IndexMap<ObjectId, ObjectData>,
    pub classes: IndexMap<String, ClassData>,
    pub functions: IndexMap<String, Arc<Function>>,
}

impl State {
    pub fn new() -> Self {
        State::default()
    }

    pub fn variable(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    pub fn object(&self, id: &ObjectId) -> Option<&ObjectData> {
        self.objects.get(id)
    }

    pub fn object_mut(&mut self, id: &ObjectId) -> Option<&mut ObjectData> {
        self.objects.get_mut(id)
    }

    pub fn property(&self, id: &ObjectId, property: &str) -> Option<&Value> {
        self.objects
            .get(id)
            .and_then(|object| object.properties.get(property))
    }

    pub fn class(&self, name: &str) -> Option<&ClassData> {
        self.classes.get(name)
    }

    pub fn class_mut(&mut self, name: &str) -> Option<&mut ClassData> {
        self.classes.get_mut(name)
    }

    pub fn clear(&mut self) {
        self.variables.clear();
        self.objects.clear();
        self.classes.clear();
        self.functions.clear();
    }
}

/// The writes. `state.assign` in `ref/src/state.js` takes one path and lets
/// JavaScript work out whether it names a variable or a property; the two are
/// separate here because the graph keys them differently.
///
/// Both go through the transaction first, so a run that fails part way leaves
/// the state exactly as it was. `state.expression`, `state.call` and
/// `state.throw` have no counterpart — they are `eval` wrappers, and nothing
/// here evaluates by handing text to another language.
impl Runtime {
    pub(crate) fn assign(&mut self, name: &str, value: Value) {
        let before = self.state.variables.get(name).cloned();
        self.transaction.record_variable(name, before);
        self.state.variables.insert(name.to_string(), value);
    }

    pub(crate) fn assign_property(&mut self, object: &ObjectId, property: &str, value: Value) {
        if !self.state.objects.contains_key(object) {
            self.transaction.record_object(object, None);
            self.state
                .objects
                .insert(object.clone(), ObjectData::new(None));
        }

        let before = self.state.property(object, property).cloned();
        self.transaction.record_property(object, property, before);

        if let Some(data) = self.state.object_mut(object) {
            data.properties.insert(property.to_string(), value);
        }
    }
}
