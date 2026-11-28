mod common;

use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use nucleoid::lang::ast::lexer::{Keyword, Spanned, Token, tokenize};
use nucleoid::lang::ast::parse;
use nucleoid::{ErrorKind, Runtime};
use serde::Deserialize;

const SPEC_TITLES: [&str; 4] = [
    "Nucleoid removes orders and every property they hold from their population",
    "Nucleoid removes departing cars with the properties class level rules derive",
    "Nucleoid removes subclass instances with the properties their parent's rules derive",
    "Nucleoid restores removed instances when the update that removes them fails",
];
const REFERENCE_TITLE: &str =
    "Nucleoid removes an instance with every property it holds through its class";
const README_TITLE: &str = "Removing a car keeps a parking lot's free spaces live";
const EXAMPLES_TITLE: &str = "Removing shipped orders keeps a warehouse's pick list live";

const ORDER_HELPER: &str = "drop = o -> delete o.total && delete o.price && delete o.qty && delete o.item && delete Order[o.id]";
const TASK_HELPER: &str = "drop = o -> delete o.cost && delete o.owner && delete o.hours && delete o.name && delete Task[o.id]";

struct Counterpart {
    title: &'static str,
    rewrites: &'static [(&'static str, &'static str)],
    before: usize,
    after: usize,
}

const COUNTERPARTS: [Counterpart; 7] = [
    Counterpart {
        title: SPEC_TITLES[0],
        rewrites: &[
            ("Order.remove(tea)", "HELPER\ndrop(tea)"),
            ("Order.remove(rye)", "drop(rye)"),
        ],
        before: 304,
        after: 276,
    },
    Counterpart {
        title: SPEC_TITLES[1],
        rewrites: &[(
            "Car.remove(bus)",
            "delete bus.ticket\ndelete bus.fee\ndelete bus.hours\ndelete bus.plate\ndelete bus",
        )],
        before: 235,
        after: 219,
    },
    Counterpart {
        title: SPEC_TITLES[2],
        rewrites: &[
            (
                "Truck.remove(rig)",
                "delete rig.toll\ndelete rig.axles\ndelete rig.plate\ndelete rig",
            ),
            (
                "Van.remove(camper)",
                "delete camper.toll\ndelete camper.axles\ndelete camper.plate\ndelete camper",
            ),
        ],
        before: 213,
        after: 191,
    },
    Counterpart {
        title: SPEC_TITLES[3],
        rewrites: &[
            ("failure = \"\"\ntry:", "failure = \"\"\nHELPER\ntry:"),
            ("Task.remove(build)", "drop(build)"),
            ("Task.remove(plan)", "drop(plan)"),
            ("Task.remove(review)", "drop(review)"),
            ("Task.remove(review)", "drop(review)"),
        ],
        before: 311,
        after: 287,
    },
    Counterpart {
        title: REFERENCE_TITLE,
        rewrites: &[(
            "Robot.remove(r2)",
            "delete r2.low\ndelete r2.battery\ndelete r2.name\ndelete r2",
        )],
        before: 160,
        after: 149,
    },
    Counterpart {
        title: README_TITLE,
        rewrites: &[(
            "Car.remove(cab)",
            "delete cab.level\ndelete cab.plate\ndelete cab",
        )],
        before: 145,
        after: 139,
    },
    Counterpart {
        title: EXAMPLES_TITLE,
        rewrites: &[(
            "        Order.remove(order)",
            "        delete order.boxes\n        delete order.shipped\n        delete order.qty\n        delete order.item\n        delete Order[order.id]",
        )],
        before: 217,
        after: 196,
    },
];

const ROBOTS: &str =
    "class Robot(name):\n    this.name = name\na = Robot(\"a\")\nb = Robot(\"b\")\n";

