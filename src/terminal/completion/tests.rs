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
