//! Classes and the rules stated about them: `class Person`, `$Person.mortal =
//! true`. Mirrors `ref/src/nuc/CLASS.js` together with the `$CLASS` variants of
//! `LET`, `OBJECT`, `PROPERTY` and `IF`, which are the same thing said about a
//! type rather than one object.

use indexmap::IndexSet;

use crate::error::{Error, Result};
use crate::graph::{NodeKey, NodeKind};
use crate::lang::ast::{
    ClassDecl, Expr, FunctionBody, Stmt, collect_class_properties, collect_roots,
    find_class_reference_statement,
};
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::state::{ClassData, Declaration};
use crate::value::{ObjectId, Value};

impl Runtime {
    pub(crate) fn define_class(&mut self, declaration: &ClassDecl) -> Result<()> {
        let existing = self.state.class(&declaration.name).cloned();
        self.transaction
            .record_class(&declaration.name, existing.clone());

        let mut data = ClassData::new(declaration.name.clone());
        data.parent = declaration.parent.clone();
        data.parameters = declaration.parameters.clone();
        data.constructor = declaration.constructor.clone();

        for method in &declaration.methods {
            if let Some(name) = &method.name {
                data.methods.insert(name.clone(), method.clone());
            }
        }

        // A redeclared class keeps its instances and class-level declarations.
        if let Some(existing) = existing {
            data.instances = existing.instances;
            data.declarations = existing.declarations;

            if data.parameters.is_empty() && data.constructor.is_empty() {
                if let Some(initializer) = data.methods.get("init").cloned() {
                    data.parameters = initializer.parameters.clone();

                    if let FunctionBody::Block(body) = &initializer.body {
                        data.constructor = body.clone();
                    }
                }
            }
        }

        if data.constructor.is_empty() {
            if let Some(initializer) = data.methods.get("init").cloned() {
                data.parameters = initializer.parameters.clone();

                if let FunctionBody::Block(body) = &initializer.body {
                    data.constructor = body.clone();
                }
            }
        }

        self.state
            .classes
            .insert(declaration.name.clone(), data.clone());

        let key = NodeKey::new(format!("${}", declaration.name));
        self.register(&key, NodeKind::Class, None, IndexSet::new(), None)?;

        Ok(())
    }

    /// `$Person.mortal = true` — a rule that holds for every instance, now and
    /// later.
    pub(crate) fn assign_class_property(
        &mut self,
        class: String,
        property: String,
        value: &Expr,
        scope: &mut Scope,
    ) -> Result<Value> {
        if property == "value" {
            return Err(Error::type_error("Cannot use 'value' as a property"));
        }

        // A rule is checked where it is written, not when an instance finally
        // arrives to run it.
        self.check_rule_references(value, scope)?;

        let statement = Stmt::Assign {
            target: Expr::Member {
                object: Box::new(Expr::ClassRef(class.clone())),
                property: property.clone(),
            },
            value: value.clone(),
        };

        self.declare_on_class(&class, format!("${class}.{property}"), statement)?;
        Ok(Value::Null)
    }

    /// Reports a name a class-level rule reads that nothing has defined.
    fn check_rule_references(&self, value: &Expr, scope: &Scope) -> Result<()> {
        let mut roots = Vec::new();
        collect_roots(value, &mut roots);

        for root in roots {
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

    pub(crate) fn declare_on_class(
        &mut self,
        class: &str,
        key: String,
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
                    let id = ObjectId::from(format!("${class}.{name}"));
                    let mut scope = Scope::new();
                    self.create_instance(&instantiated, &arguments, id.clone(), &mut scope)?;

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

        let mut pending = Vec::new();
        collect_class_properties(value, class, &mut pending);

        let mut seen: Vec<String> = Vec::new();

        while let Some(current) = pending.pop() {
            if &current == property {
                return Err(Error::type_error("Circular Dependency"));
            }

            if seen.contains(&current) {
                continue;
            }

            seen.push(current.clone());

            if let Some(declaration) = data.declarations.get(&format!("${class}.{current}")) {
                if let Stmt::Assign { value, .. } = &declaration.statement {
                    collect_class_properties(value, class, &mut pending);
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