fn documents() -> [(&'static str, Vec<common::Case>); 4] {
    [
        ("spec", common::cases(include_str!("../nucleoid.spec.md"))),
        ("reference", common::cases(include_str!("reference.md"))),
        (
            "readme",
            common::snippets(include_str!("../README.md"), "README.md"),
        ),
        (
            "examples",
            common::snippets(include_str!("../docs/examples.md"), "docs/examples.md"),
        ),
    ]
}

fn is_appended(title: &str) -> bool {
    SPEC_TITLES.contains(&title) || [REFERENCE_TITLE, README_TITLE, EXAMPLES_TITLE].contains(&title)
}

fn interpolations(raw: &str) -> Vec<&str> {
    let mut found = Vec::new();
    let mut rest = raw;
    while let Some((_, after)) = rest.split_once("${") {
        let mut depth = 0_usize;
        let mut end = after.len();
        for (index, character) in after.char_indices() {
            match character {
                '{' => depth += 1,
                '}' if depth == 0 => {
                    end = index;
                    break;
                }
                '}' => depth -= 1,
                _ => {}
            }
        }
        found.push(&after[..end]);
        rest = after.get(end + 1..).unwrap_or_default();
    }
    found
}

fn tokens(source: &str) -> Vec<Token> {
    let mut found = Vec::new();
    for spanned in tokenize(source).unwrap() {
        if let Token::Template(raw) = &spanned.token {
            for interpolation in interpolations(raw) {
                found.extend(tokens(interpolation));
            }
        }
        found.push(spanned.token);
    }
    found
}

fn calls_remove(source: &str) -> bool {
    tokens(source).windows(3).any(|window| {
        window[0] == Token::Dot
            && window[1] == Token::Identifier("remove".to_string())
            && window[2] == Token::LeftParen
    })
}

fn touches(first: &Spanned, second: &Spanned) -> bool {
    second.position.line == first.position.line
        && second.position.column == first.position.column + 1
}

fn writes_range(source: &str) -> bool {
    tokenize(source).is_ok_and(|tokens| {
        tokens.windows(3).any(|window| {
            window[0].token == Token::Dot
                && window[1].token == Token::Dot
                && matches!(window[2].token, Token::Less | Token::Assign)
                && touches(&window[0], &window[1])
                && touches(&window[1], &window[2])
        })
    })
}

fn calls_padding_alias(source: &str) -> bool {
    tokenize(source).is_ok_and(|tokens| {
        tokens.windows(3).any(|window| {
            window[0].token == Token::Dot
                && matches!(&window[1].token, Token::Identifier(name) if ["rjust", "ljust", "PadLeft", "PadRight"].contains(&name.as_str()))
                && window[2].token == Token::LeftParen
        })
    })
}

fn writes_xor_stage(source: &str) -> bool {
    tokenize(source).is_ok_and(|tokens| {
        tokens.windows(3).any(|window| {
            window[0].token == Token::Pipe
                && window[1].token == Token::Identifier("xor".to_string())
                && window[2].token == Token::LeftParen
        })
    })
}

fn writes_bang_in(source: &str) -> bool {
    tokenize(source).is_ok_and(|tokens| {
        tokens.windows(2).any(|pair| {
            pair[0].token == Token::Bang
                && pair[1].token == Token::Keyword(Keyword::In)
                && touches(&pair[0], &pair[1])
        })
    })
}

fn lexer_tokens(source: &str) -> usize {
    tokenize(source.trim())
        .unwrap()
        .iter()
        .filter(|spanned| spanned.token != Token::Eof)
        .count()
}

fn renumbered(text: &str) -> String {
    let mut seen: Vec<&str> = Vec::new();
    let mut result = String::new();
    let mut rest = text;
    while let Some((before, after)) = rest.split_once("obj:") {
        let digits = after
            .find(|character: char| !character.is_ascii_digit())
            .unwrap_or(after.len());
        let (identifier, tail) = after.split_at(digits);
        let ordinal = match seen.iter().position(|known| *known == identifier) {
            Some(ordinal) => ordinal,
            None => {
                seen.push(identifier);
                seen.len() - 1
            }
        };
        result.push_str(before);
        result.push_str("obj:");
        result.push_str(&ordinal.to_string());
        rest = tail;
    }
    result.push_str(rest);
    result
}

fn outcome(runtime: &mut Runtime, source: &str) -> String {
    match runtime.run(source) {
        Ok(value) => renumbered(&runtime.serialize_json(&value).unwrap()),
        Err(error) => renumbered(&error.to_string()),
    }
}

fn fresh(source: &str) -> String {
    outcome(&mut Runtime::new(), source)
}

fn result(source: &str) -> String {
    let mut runtime = Runtime::new();
    let value = runtime
        .run(source)
        .unwrap_or_else(|error| panic!("{source}: {error}"));
    assert!(runtime.take_assertions().is_empty(), "{source}");
    renumbered(&runtime.serialize_json(&value).unwrap())
}

fn failure(source: &str) -> String {
    renumbered(
        &Runtime::new()
            .run(source)
            .expect_err("the program raises an error")
            .to_string(),
    )
}

fn spelled(case: &common::Case, counterpart: &Counterpart) -> String {
    let mut source = case.source.clone();
    for (written, spelling) in counterpart.rewrites {
        assert!(source.contains(written), "{}: {written}", counterpart.title);
        let helper = if counterpart.title == SPEC_TITLES[0] {
            ORDER_HELPER
        } else {
            TASK_HELPER
        };
        source = source.replacen(written, &spelling.replace("HELPER", helper), 1);
    }
    source
}

#[test]
fn only_the_appended_cases_call_remove_and_they_run_every_assertion() {
    let documents = documents();
    let mut cases = 0;
    let mut assertions = 0;
    for case in documents.iter().flat_map(|(_, cases)| cases) {
        cases += 1;
        let new = is_appended(&case.title);
        assert_eq!(calls_remove(&case.source), new, "{}", case.title);
        if new {
            let count =
                common::assertions::protected_assertions(&parse(&case.source).unwrap()).len();
            assert!(count > 0, "{} asserts its behavior", case.title);
            assert!(case.expected.is_some(), "{} declares a return", case.title);
            let mut runtime = Runtime::new();
            common::check_with_count(&mut runtime, case, count)
                .unwrap_or_else(|reason| panic!("{}: {reason}", case.title));
            assert_eq!(runtime.assertions_run(), count, "{}", case.title);
            assertions += count;
        }
    }
    for title in SPEC_TITLES
        .into_iter()
        .chain([REFERENCE_TITLE, README_TITLE, EXAMPLES_TITLE])
    {
        let matching = documents
            .iter()
            .flat_map(|(_, cases)| cases)
            .filter(|case| case.title == title)
            .count();
        assert_eq!(matching, 1, "{title}");
    }
    assert_eq!(assertions, 24);
    assert_eq!(cases, 1355);
}

#[test]
fn every_appended_case_gives_what_its_counterpart_gives() {
    let documents = documents();
    for counterpart in &COUNTERPARTS {
        let case = documents
            .iter()
            .flat_map(|(_, cases)| cases)
            .find(|case| case.title == counterpart.title)
            .unwrap_or_else(|| panic!("{} is appended", counterpart.title));
        let accepted = spelled(case, counterpart);
        assert!(!calls_remove(&accepted), "{}", counterpart.title);
        assert_eq!(
            lexer_tokens(&case.source),
            counterpart.after,
            "{}",
            counterpart.title
        );
        assert_eq!(
            lexer_tokens(&accepted),
            counterpart.before,
            "{}",
            counterpart.title
        );
        let assertions = common::assertions::protected_assertions(&parse(&case.source).unwrap());
        let accepted_assertions =
            common::assertions::protected_assertions(&parse(&accepted).unwrap());
        assert_eq!(assertions, accepted_assertions, "{}", counterpart.title);
        let mut outputs = Vec::new();
        for source in [&case.source, &accepted] {
            let mut runtime = Runtime::new();
            let value = common::check_with_count(
                &mut runtime,
                &common::Case {
                    title: case.title.clone(),
                    source: source.clone(),
                    expected: case.expected.clone(),
                },
                assertions.len(),
            )
            .unwrap_or_else(|reason| panic!("{}: {reason}", counterpart.title));
            outputs.push(runtime.serialize_json(&value).unwrap());
        }
        assert_eq!(outputs[0], outputs[1], "{}", counterpart.title);
    }
}

#[test]
fn the_audit_finds_every_remove_call_and_nothing_else() {
    for source in [
        "Robot.remove(a)",
        "R.remove(a)",
        "Robot.remove()",
        "Robot.remove (a)",
        "x = Robot.remove(a, b)",
        "xs.remove(1)",
        "f(Robot.remove(a))",
        "`${Robot.remove(a)} gone`",
        "if a.x:\n    Robot.remove(a)",
    ] {
        assert!(calls_remove(source), "{source}");
    }
    for source in [
        "Robot.remove",
        "f = Robot.remove",
        "remove(1)",
        "remove = 4",
        "def remove(x) => x + 1",
        "o = {remove: 1}",
        "\"Robot.remove(a)\"",
        "# Robot.remove(a)\nx = 1",
        "Robot.removeAll(a)",
        "Robot.Remove(a)",
        "Robot[\"remove\"](a)",
        "delete a",
    ] {
        assert!(!calls_remove(source), "{source}");
    }
}

#[test]
fn the_appended_cases_stand_together_after_nuc_148_and_before_nuc_157() {
    let [(_, spec), (_, reference), (_, readme), (_, examples)] = documents();
    for (cases, own) in [
        (&spec, &SPEC_TITLES[..]),
        (&reference, &[REFERENCE_TITLE][..]),
        (&readme, &[README_TITLE][..]),
        (&examples, &[EXAMPLES_TITLE][..]),
    ] {
        let first = cases
            .iter()
            .position(|case| case.title == own[0])
            .unwrap_or_else(|| panic!("{} is appended", own[0]));
        for (offset, title) in own.iter().enumerate() {
            assert_eq!(
                cases.get(first + offset).map(|case| case.title.as_str()),
                Some(*title),
                "{title} stands with the other appended cases"
            );
        }
        let ranges = cases
            .iter()
            .rposition(|case| writes_range(&case.source))
            .expect("the cases of NUC 148 stand in every document");
        let padding = cases
            .iter()
            .position(|case| calls_padding_alias(&case.source))
            .expect("the cases of NUC 157 stand in every document");
        let xor = cases
            .iter()
            .position(|case| writes_xor_stage(&case.source))
            .expect("the cases of NUC 139 stand in every document");
        let bang_in = cases
            .iter()
            .position(|case| writes_bang_in(&case.source))
            .expect("the cases of NUC 135 stand in every document");
        assert!(ranges < first, "{}", own[0]);
        assert!(first + own.len() <= padding, "{}", own[0]);
        assert!(first + own.len() <= xor, "{}", own[0]);
        assert!(first + own.len() <= bang_in, "{}", own[0]);
    }
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### Population instance removal\n\n```\n")
        .expect("the language reference documents Name.remove");
    let (example, _) = section.split_once("```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is in the executable reference");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert_eq!(case.expected.as_deref(), Some("2"));
    for line in [
        "[NUC 162](nuc-0162.md)",
        "Name.remove(instance)                  delete every property of an instance, then the instance (NUC 162)",
        "[removal contract](../nucleoid.spec.md#population-instance-removal)",
    ] {
        assert!(reference.contains(line), "{line}");
    }
    assert!(include_str!("../README.md").contains("[NUC 162](docs/nuc-0162.md)"));
    assert!(include_str!("../docs/examples.md").contains("[NUC 162](nuc-0162.md)"));
    let specification = include_str!("../nucleoid.spec.md").replace("\r\n", "\n");
    let (_, contract) = specification
        .split_once("\n## Population instance removal\n")
        .expect("the specification states Name.remove");
    let (contract, _) = contract.split_once("\n## ").unwrap();
    let [first, .., last] = SPEC_TITLES;
    for title in [first, last] {
        assert!(contract.contains(&format!("`{title}`")), "{title}");
    }
    assert!(contract.contains("`tests/population_removal.rs`"));
    assert!(contract.contains("[NUC 162](docs/nuc-0162.md)"));
}

#[test]
fn every_first_argument_outside_the_population_raises_before_anything_is_deleted() {
    let setup = "class Robot(name):\n    this.name = name\nclass Crate(n):\n    this.n = n\nclass Animal(name):\n    this.name = name\nclass Dog extends Animal\na = Robot(\"a\")\nb = Robot(\"b\")\nc = Crate(1)\ncat = Animal(\"cat\")\nrex = Dog(\"rex\")\no = {}\ngone = Robot(\"gone\")\nheld = []\nheld.push(gone)\nRobot.remove(gone)\n";
    for (call, class) in [
        ("Robot.remove()", "Robot"),
        ("Robot.remove(null)", "Robot"),
        ("Robot.remove(undefined)", "Robot"),
        ("Robot.remove(o.missing)", "Robot"),
        ("Robot.remove(5)", "Robot"),
        ("Robot.remove(\"a\")", "Robot"),
        ("Robot.remove(true)", "Robot"),
        ("Robot.remove(Date(1))", "Robot"),
        ("Robot.remove(/a/)", "Robot"),
        ("Robot.remove([a])", "Robot"),
        ("Robot.remove({})", "Robot"),
        ("Robot.remove({id: \"a\", name: \"a\"})", "Robot"),
        ("Robot.remove(Robot)", "Robot"),
        ("Robot.remove(x -> x)", "Robot"),
        ("Robot.remove(c)", "Robot"),
        ("Robot.remove(held[0])", "Robot"),
        ("Animal.remove(rex)", "Animal"),
        ("Dog.remove(cat)", "Dog"),
    ] {
        let program = format!(
            "{setup}failure = \"\"\ntry:\n    {call}\ncatch error:\n    failure = error\n[failure, Robot.length, Animal.length, Dog.length, a.name, rex.name, cat.name]"
        );
        assert_eq!(
            result(&program),
            format!(
                "[\"TypeError: remove expects an instance of {class}\",2.0,1.0,1.0,\"a\",\"rex\",\"cat\"]"
            ),
            "{call}"
        );
    }
    let error = Runtime::new()
        .run(&format!("{ROBOTS}x = 1\nRobot.remove(5)"))
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Type);
    assert_eq!(error.message(), "remove expects an instance of Robot");
    assert_eq!(
        error.to_string(),
        "TypeError: remove expects an instance of Robot (line 6, column 1)"
    );
    assert_eq!(
        failure("class Box\nb = Box()\nR = Box\nR.remove(1)"),
        "TypeError: remove expects an instance of Box (line 4, column 1)"
    );
}

