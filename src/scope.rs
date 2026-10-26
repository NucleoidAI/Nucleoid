use indexmap::IndexMap;

use crate::history::History;
use crate::lang::ast::VersionSelector;
use crate::value::{ObjectId, Value};

#[derive(Debug, Clone)]
struct Binding {
    value: Value,
    history: History,
}

impl Binding {
    fn new(value: Value) -> Self {
        let mut history = History::default();
        history.push(value.clone());
        Self { value, history }
    }

    fn assign(&mut self, value: Value) {
        if !self.value.same_value(&value) {
            self.history.push(value.clone());
        }
        self.value = value;
    }
}

#[derive(Debug, Clone, Default)]
struct Frame {
    locals: IndexMap<String, Binding>,
    /// State names this block assigned, so a nested block writing the same name
    /// shadows it instead of writing through.
    assigned: Vec<String>,
    /// The instance a class-level declaration is being applied to.
    instance: Option<ObjectId>,
    /// The receiver of a constructor body.
    this: Option<ObjectId>,
    constructor_class: Option<String>,
}

/// The chain of local scopes surrounding the statement being executed. Locals
/// shadow state variables and are never filed in the dependency graph.
#[derive(Debug, Clone)]
pub struct Scope {
    frames: Vec<Frame>,
}

impl Default for Scope {
    fn default() -> Self {
        Scope::new()
    }
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            frames: vec![Frame::default()],
        }
    }

    pub fn push(&mut self) {
        self.frames.push(Frame::default());
    }

    pub fn pop(&mut self) {
        if self.frames.len() > 1 {
            self.frames.pop();
        }
    }

    pub fn depth(&self) -> usize {
        self.frames.len()
    }

    /// Whether this is the outermost scope, where assignments become state.
    pub fn is_root(&self) -> bool {
        self.frames.len() == 1
    }

    /// The innermost binding of a name, walking outwards through the frames.
    pub fn retrieve(&self, name: &str) -> Option<&Value> {
        self.frames
            .iter()
            .rev()
            .find_map(|frame| frame.locals.get(name).map(|binding| &binding.value))
    }

    pub(crate) fn version(&self, name: &str, selector: VersionSelector) -> Option<&Value> {
        self.frames
            .iter()
            .rev()
            .find_map(|frame| frame.locals.get(name))?
            .history
            .get(selector)
    }

    pub fn has(&self, name: &str) -> bool {
        self.retrieve(name).is_some()
    }

    /// Declares a local in the innermost frame.
    pub fn declare(&mut self, name: impl Into<String>, value: Value) {
        if let Some(frame) = self.frames.last_mut() {
            let name = name.into();
            if let Some(binding) = frame.locals.get_mut(&name) {
                binding.assign(value);
            } else {
                frame.locals.insert(name, Binding::new(value));
            }
        }
    }

    /// Assigns to the innermost frame that already holds the name, reporting
    /// whether such a frame existed.
    pub fn assign(&mut self, name: &str, value: Value) -> bool {
        for frame in self.frames.iter_mut().rev() {
            if let Some(binding) = frame.locals.get_mut(name) {
                binding.assign(value);
                return true;
            }
        }

        false
    }

    /// Records that this frame assigned a state name.
    pub fn record_assignment(&mut self, name: &str) {
        if let Some(frame) = self.frames.last_mut() {
            if !frame.assigned.iter().any(|assigned| assigned == name) {
                frame.assigned.push(name.to_string());
            }
        }
    }

    /// Whether an enclosing block — not the outermost scope — already assigned
    /// this name.
    pub fn assigned_by_enclosing(&self, name: &str) -> bool {
        let last = self.frames.len().saturating_sub(1);

        self.frames[..last]
            .iter()
            .skip(1)
            .any(|frame| frame.assigned.iter().any(|assigned| assigned == name))
    }

    pub fn instance(&self) -> Option<&ObjectId> {
        self.frames
            .iter()
            .rev()
            .find_map(|frame| frame.instance.as_ref())
    }

    pub fn set_instance(&mut self, instance: Option<ObjectId>) {
        if let Some(frame) = self.frames.last_mut() {
            frame.instance = instance;
        }
    }

    pub fn this(&self) -> Option<&ObjectId> {
        self.frames
            .iter()
            .rev()
            .find_map(|frame| frame.this.as_ref())
    }

    pub fn set_this(&mut self, this: Option<ObjectId>) {
        if let Some(frame) = self.frames.last_mut() {
            frame.this = this;
        }
    }

    pub(crate) fn constructor_class(&self) -> Option<&str> {
        self.frames
            .iter()
            .rev()
            .find_map(|frame| frame.constructor_class.as_deref())
    }

    pub(crate) fn set_constructor_class(&mut self, name: String) {
        if let Some(frame) = self.frames.last_mut() {
            frame.constructor_class = Some(name);
        }
    }
}
