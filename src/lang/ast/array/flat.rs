use super::index::integer_or_infinity;
use crate::number;
use crate::runtime::Runtime;
use crate::value::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Depth {
    levels: usize,
}

impl Depth {
    fn new(argument: Option<&Value>) -> Self {
        let levels = argument.map_or(1, |value| {
            number::to_usize(integer_or_infinity(value.to_number()))
        });
        Depth { levels }
    }

    fn reads(self, level: usize) -> bool {
        level <= self.levels.max(1)
    }

    fn descends(self, level: usize) -> bool {
        level <= self.levels
    }
}

impl Runtime {
    pub(super) fn flatten_items(
        &mut self,
        items: Vec<Value>,
        argument: Option<&Value>,
    ) -> Vec<Value> {
        let depth = Depth::new(argument);
        let mut flattened = Vec::with_capacity(items.len());
        let mut pending = vec![items.into_iter()];

        while let Some(frame) = pending.last_mut() {
            let Some(item) = frame.next() else {
                pending.pop();
                continue;
            };
            let level = pending.len();

            if depth.reads(level) {
                self.note_nullish(&item);
            }

            match item {
                Value::List(inner) if depth.descends(level) => pending.push(inner.into_iter()),
                other => flattened.push(other),
            }
        }

        flattened
    }
}

#[cfg(test)]
mod tests;