#[test]
fn membership_agrees_with_in_for_every_instance_and_value() {
    let setup = "class Vehicle(plate):\n    this.plate = plate\nclass Truck extends Vehicle\nrig = Truck(\"R\")\ncar = Vehicle(\"C\")\nkept = [rig]\no = {}\n";
    let mut members = 0;
    for (argument, class) in [
        ("rig", "Truck"),
        ("rig", "Vehicle"),
        ("car", "Vehicle"),
        ("car", "Truck"),
        ("kept[0]", "Truck"),
        ("Truck[0]", "Truck"),
        ("Vehicle[0]", "Vehicle"),
        ("\"R\"", "Truck"),
        ("{id: \"rig\"}", "Truck"),
        ("5", "Truck"),
        ("null", "Truck"),
        ("o.missing", "Vehicle"),
        ("[rig]", "Truck"),
    ] {
        let inside = result(&format!("{setup}seen = {argument} in {class}\nseen"));
        let removed = fresh(&format!(
            "{setup}{class}.remove({argument})\n[Truck.length, Vehicle.length]"
        ));
        let expected = if inside != "true" {
            format!("TypeError: remove expects an instance of {class} (line 8, column 1)")
        } else if class == "Truck" {
            "[0.0,1.0]".to_string()
        } else {
            "[1.0,0.0]".to_string()
        };
        members += usize::from(inside == "true");
        assert_eq!(removed, expected, "{argument} {class}: {inside}");
    }
    assert_eq!(members, 5);
    let structural = "class Robot(name):\n    this.name = name\na = Robot(\"a\")\n";
    assert_eq!(
        result(&format!(
            "{structural}seen = {{id: \"a\", name: \"a\"}} in Robot\nseen"
        )),
        "true"
    );
    assert_eq!(
        failure(&format!(
            "{structural}Robot.remove({{id: \"a\", name: \"a\"}})"
        )),
        "TypeError: remove expects an instance of Robot (line 4, column 1)"
    );
}

