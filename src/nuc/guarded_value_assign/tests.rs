use super::{Guard, convert};
use crate::lang::ast::{Expr, parse_expression};
use crate::runtime::Runtime;
use crate::scope::Scope;

#[test]
fn both_guards_restore_both_read_flags_on_takes_skips_and_errors() {
    for guard in [Guard::Nullish, Guard::Truthy] {
        for (null_read, undefined_read) in
            [(false, false), (true, false), (false, true), (true, true)]
        {
            for (target, succeeds) in [
                ("slot", true),
                ("active", true),
                ("box.absent", true),
                ("missing", false),
                ("box.value", false),
                ("number.item", false),
                ("slot@previous", false),
            ] {
                let mut runtime = Runtime::new();
                runtime
                    .run("slot = null\nactive = 1\nbox = {}\nnumber = 0")
                    .unwrap();
                runtime.null_read = null_read;
                runtime.undefined_read = undefined_read;
                let result = convert(
                    &mut runtime,
                    &mut Scope::new(),
                    &parse_expression(target).unwrap(),
                    &Expr::Number(9.0),
                    guard,
                );
                assert_eq!(result.is_ok(), succeeds, "{guard:?}: {target}");
                assert_eq!(runtime.null_read, null_read, "{guard:?}: {target}");
                assert_eq!(
                    runtime.undefined_read, undefined_read,
                    "{guard:?}: {target}"
                );
            }
        }
    }
}
