use std::collections::BTreeSet;

use nucleoid::Runtime;
use nucleoid::graph::NodeKind;
use tui_textarea::{CursorMove, TextArea};

const WORDS: &[&str] = &[
    "and",
    "assert",
    "catch",
    "class",
    "def",
    "delete",
    "elif",
    "else",
    "enum",
    "extends",
    "false",
    "for",
    "function",
    "if",
    "in",
    "lambda",
    "new",
    "not",
    "null",
    "of",
    "or",
    "pass",
    "return",
    "self",
    "super",
    "suppose",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "undefined",
    "while",
    "Array",
    "Boolean",
    "Class",
    "Date",
    "Function",
    "JSON",
    "List",
    "Math",
    "Number",
    "Object",
    "RegExp",
    "String",
];
const COMMANDS: &[&str] = &[
    ".clear", ".exit", ".graph", ".help", ".json", ".quit", ".table",
];
/// Read-only queries that lists and class instance lists share.
const LIST_MEMBERS: &[&str] = &[
    "length",
    "map",
    "filter",
    "count",
    "reduce",
    "sum",
    "average",
    "median",
    "max",
    "min",
    "distinct",
    "mode",
    "maxBy",
    "minBy",
    "some",
    "every",
    "find",
    "findIndex",
    "findLast",
    "findLastIndex",
    "includes",
    "indexOf",
    "lastIndexOf",
    "at",
    "slice",
    "concat",
    "join",
    "flatMap",
];
/// Members only a class's instance list has.
const INSTANCE_LIST_MEMBERS: &[&str] = &["list", "all"];

pub(super) struct Completion {
    pub candidates: Vec<String>,
    pub selected: usize,
    pub row: usize,
    pub start: usize,
    end: usize,
}

impl Completion {
    pub(super) fn find(runtime: &Runtime, editor: &TextArea<'_>, explicit: bool) -> Option<Self> {
        if editor.is_selecting() {
            return None;
        }
        let (row, column) = editor.cursor();
        let line: Vec<char> = editor
            .lines()
            .get(row)
            .map(|text| text.chars().collect())
            .unwrap_or_default();
        let column = column.min(line.len());
        let (head, tail) = line.split_at(column);
        let start = column
            - head
                .iter()
                .rev()
                .take_while(|&&character| word(character))
                .count();
        let end = column
            + tail
                .iter()
                .take_while(|&&character| word(character))
                .count();
        let (before, prefix) = head.split_at(start);
        let prefix: String = prefix.iter().collect();
        if prefix.is_empty() && !explicit {
            return None;
        }
        let before: String = before.iter().collect();
        if in_literal_or_comment(head) {
            return None;
        }
        let candidates = if row == 0 && before.trim().is_empty() && prefix.starts_with('.') {
            COMMANDS.iter().map(|name| (*name).to_owned()).collect()
        } else {
            names(runtime, &prefix)
        };
        let candidates: Vec<_> = candidates
            .into_iter()
            .filter(|candidate| {
                candidate.starts_with(&prefix)
                    && candidate != &prefix
                    && candidate.chars().all(word)
            })
            .collect();
        (!candidates.is_empty()).then_some(Self {
            candidates,
            selected: 0,
            row,
            start,
            end,
        })
    }

    pub(super) fn select(&mut self, forward: bool) {
        let length = self.candidates.len();
        self.selected = if forward {
            (self.selected + 1) % length
        } else {
            (self.selected + length - 1) % length
        };
    }

    pub(super) fn apply(&self, editor: &mut TextArea<'_>) {
        let Some(candidate) = self.candidates.get(self.selected) else {
            return;
        };
        editor.move_cursor(CursorMove::Jump(
            u16::try_from(self.row).unwrap_or(u16::MAX),
            u16::try_from(self.start).unwrap_or(u16::MAX),
        ));
        editor.delete_str(self.end - self.start);
        editor.insert_str(candidate);
    }
}

fn names(runtime: &Runtime, prefix: &str) -> BTreeSet<String> {
    let mut candidates: BTreeSet<String> = WORDS.iter().map(|name| (*name).to_owned()).collect();
    for node in runtime.graph.nodes() {
        match node.kind {
            NodeKind::Variable | NodeKind::Function | NodeKind::Object | NodeKind::Property => {
                candidates.insert(node.key.to_string());
            }
            NodeKind::Class => {
                candidates.insert(node.key.to_string());
                candidates.insert(node.key.as_str().trim_start_matches('$').to_owned());
            }
            _ => {}
        }
    }
    if let Some((owner, _)) = prefix.rsplit_once('.') {
        add_members(runtime, owner, &mut candidates);
    }
    candidates
}

fn word(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '_' | '$' | '.')
}

fn in_literal_or_comment(line: &[char]) -> bool {
    let mut quote = None;
    let mut escaped = false;
    for &character in line {
        if escaped {
            escaped = false;
        } else if character == '\\' && quote.is_some() {
            escaped = true;
        } else if quote == Some(character) {
            quote = None;
        } else if quote.is_none() && character == '#' {
            return true;
        } else if quote.is_none() && matches!(character, '\'' | '"' | '`') {
            quote = Some(character);
        }
    }
    quote.is_some()
}

fn add_members(runtime: &Runtime, owner: &str, candidates: &mut BTreeSet<String>) {
    let mut path = owner.split('.');
    let root = path.next().unwrap_or_default();
    let mut value = runtime.state.variable(root);
    for property in path {
        value = value.and_then(|value| match value {
            nucleoid::Value::Object(id) => runtime.state.property(id, property),
            _ => None,
        });
    }
    if let Some(nucleoid::Value::Object(id)) = value {
        if let Some(object) = runtime.state.object(id) {
            candidates.extend(
                object
                    .properties
                    .keys()
                    .map(|name| format!("{owner}.{name}")),
            );
            if let Some(class) = object
                .class
                .as_deref()
                .and_then(|name| runtime.state.class(name))
            {
                candidates.extend(class.methods.keys().map(|name| format!("{owner}.{name}")));
            }
        }
    }
    // A class name that no variable shadows is its instance list.
    let instance_list = value.is_none() && runtime.state.class(owner).is_some();
    let members: &[&str] = match owner {
        "Math" => &[
            "abs", "ceil", "floor", "round", "min", "max", "sqrt", "pow", "PI", "E", "clamp",
            "lerp",
        ],
        _ => match value {
            Some(nucleoid::Value::List(_)) => LIST_MEMBERS,
            None if instance_list => LIST_MEMBERS,
            Some(nucleoid::Value::String(_)) => &[
                "length",
                "includes",
                "indexOf",
                "slice",
                "split",
                "trim",
                "toUpperCase",
                "toLowerCase",
                "replace",
                "replaceAll",
            ],
            _ => &[],
        },
    };
    candidates.extend(members.iter().map(|name| format!("{owner}.{name}")));
    if instance_list {
        candidates.extend(
            INSTANCE_LIST_MEMBERS
                .iter()
                .map(|name| format!("{owner}.{name}")),
        );
    }
}

#[cfg(test)]
mod tests;