#[test]
fn arguments_are_evaluated_once_in_order_and_their_errors_come_first() {
    assert_eq!(
        result(&format!(
            "{ROBOTS}log = []\ndef pick(r) => [log.push(\"first\"), r][1]\ndef note(x) => log.push(x)\nRobot.remove(pick(a), note(\"second\"), note(\"third\"))\n[log, Robot.length]"
        )),
        "[[\"first\",\"second\",\"third\"],1.0]"
    );
    assert_eq!(
        failure(&format!("{ROBOTS}Robot.remove(missing)")),
        "ReferenceError: missing is not defined (line 5, column 1)"
    );
    assert_eq!(
        result(&format!(
            "{ROBOTS}failure = \"\"\ntry:\n    Robot.remove(a, log.push(1))\ncatch error:\n    failure = error\n[failure, Robot.length, a.name]"
        )),
        "[\"ReferenceError: log is not defined\",2.0,\"a\"]"
    );
    assert_eq!(
        result(&format!(
            "{ROBOTS}Robot.remove(Robot(\"z\"))\nRobot.map(r -> r.name)"
        )),
        "[\"a\",\"b\"]"
    );
    assert_eq!(
        result(&format!(
            "{ROBOTS}[Robot.remove(a), typeof Robot.remove(b), Robot.length]"
        )),
        "[null,\"undefined\",0.0]"
    );
    assert_eq!(
        result(&format!("{ROBOTS}x = Robot.remove(a)\n[x, Robot.length]")),
        "[null,1.0]"
    );
}

