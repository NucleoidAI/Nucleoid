use crate::error::Result;
use crate::lang::ast::Expr;
use crate::lang::evaluation::TrackingMode;
use crate::nuc::Nuc;
use crate::runtime::Runtime;
use crate::scope::Scope;
use crate::value::Value;

pub(super) fn convert(
    runtime: &mut Runtime,
    scope: &mut Scope,
    target: &Expr,
    value: &Expr,
) -> Result<Nuc> {
    let null_read = runtime.null_read;
    let undefined_read = runtime.undefined_read;
    let result = runtime.with_tracking(TrackingMode::Isolated, |runtime| {
        let prepared = if matches!(target, Expr::Identifier(_)) {
            None
        } else {
            let assignment = Nuc::value_assignment(runtime, scope, target, value.clone())?;
            if matches!(assignment, Nuc::Pass) {
                return Ok(assignment);
            }
            if let Nuc::ValueAssign(action) = &assignment {
                if let Nuc::Property(property) = action.assignment.as_ref() {
                    property.check_name()?;
                }
            }
            Some(assignment)
        };

        if !matches!(
            runtime.evaluate(target, scope)?,
            Value::Null | Value::Undefined
        ) {
            return Ok(Nuc::Pass);
        }

        match prepared {
            Some(assignment) => Ok(assignment),
            None => Nuc::value_assignment(runtime, scope, target, value.clone()),
        }
    });
    runtime.null_read = null_read;
    runtime.undefined_read = undefined_read;
    result.map(|(assignment, _)| assignment)
}
