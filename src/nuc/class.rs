//! `CLASS` — a type, and the rules stated about it. Mirrors
//! `ref/src/nuc/CLASS.js`, which likewise keeps the methods, the instances and
//! the declarations on the node and hands each declaration to every instance.

use indexmap::IndexSet;

use crate::error::{Error, Result};
use crate::expression::Expression;
use crate::graph::{NodeKey, NodeKind};
use crate::lang::ast::{ClassDecl, Expr, FunctionBody, Stmt, find_class_reference_statement};
use crate::nuc::Outcome;
use crate::nuc::object::Object;
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::state::{ClassData, Declaration, DeclarationKey};
use crate::value::ObjectId;

pub struct Class {
    pub declaration: ClassDecl,
}

impl Class {
    pub fn new(declaration: ClassDecl) -> Self {
        Class { declaration }
    }

    pub fn key(&self) -> NodeKey {
        NodeKey::class(&self.declaration.name)
    }

    pub fn run(&mut self, runtime: &mut Runtime, _scope: &mut Scope) -> Result<Outcome> {
        let name = &self.declaration.name;
        let existing = runtime.state.class(name).cloned();
        runtime.transaction.record_class(name, existing.clone());

        let mut data = ClassData::new(name.clone());
        data.parent = self.declaration.parent.clone();
        data.parameters = self.declaration.parameters.clone();
        data.constructor = self.declaration.constructor.clone();

        for method in &self.declaration.methods {
            if let Some(method_name) = &method.name {
                data.methods.insert(method_name.clone(), method.clone());
            }
        }

        // A redeclared class keeps its instances and class-level declarations.
        if let Some(existing) = existing {
            data.instances = existing.instances;
            data.declarations = existing.declarations;

            if data.parameters.is_empty() && data.constructor.is_empty() {
                data.adopt_initializer();
            }
        }

        if data.constructor.is_empty() {
            data.adopt_initializer();
        }

        runtime.state.classes.insert(name.clone(), data);

        Ok(Outcome::null())
    }

    pub fn graph(&self, runtime: &mut Runtime) -> Result<()> {
        runtime.file(&self.key(), NodeKind::Class, None, IndexSet::new(), None)
    }
}

impl ClassData {
    /// A class written with an `init` method takes its parameters and its
    /// constructor body from it.
    fn adopt_initializer(&mut self) {
        let Some(initializer) = self.methods.get("init").cloned() else {
            return;
        };

        self.parameters = initializer.parameters.clone();

        if let FunctionBody::Block(body) = &initializer.body {
            self.constructor = body.clone();
        }
    }
}

impl Runtime {
    /// Reports a name a class-level rule reads that nothing has defined.
    pub(crate) fn check_rule_references(&self, value: &Expr, scope: &Scope) -> Result<()> {
        for root in Expression::new(value).roots() {
            let known = scope.has(&root)
                || self.state.variables.contains_key(&root)
                || self.state.classes.contains_key(&root)
                || self.state.functions.contains_key(&root)
                || crate::builtins::is_global(&root);

            if !known {
                return Err(Error::not_defined(root));
            }
        }

        Ok(())
    }

    /// The class a statement declares against, if it mentions `$Class` outside
    /// of an instance scope.
    pub(crate) fn class_reference(&self, statement: &Stmt, scope: &Scope) -> Option<String> {
        if scope.instance().is_some() {
            return None;
        }

        let mut found = None;
        find_class_reference_statement(statement, &mut found);

        found.filter(|name| self.state.classes.contains_key(name))
    }

    /// Keeps a statement on the class and hands it to every instance there
    /// already is.
    pub(crate) fn declare_on_class(
        &mut self,
        class: &str,
        key: DeclarationKey,
        statement: Stmt,
    ) -> Result<()> {
        // `$Class.property = Other()` creates one object that every instance
        // shares, rather than one per instance.
        let statement = match &statement {
            Stmt::Assign { target, value } => match (target, self.instantiation(value)) {
                (
                    Expr::Member {
                        object,
                        property: name,
                    },
                    Some((instantiated, arguments)),
                ) if matches!(object.as_ref(), Expr::ClassRef(_)) => {
                    let id = ObjectId::shared(class, name);
                    let object = Object::new(id.clone(), instantiated, arguments);
                    let mut scope = Scope::new();
                    object.run(self, &mut scope)?;

                    Stmt::Assign {
                        target: target.clone(),
                        value: Expr::ObjectRef(id.to_string()),
                    }
                }
                _ => statement.clone(),
            },
            other => other.clone(),
        };

        self.check_class_cycle(class, &statement)?;

        let sequence = self.graph.next_sequence();

        let Some(data) = self.state.class(class) else {
            return Err(Error::not_defined(class));
        };

        let before = data.clone();
        let instances = data.instances.clone();

        self.transaction.record_class(class, Some(before));

        if let Some(data) = self.state.class_mut(class) {
            data.declarations.shift_remove(&key);
            data.declarations.insert(
                key.clone(),
                Declaration {
                    key,
                    statement: statement.clone(),
                    sequence,
                },
            );
        }

        for instance in instances {
            self.apply_declaration(&statement, &instance)?;
        }

        Ok(())
    }

    /// Rejects a class-level rule that would make two properties of the same
    /// class depend on each other, before any instance exists to reveal it.
    fn check_class_cycle(&self, class: &str, statement: &Stmt) -> Result<()> {
        let Stmt::Assign { target, value } = statement else {
            return Ok(());
        };

        let Expr::Member { object, property } = target else {
            return Ok(());
        };

        if !matches!(object.as_ref(), Expr::ClassRef(name) if name == class) {
            return Ok(());
        }

        let Some(data) = self.state.class(class) else {
            return Ok(());
        };

        let mut pending = Expression::new(value).class_properties(class);

        let mut seen: Vec<String> = Vec::new();

        while let Some(current) = pending.pop() {
            if &current == property {
                return Err(Error::type_error("Circular Dependency"));
            }

            if seen.contains(&current) {
                continue;
            }

            seen.push(current.clone());

            if let Some(declaration) = data
                .declarations
                .get(&DeclarationKey::property(class, &current))
            {
                if let Stmt::Assign { value, .. } = &declaration.statement {
                    pending.extend(Expression::new(value).class_properties(class));
                }
            }
        }

        Ok(())
    }

    /// Runs a class-level rule against one instance.
    pub(crate) fn apply_declaration(
        &mut self,
        statement: &Stmt,
        instance: &ObjectId,
    ) -> Result<()> {
        let mut scope = Scope::new();
        scope.set_instance(Some(instance.clone()));

        // A class-level statement is a declaration wherever the instantiation
        // that triggered it happened to be written.
        let nested = self.suspend_imperative();
        self.instances.push(instance.clone());
        self.push_tracking(true);
        let result = self.execute(statement, &mut scope);
        self.pop_tracking();
        self.instances.pop();
        self.restore_imperative(nested);

        result.map(|_| ())
    }

    /// Every rule that applies to a class, its parents' first, in the order
    /// they were declared.
    pub(crate) fn declarations_for(&self, class_name: &str) -> Vec<Declaration> {
        let mut declarations = Vec::new();
        let mut chain = Vec::new();
        let mut current = Some(class_name.to_string());

        while let Some(name) = current {
            let Some(class) = self.state.class(&name) else {
                break;
            };
            chain.push(class);
            current = class.parent.clone();
        }

        for class in chain.into_iter().rev() {
            declarations.extend(class.declarations_in_order());
        }

        declarations.sort_by_key(|declaration| declaration.sequence);
        declarations
    }
}