#[test]
fn removals_leave_the_state_that_the_deletions_leave() {
    for (program, spelling, expected) in [
        (
            "class Empty\ne = Empty()\nf = Empty()\nEmpty.remove(e)\n[Empty.length, Empty[\"e\"], Empty[\"f\"].id]",
            "class Empty\ne = Empty()\nf = Empty()\ndelete e\n[Empty.length, Empty[\"e\"], Empty[\"f\"].id]",
            "[1.0,null,\"f\"]",
        ),
        (
            "class Robot(name):\n    this.name = name\na = Robot(\"a\")\nb = Robot(\"b\")\nRobot.remove(a)\n[Robot.length, Robot.map(r -> r.name)]",
            "class Robot(name):\n    this.name = name\na = Robot(\"a\")\nb = Robot(\"b\")\ndelete a.name\ndelete a\n[Robot.length, Robot.map(r -> r.name)]",
            "[1.0,[\"b\"]]",
        ),
        (
            "class Point(x, y):\n    this.x = x\n    this.y = y\np = Point(1, 2)\nq = Point(3, 4)\np.z = 5\nPoint.remove(p)\nPoint.map(t -> [t.x, t.y, t.z])",
            "class Point(x, y):\n    this.x = x\n    this.y = y\np = Point(1, 2)\nq = Point(3, 4)\np.z = 5\ndelete p.z\ndelete p.y\ndelete p.x\ndelete p\nPoint.map(t -> [t.x, t.y, t.z])",
            "[[3.0,4.0,null]]",
        ),
        (
            "class Car(plate, hours):\n    this.plate = plate\n    this.hours = hours\n$Car.fee = $Car.hours * 3\n$Car.ticket = $Car.plate + \" \" + $Car.fee\na = Car(\"A\", 2)\nb = Car(\"B\", 5)\nboard = Car.map(c -> c.ticket)\nCar.remove(a)\n[board, Car.length]",
            "class Car(plate, hours):\n    this.plate = plate\n    this.hours = hours\n$Car.fee = $Car.hours * 3\n$Car.ticket = $Car.plate + \" \" + $Car.fee\na = Car(\"A\", 2)\nb = Car(\"B\", 5)\nboard = Car.map(c -> c.ticket)\ndelete a.ticket\ndelete a.fee\ndelete a.hours\ndelete a.plate\ndelete a\n[board, Car.length]",
            "[[\"B 15\"],1.0]",
        ),
        (
            "class Engine(power):\n    this.power = power\nclass Car(plate):\n    this.plate = plate\nc = Car(\"C1\")\nc.engine = Engine(90)\nCar.remove(c)\n[Car.length, Engine.length, Engine.map(e -> e.power)]",
            "class Engine(power):\n    this.power = power\nclass Car(plate):\n    this.plate = plate\nc = Car(\"C1\")\nc.engine = Engine(90)\ndelete c.engine\ndelete c.plate\ndelete c\n[Car.length, Engine.length, Engine.map(e -> e.power)]",
            "[0.0,1.0,[90.0]]",
        ),
        (
            "class Robot(name):\n    this.name = name\nRobot(\"x\")\nRobot(\"y\")\nRobot.remove(Robot[0])\nRobot.map(r -> r.name)",
            "class Robot(name):\n    this.name = name\nRobot(\"x\")\nRobot(\"y\")\ndelete Robot[0].name\ndelete Robot[0]\nRobot.map(r -> r.name)",
            "[\"y\"]",
        ),
    ] {
        assert_eq!(result(program), expected, "{program}");
        assert_eq!(result(spelling), expected, "{spelling}");
        assert!(!calls_remove(spelling), "{spelling}");
    }
}

#[test]
fn properties_go_from_the_most_recently_added_to_the_first() {
    let setup = "class Tag(name):\n    this.name = name\n$Tag.label = $Tag.name + \"!\"\na = Tag(\"a\")\nb = Tag(\"b\")\nlog = []\nif a.label == null:\n    log.push(a.name)\n";
    let removed = result(&format!("{setup}Tag.remove(a)\n[log, Tag.length]"));
    let latest_first = result(&format!(
        "{setup}delete a.label\ndelete a.name\ndelete a\n[log, Tag.length]"
    ));
    let creation_order = result(&format!(
        "{setup}delete a.name\ndelete a.label\ndelete a\n[log, Tag.length]"
    ));
    assert_eq!(removed, latest_first);
    assert_ne!(removed, creation_order);
    assert_eq!(removed, "[[\"a\",null,null],1.0]");
}

#[test]
fn the_removal_deletes_what_its_deletions_delete_and_nothing_more() {
    let gone_twice = "class Robot(name, x):\n    this.name = name\n    this.x = x\na = Robot(\"a\", 1)\nb = Robot(\"b\", 2)\nlog = []\nif a.x == null:\n    delete a.name\nif a.name == null:\n    log.push(\"name gone\")\n";
    let removed = result(&format!("{gone_twice}Robot.remove(a)\n[log, Robot.length]"));
    assert_eq!(
        removed,
        result(&format!(
            "{gone_twice}delete a.x\ndelete a.name\ndelete a\n[log, Robot.length]"
        ))
    );
    assert_eq!(removed, "[[\"name gone\",\"name gone\"],1.0]");
    let identity = "class Robot(name):\n    this.name = name\na = Robot(\"a\")\nb = Robot(\"b\")\nlog = []\nif a.id == null:\n    log.push(\"id gone\")\n";
    let kept = result(&format!("{identity}Robot.remove(a)\n[log, Robot.length]"));
    assert_eq!(
        kept,
        result(&format!(
            "{identity}delete a.name\ndelete a\n[log, Robot.length]"
        ))
    );
    assert_ne!(
        kept,
        result(&format!(
            "{identity}delete a.name\ndelete a.id\ndelete a\n[log, Robot.length]"
        ))
    );
    assert_eq!(kept, "[[\"id gone\"],1.0]");
}

