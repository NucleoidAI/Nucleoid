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
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "undefined",
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
    "reduce",
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
    pub fn find(runtime: &Runtime, editor: &TextArea<'_>, explicit: bool) -> Option<Self> {
        if editor.is_selecting() {
            return None;
        }
        let (row, column) = editor.cursor();
        let line: Vec<char> = editor.lines()[row].chars().collect();
        let mut start = column;
        while start > 0 && word(line[start - 1]) {
            start -= 1;
        }
        let mut end = column;
        while end < line.len() && word(line[end]) {
            end += 1;
        }
        let prefix: String = line[start..column].iter().collect();
        if prefix.is_empty() && !explicit {
            return None;
        }
        let before: String = line[..start].iter().collect();
        if in_literal_or_comment(&line[..column]) {
            return None;
        }
        let mut candidates = BTreeSet::new();
        if row == 0 && before.trim().is_empty() && prefix.starts_with('.') {
            candidates.extend(COMMANDS.iter().map(|name| (*name).to_owned()));
        } else {
            candidates.extend(WORDS.iter().map(|name| (*name).to_owned()));
            for node in runtime.graph.nodes() {
                match node.kind {
                    NodeKind::Variable
                    | NodeKind::Function
                    | NodeKind::Object
                    | NodeKind::Property => {
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
        }
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

    pub fn select(&mut self, forward: bool) {
        let length = self.candidates.len();
        self.selected = if forward {
            (self.selected + 1) % length
        } else {
            (self.selected + length - 1) % length
        };
    }

    pub fn apply(&self, editor: &mut TextArea<'_>) {
        editor.move_cursor(CursorMove::Jump(
            u16::try_from(self.row).unwrap_or(u16::MAX),
            u16::try_from(self.start).unwrap_or(u16::MAX),
        ));
        editor.delete_str(self.end - self.start);
        editor.insert_str(&self.candidates[self.selected]);
    }
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
mod tests {
    use super::*;
    use crate::terminal::editor;

    #[test]
    fn completion_uses_runtime_bindings_aliases_and_nested_properties_without_execution() {
        let mut runtime = Runtime::new();
        runtime
            .run("person = {name: \"Ada\", address: {city: \"London\"}}\nalias = person")
            .unwrap();
        for (prefix, expected) in [
            ("ali", "alias"),
            ("alias.na", "alias.name"),
            ("alias.address.c", "alias.address.city"),
            ("Math.sq", "Math.sqrt"),
            (".ta", ".table"),
        ] {
            let completion = Completion::find(&runtime, &editor::new(prefix), false).unwrap();
            assert!(
                completion.candidates.iter().any(|name| name == expected),
                "{prefix}"
            );
        }
    }

    #[test]
    fn completion_offers_list_queries_and_instance_list_members() {
        let mut runtime = Runtime::new();
        runtime
            .run("scores = [3, 1]\nclass User\nada = User()")
            .unwrap();
        for (prefix, expected) in [
            ("scores.findL", "scores.findLast"),
            ("scores.findLastI", "scores.findLastIndex"),
            ("scores.lastI", "scores.lastIndexOf"),
            ("scores.con", "scores.concat"),
            ("scores.a", "scores.at"),
            ("User.li", "User.list"),
            ("User.al", "User.all"),
            ("User.findI", "User.findIndex"),
            ("User.sl", "User.slice"),
        ] {
            let completion = Completion::find(&runtime, &editor::new(prefix), false).unwrap();
            assert!(
                completion.candidates.iter().any(|name| name == expected),
                "{prefix}"
            );
        }
        // A plain list is already a list, so it offers neither list() nor all().
        for prefix in ["scores.li", "scores.al"] {
            assert!(
                Completion::find(&runtime, &editor::new(prefix), false).is_none_or(|completion| {
                    !completion
                        .candidates
                        .iter()
                        .any(|name| name.ends_with(".list") || name.ends_with(".all"))
                }),
                "{prefix}"
            );
        }
    }

    #[test]
    fn completing_in_the_middle_replaces_the_suffix_and_respects_character_offsets() {
        let runtime = Runtime::new();
        let mut editor = editor::new("\"界\"; Math.squ");
        editor.move_cursor(CursorMove::Back);
        let completion = Completion::find(&runtime, &editor, false).unwrap();
        completion.apply(&mut editor);
        assert_eq!(editor.lines(), ["\"界\"; Math.sqrt"]);
    }

    #[test]
    fn strings_comments_and_whitespace_do_not_trigger_automatic_completion() {
        let runtime = Runtime::new();
        for source in ["", "    ", "\"Math.sq", "# Math.sq", "value = 'ret"] {
            assert!(Completion::find(&runtime, &editor::new(source), false).is_none());
        }
        assert!(Completion::find(&runtime, &editor::new(""), true).is_some());
        assert!(Completion::find(&runtime, &editor::new("\"#\"; Math.sq"), false).is_some());
    }
}
