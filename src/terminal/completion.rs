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
            ("scores.su", "scores.sum"),
            ("User.su", "User.sum"),
            ("scores.ma", "scores.max"),
            ("scores.mi", "scores.min"),
            ("User.ma", "User.max"),
            ("User.mi", "User.min"),
            ("scores.di", "scores.distinct"),
            ("User.di", "User.distinct"),
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
                    !completion.candidates.iter().any(|name| {
                        name.rsplit_once('.')
                            .is_some_and(|(_, member)| member == "list" || member == "all")
                    })
                }),
                "{prefix}"
            );
        }
    }

    #[test]
    fn completion_offers_the_mode_query_on_lists_and_instance_lists() {
        let mut runtime = Runtime::new();
        runtime
            .run("sizes = [3, 1]\nclass Vote\nVote()\nmoderate = 1")
            .unwrap();
        for (prefix, expected) in [("sizes.mo", "sizes.mode"), ("Vote.mo", "Vote.mode")] {
            let completion = Completion::find(&runtime, &editor::new(prefix), false).unwrap();
            assert!(
                completion.candidates.iter().any(|name| name == expected),
                "{prefix}"
            );
        }
        // `mode` is a member of lists, not a name of its own.
        let completion = Completion::find(&runtime, &editor::new("mo"), false).unwrap();
        assert_eq!(completion.candidates, ["moderate"]);
    }

    #[test]
    fn completion_offers_the_extreme_item_queries_on_lists_and_instance_lists() {
        let mut runtime = Runtime::new();
        runtime
            .run("readings = [3, 1]\nclass Bid\nBid()\nmaxBytes = 1")
            .unwrap();
        for (prefix, expected) in [
            ("readings.maxB", "readings.maxBy"),
            ("readings.minB", "readings.minBy"),
            ("Bid.maxB", "Bid.maxBy"),
            ("Bid.minB", "Bid.minBy"),
        ] {
            let completion = Completion::find(&runtime, &editor::new(prefix), false).unwrap();
            assert!(
                completion.candidates.iter().any(|name| name == expected),
                "{prefix}"
            );
        }
        let completion = Completion::find(&runtime, &editor::new("maxB"), false).unwrap();
        assert_eq!(completion.candidates, ["maxBytes"]);
    }

    #[test]
    fn completion_offers_the_average_query_on_lists_and_instance_lists() {
        let mut runtime = Runtime::new();
        runtime
            .run("temps = [3, 1]\nclass Trip\nTrip()\navailable = 1")
            .unwrap();
        for (prefix, expected) in [("temps.av", "temps.average"), ("Trip.av", "Trip.average")] {
            let completion = Completion::find(&runtime, &editor::new(prefix), false).unwrap();
            assert!(
                completion.candidates.iter().any(|name| name == expected),
                "{prefix}"
            );
        }
        let completion = Completion::find(&runtime, &editor::new("av"), false).unwrap();
        assert_eq!(completion.candidates, ["available"]);
    }

    #[test]
    fn completion_offers_the_median_query_on_lists_and_instance_lists() {
        let mut runtime = Runtime::new();
        runtime
            .run("waits = [3, 1]\nclass Trip\nTrip()\nmeters = 1")
            .unwrap();
        for (prefix, expected) in [("waits.med", "waits.median"), ("Trip.med", "Trip.median")] {
            let completion = Completion::find(&runtime, &editor::new(prefix), false).unwrap();
            assert!(
                completion.candidates.iter().any(|name| name == expected),
                "{prefix}"
            );
        }
        let completion = Completion::find(&runtime, &editor::new("me"), false).unwrap();
        assert_eq!(completion.candidates, ["meters"]);
    }

    #[test]
    fn completion_offers_suppose_beside_a_variable_of_the_same_prefix() {
        let mut runtime = Runtime::new();
        runtime.run("supply = 3").unwrap();
        let completion = Completion::find(&runtime, &editor::new("sup"), false).unwrap();
        for expected in ["suppose", "supply", "super"] {
            assert!(
                completion.candidates.iter().any(|name| name == expected),
                "{expected}"
            );
        }
    }

    #[test]
    fn completion_offers_while_beside_bindings_that_share_its_prefix() {
        let mut runtime = Runtime::new();
        runtime.run("whale = 1").unwrap();
        let completion = Completion::find(&runtime, &editor::new("wh"), false).unwrap();
        assert_eq!(completion.candidates, ["whale", "while"]);
        assert!(Completion::find(&runtime, &editor::new("while"), false).is_none());
    }

    #[test]
    fn completion_offers_extends_beside_bindings_that_share_its_prefix() {
        let mut runtime = Runtime::new();
        runtime.run("extent = 4\nclass Animal").unwrap();
        let completion = Completion::find(&runtime, &editor::new("ext"), false).unwrap();
        assert_eq!(completion.candidates, ["extends", "extent"]);
        let completion =
            Completion::find(&runtime, &editor::new("class Dog extends An"), false).unwrap();
        assert_eq!(completion.candidates, ["Animal"]);
        assert!(Completion::find(&runtime, &editor::new("extends"), false).is_none());
    }

    #[test]
    fn completion_offers_nothing_after_a_selector_dot_and_no_unnamed_binding() {
        let mut runtime = Runtime::new();
        runtime
            .run("class Bid(amount): this.amount = amount\nada = Bid(5)\ntop = Bid.max(.amount)")
            .unwrap();
        for prefix in ["Bid.max(.", "Bid.max(.am", "Bid.maxBy(1, .a"] {
            assert!(
                Completion::find(&runtime, &editor::new(prefix), true).is_none(),
                "{prefix}"
            );
        }
        let completion = Completion::find(&runtime, &editor::new(""), true).unwrap();
        assert!(completion.candidates.iter().any(|name| name == "top"));
        assert!(
            completion
                .candidates
                .iter()
                .all(|name| !name.is_empty() && !name.starts_with('.'))
        );
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