#[test]
fn the_instance_leaves_as_an_index_deletion_removes_it() {
    assert_eq!(
        failure(&format!("{ROBOTS}Robot.remove(a)\na")),
        "ReferenceError: a is not defined (line 6, column 1)"
    );
    for (tail, expected) in [
        ("r = a\nRobot.remove(r)\n[Robot.length, r]", "[1.0,null]"),
        ("xs = [a]\nRobot.remove(a)\nxs", "[null]"),
        (
            "held = []\nheld.push(a)\nRobot.remove(a)\n[held.length, held[0].name, held[0] in Robot]",
            "[1.0,null,false]",
        ),
        (
            "label = a.name\ncount = Robot.length\nRobot.remove(a)\n[label, count, Robot[\"a\"], Robot[0].name]",
            "[null,1.0,null,\"b\"]",
        ),
        (
            "Robot.remove(a)\na = Robot(\"again\")\n[Robot.length, a.name, Robot.map(r -> r.name)]",
            "[2.0,\"again\",[\"b\",\"again\"]]",
        ),
    ] {
        let program = format!("{ROBOTS}{tail}");
        let spelling = program.replace("Robot.remove(a)", "delete a.name\ndelete Robot[0]");
        let spelling = spelling.replace("Robot.remove(r)", "delete r.name\ndelete Robot[0]");
        assert_eq!(result(&program), expected, "{program}");
        assert_eq!(result(&spelling), expected, "{spelling}");
    }
}

#[test]
fn subclass_and_parent_instances_leave_through_their_own_class() {
    let program = "class Vehicle(plate, axles):\n    this.plate = plate\n    this.axles = axles\n$Vehicle.toll = $Vehicle.axles * 2\nclass Truck extends Vehicle\nrig = Truck(\"R\", 5)\nhauler = Truck(\"H\", 3)\ncar = Vehicle(\"C\", 2)\ntolls = Truck.map(t -> t.toll)\nTruck.remove(rig)\n[tolls, Truck.length, Vehicle.length, car.toll]";
    assert_eq!(result(program), "[[6.0],1.0,1.0,4.0]");
    let spelling = program.replace(
        "Truck.remove(rig)",
        "delete rig.toll\ndelete rig.axles\ndelete rig.plate\ndelete rig",
    );
    assert_eq!(result(&spelling), result(program));
    for (call, class) in [
        ("Vehicle.remove(rig)", "Vehicle"),
        ("Truck.remove(car)", "Truck"),
    ] {
        let attempt = program.replace("Truck.remove(rig)", call);
        assert_eq!(
            failure(&attempt),
            format!("TypeError: remove expects an instance of {class} (line 10, column 1)"),
            "{call}"
        );
    }
}

#[test]
fn receivers_are_those_of_every_population_query() {
    assert_eq!(
        result(&format!(
            "{ROBOTS}R = Robot\nR.remove(a)\n[Robot.length, R.length]"
        )),
        "[1.0,1.0]"
    );
    for (tail, expected) in [
        (
            "Robot = [1, 2]\nRobot.remove(1)",
            "TypeError: Robot.remove is not a function (line 6, column 1)",
        ),
        (
            "def f(Robot) => Robot.remove(1)\nf([1])",
            "TypeError: Robot.remove is not a function (line 6, column 1)",
        ),
        (
            "Robot.list().remove(a)",
            "TypeError: Robot.list().remove is not a function (line 5, column 1)",
        ),
        (
            "Robot.removeAll(a)",
            "TypeError: Robot.removeAll is not a function (line 5, column 1)",
        ),
        (
            "Robot.Remove(a)",
            "TypeError: Robot.Remove is not a function (line 5, column 1)",
        ),
        (
            "Math.remove(1)",
            "TypeError: Math.remove is not a function (line 5, column 1)",
        ),
        (
            "String.remove(\"a\")",
            "TypeError: String.remove is not a function (line 5, column 1)",
        ),
        (
            "Rover.remove(a)",
            "ReferenceError: Rover is not defined (line 5, column 1)",
        ),
        (
            "Class.remove(Robot)",
            "ReferenceError: Class is not defined (line 5, column 1)",
        ),
        (
            "Class.remove(missing)",
            "ReferenceError: Class is not defined (line 5, column 1)",
        ),
    ] {
        assert_eq!(failure(&format!("{ROBOTS}{tail}")), expected, "{tail}");
    }
    assert_eq!(
        result(&format!(
            "{ROBOTS}log = []\nfailure = \"\"\ntry:\n    Class.remove(log.push(1))\ncatch error:\n    failure = error\n[failure, log]"
        )),
        "[\"ReferenceError: Class is not defined\",[]]"
    );
}

#[test]
fn dependents_follow_the_removal() {
    let program = "class Order(item, qty, price):\n    this.item = item\n    this.qty = qty\n    this.price = price\n$Order.total = $Order.qty * $Order.price\ntea = Order(\"tea\", 2, 4)\njam = Order(\"jam\", 1, 6)\npending = Order.length\nitems = Order.map(o -> o.item)\nlarge = Order.filter(o -> o.total > 5).map(o -> o.item)\nfirst = tea.total\nOrder.remove(tea)\nOrder(\"rye\", 3, 5)\njam.qty = 4\n[pending, items, large, first, jam.total]";
    assert_eq!(
        result(program),
        "[2.0,[\"jam\",\"rye\"],[\"jam\",\"rye\"],null,24.0]"
    );
    let spelling = program.replace(
        "Order.remove(tea)",
        "delete tea.total\ndelete tea.price\ndelete tea.qty\ndelete tea.item\ndelete tea",
    );
    assert_eq!(result(&spelling), result(program));
    let explained = format!("{ROBOTS}count = Robot.length\nRobot.remove(a)\nwhy count");
    assert_eq!(
        result(&explained),
        result(&explained.replace("Robot.remove(a)", "delete a.name\ndelete a"))
    );
    assert_eq!(
        result(&format!(
            "{ROBOTS}count = Robot.length\nRobot.remove(a)\n(why count)[0].holds"
        )),
        "1.0"
    );
}

#[test]
fn failed_removals_restore_everything() {
    for (tail, expected) in [
        (
            "ratio = Robot.length > 1 ? 1 : missing\nfailure = \"\"\ntry:\n    Robot.remove(a)\ncatch error:\n    failure = error\n[failure, Robot.length, a.name, ratio]",
            "[\"ReferenceError: missing is not defined\",2.0,\"a\",1.0]",
        ),
        (
            "names = Robot.map(r -> r.name)\ntry:\n    Robot.remove(a)\n    throw \"UNDO\"\ncatch error:\n    pass\n[names, Robot.length, a.name]",
            "[[\"a\",\"b\"],2.0,\"a\"]",
        ),
        (
            "failure = \"\"\ntry:\n    Robot.remove(a)\n    Robot.remove(b)\n    reveiw.hours = 1\ncatch error:\n    failure = error\n[failure, Robot.map(r -> r.name), a.name, b.name]",
            "[\"ReferenceError: reveiw is not defined\",[\"a\",\"b\"],\"a\",\"b\"]",
        ),
        (
            "seen = suppose:\n    Robot.remove(a)\n    Robot.length\n[seen, Robot.length, a.name]",
            "[1.0,2.0,\"a\"]",
        ),
    ] {
        assert_eq!(result(&format!("{ROBOTS}{tail}")), expected, "{tail}");
    }
    let readded = "class Robot(name, x):\n    this.name = name\n    this.x = x\na = Robot(\"a\", 1)\nb = Robot(\"b\", 2)\nif a.x == null:\n    a.status = \"gone\"\nfailure = \"\"\ntry:\n    Robot.remove(a)\ncatch error:\n    failure = error\n[failure, Robot.length, a.name, a.x, a.status]";
    assert_eq!(
        result(readded),
        "[\"TypeError: Cannot delete object 'a'\",2.0,\"a\",1.0,null]"
    );
    assert_eq!(
        result(&readded.replace(
            "    Robot.remove(a)",
            "    delete a.x\n    delete a.name\n    delete a"
        )),
        result(readded)
    );
}

#[test]
fn removals_run_once_where_statements_run_and_stand_where_declarations_stand() {
    for (tail, expected) in [
        (
            "def retire(r) => Robot.remove(r)\nretire(b)\nRobot.map(r -> r.name)",
            "[\"a\"]",
        ),
        (
            "class Depot:\n    def retire(r) => Robot.remove(r)\ndepot = Depot()\ndepot.retire(a)\nRobot.map(r -> r.name)",
            "[\"b\"]",
        ),
        (
            "retire = r -> Robot.remove(r)\nretire(a)\nRobot.map(r -> r.name)",
            "[\"b\"]",
        ),
        ("[a, b].map(r -> Robot.remove(r))\nRobot.length", "0.0"),
        (
            "for robot in Robot:\n    if robot.name == \"a\":\n        Robot.remove(robot)\nRobot.map(r -> r.name)",
            "[\"b\"]",
        ),
        (
            "Robot(\"c\")\nif Robot.length > 2:\n    Robot.remove(Robot[0])\nd = Robot(\"d\")\nRobot.map(r -> r.name)",
            "[\"c\",\"d\"]",
        ),
        (
            "def retire(r):\n    delete r.name\n    delete r\nretire(b)\nRobot.map(r -> r.name)",
            "[\"a\",null]",
        ),
    ] {
        assert_eq!(result(&format!("{ROBOTS}{tail}")), expected, "{tail}");
    }
    let declared = format!(
        "{ROBOTS}gone = Robot.remove(a)\nfailure = \"\"\ntry:\n    Robot(\"c\")\ncatch error:\n    failure = error\n[failure, Robot.length, gone]"
    );
    assert_eq!(
        result(&declared),
        "[\"TypeError: remove expects an instance of Robot\",1.0,null]"
    );
    assert_eq!(
        failure(&format!("{ROBOTS}gone = Robot.remove(a)\nRobot(\"c\")")),
        "TypeError: remove expects an instance of Robot (line 6, column 1)"
    );
    let rule = "class Robot(name):\n    this.name = name\nclass Tag(owner):\n    this.owner = owner\na = Robot(\"a\")\nb = Robot(\"b\")\nt = Tag(a)\n$Tag.gone = Robot.remove($Tag.owner)\n[Robot.length, Robot.map(r -> r.name)]";
    assert_eq!(result(rule), "[1.0,[\"b\"]]");
}

#[test]
fn a_program_that_caught_the_former_error_now_receives_the_removal() {
    assert_eq!(
        result(&format!(
            "{ROBOTS}r = 1\ntry:\n    r = Robot.remove(a)\ncatch e:\n    r = e\n[r, Robot.length]"
        )),
        "[null,1.0]"
    );
}

#[test]
fn the_kept_readings_keep_their_accepted_results() {
    for (tail, expected) in [
        ("Robot.remove", "null"),
        ("f = Robot.remove\n[f, Robot.length]", "[null,2.0]"),
        ("Robot[\"remove\"](a)", "null"),
        ("a.remove()\nRobot.length", "2.0"),
        ("a.remove = x -> x + 1\na.remove(1)", "2.0"),
        ("o = {remove: x -> x * 2}\no.remove(3)", "6.0"),
        ("def remove(x) => x + 1\nremove(1)", "2.0"),
        ("remove = 4\nremove + 1", "5.0"),
        ("delete a.name\ndelete a\nRobot.length", "1.0"),
    ] {
        assert_eq!(result(&format!("{ROBOTS}{tail}")), expected, "{tail}");
    }
    assert_eq!(
        result("class Tool:\n    def remove() => 1\nt = Tool()\nt.remove()"),
        "1.0"
    );
    assert_eq!(
        result("class Empty\ne = Empty()\ndelete e\nEmpty.length"),
        "0.0"
    );
    for (tail, expected) in [
        (
            "delete a",
            "TypeError: Cannot delete object 'a' (line 5, column 1)",
        ),
        (
            "xs = [1, 2]\nxs.remove(1)",
            "TypeError: xs.remove is not a function (line 6, column 1)",
        ),
        (
            "remove a",
            "ReferenceError: remove is not defined (line 5, column 1)",
        ),
        (
            "remove of Robot",
            "SyntaxError: Unexpected keyword 'of' (line 5, column 11)",
        ),
    ] {
        assert_eq!(failure(&format!("{ROBOTS}{tail}")), expected, "{tail}");
    }
}

#[derive(Deserialize)]
struct Record {
    id: String,
    description: String,
    code: String,
    #[serde(rename = "return")]
    returns: Option<String>,
}

#[test]
fn the_synthesized_records_run_their_assertions_and_returns() {
    let records: Vec<Record> = include_str!("../dataset/nucleoid.synth.162.jsonl")
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(records.len(), 12);
    for (index, record) in records.iter().enumerate() {
        assert_eq!(record.id, format!("synth-162-{:04}", index + 1));
        assert!(!record.description.trim().is_empty(), "{}", record.id);
        assert!(calls_remove(&record.code), "{}", record.id);
        assert!(record.returns.is_some(), "{}", record.id);
        let count = common::assertions::protected_assertions(&parse(&record.code).unwrap()).len();
        let mut runtime = Runtime::new();
        common::check_with_count(
            &mut runtime,
            &common::Case {
                title: record.id.clone(),
                source: record.code.clone(),
                expected: record.returns.clone(),
            },
            count,
        )
        .unwrap_or_else(|reason| panic!("{}: {reason}", record.id));
        assert_eq!(runtime.assertions_run(), count, "{}", record.id);
    }
}

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-population-removal-{label}-{}.nuc",
            std::process::id()
        ));
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        Self(path)
    }

    fn run(&self, source: &str, options: &[&str]) -> Output {
        std::fs::write(&self.0, source).unwrap();
        Command::new(env!("CARGO_BIN_EXE_nucleoid"))
            .args(options)
            .arg(&self.0)
            .output()
            .unwrap()
    }
}

impl Drop for SourceFile {
    fn drop(&mut self) {
        std::fs::remove_file(&self.0).unwrap();
    }
}

fn repl(source: &str, options: &[&str]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .args(options)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(source.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn files_graphs_and_repl_sessions_run_remove() {
    let program = "class Robot(name, battery):\n    this.name = name\n    this.battery = battery\n$Robot.low = $Robot.battery < 20\na = Robot(\"a\", 80)\nb = Robot(\"b\", 10)\ncharging = Robot.filter(r -> r.low).map(r -> r.name)\ncount = Robot.length\nRobot.remove(b)\n[charging, count, Robot.map(r -> r.name)]";
    let spelled = program.replace(
        "Robot.remove(b)",
        "delete b.low\ndelete b.battery\ndelete b.name\ndelete b",
    );
    let file = SourceFile::new("file");
    for options in [
        &[][..],
        &["--json"][..],
        &["--graph"][..],
        &["--json", "--graph"][..],
    ] {
        let written = file.run(program, options);
        let accepted = file.run(&spelled, options);
        assert!(written.status.success(), "{options:?} {written:?}");
        assert!(written.stderr.is_empty(), "{options:?} {written:?}");
        assert_eq!(written.stdout, accepted.stdout, "{options:?}");
    }
    assert_eq!(
        String::from_utf8(file.run(program, &["--json"]).stdout).unwrap(),
        "[[],1.0,[\"a\"]]\n"
    );
    let graph = String::from_utf8(file.run(program, &["--graph"]).stdout).unwrap();
    assert!(graph.contains("  b [pending]\n"), "{graph}");
    assert!(graph.contains("  b.low [pending]\n"), "{graph}");
    let failed = file.run(&format!("{ROBOTS}Robot.remove(5)\n"), &["--json"]);
    assert!(!failed.status.success(), "{failed:?}");
    assert_eq!(
        String::from_utf8(failed.stderr).unwrap(),
        "TypeError: remove expects an instance of Robot (line 5, column 1)\n"
    );
    let session = "class Robot(name):\n    this.name = name\n\na = Robot(\"a\")\nb = Robot(\"b\")\nnames = Robot.map(r -> r.name)\nRobot.remove(a)\nnames\nRobot.length\n.exit\n";
    for options in [&["--plain"][..], &["--plain", "--json"][..]] {
        for newline in ["\n", "\r\n"] {
            let output = repl(&session.replace('\n', newline), options);
            assert!(output.status.success(), "{options:?} {output:?}");
            assert!(output.stderr.is_empty(), "{options:?} {output:?}");
            let stdout = String::from_utf8(output.stdout).unwrap();
            let expected = if options.contains(&"--json") {
                "[\"b\"]\n1.0\n"
            } else {
                "b\n1\n"
            };
            assert!(stdout.contains(expected), "{options:?} {stdout}");
        }
    }
    let failing = repl(
        "class Robot(name):\n    this.name = name\n\na = Robot(\"a\")\nRobot.remove(5)\nRobot.length\n.exit\n",
        &["--plain"],
    );
    assert!(failing.status.success(), "{failing:?}");
    assert_eq!(
        String::from_utf8(failing.stderr).unwrap(),
        "TypeError: remove expects an instance of Robot (line 1, column 1)\n"
    );
    let stdout = String::from_utf8(failing.stdout).unwrap();
    assert!(stdout.contains("1\n"), "{stdout}");
}
