mod common;

use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Keyword, Spanned, Token, tokenize};
use nucleoid::lang::ast::{Stmt, parse, parse_expression};
use nucleoid::{ErrorKind, Position, Runtime};

const SPEC_SECTION: &str = "## Implicit receivers in method bodies\n";
const REFERENCE_SECTION: &str = "### Implicit receivers in method bodies\n";
const DOT: &str = "Unexpected '.'";
const RECT: &str = "class Rect(this.w, this.h):\n";

fn statements(source: &str) -> Vec<Stmt> {
    parse(source).unwrap_or_else(|error| panic!("{source}: {error}"))
}

fn same_tree(read: &str, written: &str) {
    let tree = statements(read);
    let spelled = statements(written);
    assert_eq!(tree, spelled, "{read}");
    assert_eq!(generate_all(&tree), generate_all(&spelled), "{read}");
}

fn syntax_error(source: &str) -> (String, Position) {
    let error = Runtime::check(source).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax, "{source}: {error}");
    (error.message(), error.position().unwrap())
}

fn outcome(runtime: &mut Runtime, source: &str) -> String {
    match runtime.run(source) {
        Ok(value) => runtime.serialize_json(&value).unwrap(),
        Err(error) => format!(
            "{}: {} at {:?}",
            error.kind(),
            error.message(),
            error.position()
        ),
    }
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

fn graph_shape(runtime: &Runtime) -> String {
    let nodes: Vec<String> = runtime
        .graph
        .nodes()
        .map(|node| {
            let reads: Vec<String> = node.dependencies.iter().map(ToString::to_string).collect();
            let updates: Vec<String> = node.dependents.iter().map(ToString::to_string).collect();
            format!("{} [{}] {reads:?} {updates:?}", node.key, node.kind)
        })
        .collect();
    renumbered(&nodes.join("\n"))
}

fn behaves_alike(steps: &[(&str, &str)]) -> Vec<String> {
    let mut read = Runtime::new();
    let mut spelled = Runtime::new();
    let mut outcomes = Vec::new();
    for (receiver, written) in steps {
        let result = renumbered(&outcome(&mut read, receiver));
        assert_eq!(
            result,
            renumbered(&outcome(&mut spelled, written)),
            "{receiver}"
        );
        assert_eq!(graph_shape(&read), graph_shape(&spelled), "{receiver}");
        outcomes.push(result);
    }
    outcomes
}

fn runs(source: &str, expected: &str) {
    let count = common::assertions::protected_assertions(&statements(source)).len();
    assert!(count > 0, "{source} asserts its behavior");
    let case = common::Case {
        title: "method receiver regression".to_string(),
        source: source.to_string(),
        expected: Some(expected.to_string()),
    };
    common::check_with_assertions(&case, count)
        .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
}

fn operand_position(token: &Token) -> bool {
    !matches!(
        token,
        Token::Identifier(_)
            | Token::ClassRef(_)
            | Token::Number(_)
            | Token::VersionIndex(_)
            | Token::Str(_)
            | Token::Template(_)
            | Token::Regex(_)
            | Token::RightParen
            | Token::RightBracket
            | Token::RightBrace
            | Token::Keyword(Keyword::This)
    )
}

fn ends_operand(token: &Token) -> bool {
    !operand_position(token) || matches!(token, Token::Keyword(Keyword::Super))
}

fn assigns(token: &Token) -> bool {
    matches!(
        token,
        Token::Assign
            | Token::ColonAssign
            | Token::PlusAssign
            | Token::MinusAssign
            | Token::StarAssign
            | Token::StarStarAssign
            | Token::SlashAssign
            | Token::PercentAssign
            | Token::AndAndAssign
            | Token::OrOrAssign
    )
}

fn starts_statement(token: Option<&Token>) -> bool {
    token.is_none_or(|token| {
        matches!(
            token,
            Token::Newline | Token::Indent | Token::Dedent | Token::Semicolon
        )
    })
}

fn property_target_end(tokens: &[Token], start: usize) -> Option<usize> {
    let mut index = start;
    if !matches!(
        tokens.get(index),
        Some(Token::Identifier(_) | Token::ClassRef(_) | Token::Keyword(Keyword::This))
    ) {
        return None;
    }
    index += 1;
    let mut links = 0;
    while tokens.get(index) == Some(&Token::Dot)
        && matches!(
            tokens.get(index + 1),
            Some(Token::Identifier(_) | Token::Keyword(_))
        )
    {
        index += 2;
        links += 1;
    }
    (links > 0 && tokens.get(index).is_some_and(assigns)).then_some(index)
}

fn class_body_start(tokens: &[Token], class: usize) -> Option<(usize, bool)> {
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(class + 1) {
        match token {
            Token::LeftParen | Token::LeftBracket | Token::LeftBrace => depth += 1,
            Token::RightParen | Token::RightBracket | Token::RightBrace => {
                depth = depth.saturating_sub(1);
            }
            Token::Newline | Token::Indent | Token::Eof if depth == 0 => return None,
            Token::Colon if depth == 0 => {
                let parent = matches!(tokens.get(index + 1), Some(Token::Identifier(_)))
                    && matches!(
                        tokens.get(index + 2),
                        Some(Token::Newline | Token::Indent | Token::Eof)
                    );
                let after = if parent { index + 2 } else { index + 1 };
                let indented = matches!(tokens.get(after), Some(Token::Newline | Token::Indent));
                return Some((after, indented));
            }
            _ => {}
        }
    }
    None
}

fn method_reads(source: &str) -> (Vec<Position>, usize) {
    let spanned: Vec<Spanned> = tokenize(source).unwrap();
    let tokens: Vec<Token> = spanned.iter().map(|item| item.token.clone()).collect();
    let mut level = 0usize;
    let mut bodies: Vec<usize> = Vec::new();
    let mut pending = false;
    let mut line_body: Option<usize> = None;
    let mut brackets: Vec<bool> = Vec::new();
    let mut value_depth: Option<usize> = None;
    let mut reads = Vec::new();
    let mut property_reads = 0;
    for (index, token) in tokens.iter().enumerate() {
        let previous = index.checked_sub(1).and_then(|before| tokens.get(before));
        if value_depth.is_some_and(|depth| depth == brackets.len())
            && matches!(
                token,
                Token::Newline | Token::Semicolon | Token::Indent | Token::Dedent
            )
        {
            value_depth = None;
        }
        if starts_statement(previous) && value_depth.is_none() {
            if let Some(end) = property_target_end(&tokens, index) {
                debug_assert!(end > index);
                value_depth = Some(brackets.len());
            }
        }
        match token {
            Token::Indent => {
                level += 1;
                if pending {
                    bodies.push(level);
                    pending = false;
                }
            }
            Token::Dedent => {
                level = level.saturating_sub(1);
                while bodies.last().is_some_and(|body| *body > level) {
                    bodies.pop();
                }
            }
            Token::Newline if line_body.is_some_and(|depth| depth == brackets.len()) => {
                line_body = None;
            }
            Token::Keyword(Keyword::Class) => match class_body_start(&tokens, index) {
                Some((_, true)) => pending = true,
                Some((_, false)) => line_body = Some(brackets.len()),
                None => {}
            },
            Token::LeftParen => brackets.push(previous.is_some_and(ends_operand)),
            Token::LeftBracket | Token::LeftBrace => brackets.push(false),
            Token::RightParen | Token::RightBracket | Token::RightBrace => {
                brackets.pop();
            }
            Token::Dot => {
                let named = matches!(
                    tokens.get(index + 1),
                    Some(Token::Identifier(_) | Token::Keyword(_))
                );
                let in_class = !bodies.is_empty() || line_body.is_some();
                let head = matches!(previous, Some(Token::LeftParen | Token::Comma))
                    && brackets.last() == Some(&true);
                let read = named
                    && in_class
                    && previous.is_none_or(operand_position)
                    && !starts_statement(previous)
                    && previous != Some(&Token::Dot)
                    && !head;
                if read && value_depth.is_some() {
                    property_reads += 1;
                } else if read {
                    reads.push(spanned[index].position);
                }
            }
            _ => {}
        }
    }
    (reads, property_reads)
}

fn documents() -> [Vec<common::Case>; 4] {
    [
        common::cases(include_str!("../nucleoid.spec.md")),
        common::cases(include_str!("reference.md")),
        common::snippets(include_str!("../README.md"), "README.md"),
        common::snippets(include_str!("../docs/examples.md"), "docs/examples.md"),
    ]
}

#[test]
fn no_document_case_reads_the_instance_of_a_method() {
    let mut cases = 0;
    let mut property_reads = 0;
    for case in documents().iter().flatten() {
        cases += 1;
        let (reads, kept) = method_reads(&case.source);
        assert_eq!(reads, Vec::new(), "{}", case.title);
        property_reads += kept;
    }
    assert_eq!(cases, 1355);
    assert_eq!(property_reads, 3);
    for (source, reads) in [
        (
            "class R(this.w):\n    def f() => .w",
            vec![Position::new(2, 16)],
        ),
        (
            "class R(this.w):\n    def f():\n        return [.w, (.w)]",
            vec![Position::new(3, 17), Position::new(3, 22)],
        ),
        ("class R(w): if .w: this.k = 1", vec![Position::new(1, 16)]),
        (
            "class S: R\n    def f() => `${.w}` + -.w",
            vec![Position::new(2, 27)],
        ),
        ("class R(w):\n    this.w = w\n    this.k = .w * 2", vec![]),
        (
            "class R:\n    def f() => g(.w)\n    def h() => .w.push(1)",
            vec![Position::new(3, 16)],
        ),
        ("def f() => .w\no.p = .q", vec![]),
    ] {
        assert_eq!(method_reads(source).0, reads, "{source}");
    }
}

fn section(document: &str, heading: &str, fence: &str) -> Vec<common::Case> {
    let document = document.replace("\r\n", "\n");
    let (_, rest) = document
        .split_once(heading)
        .unwrap_or_else(|| panic!("the document holds {heading:?}"));
    let end = rest.find("\n## ").unwrap_or(rest.len());
    let body = rest.get(..end).unwrap();
    let opening = format!("\n{fence}\n");
    let mut cases = Vec::new();
    let mut rest = body;
    while let Some((_, after)) = rest.split_once(opening.as_str()) {
        let (block, tail) = after
            .split_once(&format!("\n{fence}"))
            .expect("every program block is closed");
        let expected = block.lines().find_map(|line| {
            line.trim()
                .strip_prefix("# return:")
                .map(|value| value.trim().to_string())
        });
        let source = block
            .lines()
            .filter(|line| !line.trim().starts_with("# return:"))
            .collect::<Vec<_>>()
            .join("\n");
        cases.push(common::Case {
            title: format!("{heading} program {}", cases.len() + 1),
            source,
            expected,
        });
        rest = tail;
    }
    cases
}

#[test]
fn the_specification_and_the_reference_run_their_programs() {
    let specification = section(include_str!("../nucleoid.spec.md"), SPEC_SECTION, "~~~");
    let reference = section(include_str!("../docs/README.md"), REFERENCE_SECTION, "```");
    assert_eq!(specification.len(), 4);
    assert_eq!(reference.len(), 1);
    let mut assertions = 0;
    for case in specification.iter().chain(&reference) {
        assert!(case.expected.is_some(), "{} states its result", case.title);
        let count = common::assertions::protected_assertions(&statements(&case.source)).len();
        assert!(count > 0, "{} asserts its behavior", case.title);
        common::check_with_assertions(case, count)
            .unwrap_or_else(|reason| panic!("{}: {reason}\n{}", case.title, case.source));
        assert!(
            !method_reads(&case.source).0.is_empty(),
            "{} reads the instance",
            case.title
        );
        assertions += count;
    }
    assert_eq!(assertions, 9);
    let reference_text = include_str!("../docs/README.md");
    assert!(reference_text.contains("[NUC 113](nuc-0113.md)"));
    assert!(reference_text.contains("../nucleoid.spec.md#implicit-receivers-in-method-bodies"));
    assert!(include_str!("../nucleoid.spec.md").contains("[NUC 113](docs/nuc-0113.md)"));
}

#[test]
fn every_body_kind_of_a_class_reads_its_instance() {
    for (read, written) in [
        (
            "class R(this.w):\n    def f() => .w",
            "class R(this.w):\n    def f() => this.w",
        ),
        (
            "class R(this.w):\n    def f():\n        return .w",
            "class R(this.w):\n    def f():\n        return this.w",
        ),
        (
            "class R(this.w):\n    def f() =>\n        .w * 2",
            "class R(this.w):\n    def f() =>\n        this.w * 2",
        ),
        (
            "class R(this.w):\n    def f(k) => .w * k",
            "class R(this.w):\n    def f(k) => this.w * k",
        ),
        (
            "class R(this.w):\n    def f k => .w * k",
            "class R(this.w):\n    def f k => this.w * k",
        ),
        (
            "class R(this.w):\n    def f a b => .w + a + b",
            "class R(this.w):\n    def f a b => this.w + a + b",
        ),
        (
            "class R(this.w):\n    def f a, b => .w + a + b",
            "class R(this.w):\n    def f a, b => this.w + a + b",
        ),
        (
            "class R(this.w):\n    def f k:\n        return .w * k",
            "class R(this.w):\n    def f k:\n        return this.w * k",
        ),
        (
            "class R(this.w):\n    def f(k: int) => .w * k",
            "class R(this.w):\n    def f(k: int) => this.w * k",
        ),
        (
            "class R(this.w):\n    def f(k = 2) => .w * k",
            "class R(this.w):\n    def f(k = 2) => this.w * k",
        ),
        (
            "class R(this.w):\n    def f(...ks) => .w + ks.length",
            "class R(this.w):\n    def f(...ks) => this.w + ks.length",
        ),
        (
            "class R(this.w):\n    def f(self, k) => .w * k",
            "class R(this.w):\n    def f(self, k) => self.w * k",
        ),
        (
            "class R(this.w):\n    def f(this) => .w",
            "class R(this.w):\n    def f(this) => this.w",
        ),
        (
            "class R(this.w):\n    function f() { return .w }",
            "class R(this.w):\n    function f() { return this.w }",
        ),
        (
            "class R:\n    def init(w):\n        this.w = w\n        if .w > 1:\n            this.big = true",
            "class R:\n    def init(w):\n        this.w = w\n        if this.w > 1:\n            this.big = true",
        ),
        (
            "class R:\n    def __init__(self, w):\n        self.w = w\n        self.k = [.w]",
            "class R:\n    def __init__(self, w):\n        self.w = w\n        self.k = [self.w]",
        ),
        (
            "class R(w):\n    this.w = w\n    if .w > 1:\n        this.big = true",
            "class R(w):\n    this.w = w\n    if this.w > 1:\n        this.big = true",
        ),
        (
            "class R(this.w): if .w > 1: this.big = true",
            "class R(this.w): if this.w > 1: this.big = true",
        ),
        (
            "class S(this.s) extends R:\n    def f() => .s + .w",
            "class S(this.s) extends R:\n    def f() => this.s + this.w",
        ),
        (
            "class S: R\n    def f() => .w",
            "class S: R\n    def f() => this.w",
        ),
        (
            "def outer():\n    class Inner(this.v):\n        def get() => .v\n    return Inner(2).get()",
            "def outer():\n    class Inner(this.v):\n        def get() => this.v\n    return Inner(2).get()",
        ),
    ] {
        same_tree(read, written);
    }
    runs(
        "class Rect(this.w, this.h):\n    def area() => .w * .h\n    def fits(self, box):\n        return .w <= box.w and .h <= box.h\nclass Square(this.side) extends Rect:\n    def area() => .side * .side\nr = Rect(2, 3)\nassert([r.area(), r.fits(Rect(2, 4)), Square(3).area()], [6, true, 9])\nr.fits(Rect(1, 1))",
        "false",
    );
}

#[test]
fn reads_work_in_every_operand_position_with_every_suffix() {
    for (read, written) in [
        (".w + .h * -.w", "this.w + this.h * -this.w"),
        ("not .c or typeof .w", "not this.c or typeof this.w"),
        (".w ?? .h", "this.w ?? this.h"),
        (".c ? .w : .h", "this.c ? this.w : this.h"),
        (".w if .c else .h", "this.w if this.c else this.h"),
        ("[.w, .h]", "[this.w, this.h]"),
        ("{w: .w, h: .h}", "{w: this.w, h: this.h}"),
        ("`${.w}x${.h}`", "`${this.w}x${this.h}`"),
        (
            ".xs[.i] + .xs[.i:].length",
            "this.xs[this.i] + this.xs[this.i:].length",
        ),
        (".xs[:.i]", "this.xs[:this.i]"),
        (".a.b.c", "this.a.b.c"),
        (".area() * 2", "this.area() * 2"),
        (".items[0].name", "this.items[0].name"),
        (".w@first + .w@last", "this.w@first + this.w@last"),
        ("new .Kind(1)", "new this.Kind(1)"),
        (".self.w + .None", "this.self.w + this.None"),
        ("f(1 + .w, -.h)", "f(1 + this.w, -this.h)"),
        ("f((.w), [.h])", "f((this.w), [this.h])"),
        (
            "(.w is 3) and (1 is not .h)",
            "(this.w is 3) and (1 is not this.h)",
        ),
        ("all x in .xs: x > .w", "all x in this.xs: x > this.w"),
        (".xs |> why", "this.xs |> why"),
        (".w in .xs", "this.w in this.xs"),
        ("sum of .xs", "sum of this.xs"),
    ] {
        same_tree(
            &format!("{RECT}    def f() => {read}"),
            &format!("{RECT}    def f() => {written}"),
        );
        same_tree(
            &format!("{RECT}    def f() => {read}\n    def g() => [x for x in [1]]"),
            &format!("{RECT}    def f() => {written}\n    def g() => [x for x in [1]]"),
        );
    }
    for (read, written) in [
        (
            "if .w > .h:\n            return 1\n        return 2",
            "if this.w > this.h:\n            return 1\n        return 2",
        ),
        (
            "for x in .xs:\n            this.t = x\n        return .t",
            "for x in this.xs:\n            this.t = x\n        return this.t",
        ),
        (
            "while 0 < .w:\n            this.w = .w - 1\n        return .w",
            "while 0 < this.w:\n            this.w = this.w - 1\n        return this.w",
        ),
        (
            "try:\n            throw .w\n        catch e:\n            return e + .h",
            "try:\n            throw this.w\n        catch e:\n            return e + this.h",
        ),
        (
            "delete .cache\n        return 1",
            "delete this.cache\n        return 1",
        ),
        (
            "s = suppose this.w = 5: (.w * .h)\n        return s",
            "s = suppose this.w = 5: (this.w * this.h)\n        return s",
        ),
        (
            "if .w > 1: return .h\n        return 0",
            "if this.w > 1: return this.h\n        return 0",
        ),
        (
            "ys = [.w, .h]\n        for y in ys:\n            this.t = y\n        return ys",
            "ys = [this.w, this.h]\n        for y in ys:\n            this.t = y\n        return ys",
        ),
    ] {
        same_tree(
            &format!("{RECT}    def f():\n        {read}"),
            &format!("{RECT}    def f():\n        {written}"),
        );
    }
}

#[test]
fn property_values_keep_the_innermost_receiver_and_names_read_the_instance() {
    for (read, written) in [
        ("this.w = .h", "this.w = this.h"),
        ("other.p = .q + .r", "other.p = other.q + other.r"),
        ("other.a.p = .q", "other.a.p = other.a.q"),
        ("$Rect.p = .q", "$Rect.p = $Rect.q"),
        ("x = .w * 2", "x = this.w * 2"),
        ("x += .w", "x += this.w"),
        ("x := .w", "x := this.w"),
        ("x = (o.p = .a) + .b", "x = (o.p = o.a) + this.b"),
        ("x = (y = .a)", "x = (y = this.a)"),
        ("a, b = .pair", "a, b = this.pair"),
        ("{a, b} = .pair", "{a, b} = this.pair"),
        ("x = P(n = .w)", "x = P(n = this.w)"),
        (
            "x = [1].map(v -> (o.p = .q))",
            "x = [1].map(v -> (o.p = o.q))",
        ),
        (
            "this.f = function() { return .w }",
            "this.f = function() { return this.w }",
        ),
        (
            "this.xs = [1].map(i -> i * .w)",
            "this.xs = [1].map(i -> i * this.w)",
        ),
    ] {
        same_tree(
            &format!("{RECT}    def f():\n        {read}\n        return 1"),
            &format!("{RECT}    def f():\n        {written}\n        return 1"),
        );
    }
    runs(
        "class Team(this.bonus):\n    def promote(person):\n        raise = .bonus\n        person.pay = .pay + raise\n        return person.pay\nbob = {pay: 90}\nt = Team(5)\nassert(t.promote(bob), 95)\nbob.pay",
        "95",
    );
}

#[test]
fn nested_functions_keep_their_accepted_errors() {
    for (body, line, column) in [
        ("def f() => [1, 2].map(i -> i * .w)", 2, 37),
        ("def f() => [1, 2].map((i) => i * .w)", 2, 39),
        ("def f() => [1].map(i => { return i * .w })", 2, 43),
        ("def f() => [1].map(function(i) { return i * .w })", 2, 50),
        ("def f() => [1].map(lambda i: i * .w)", 2, 39),
        ("def f():\n        def g() => .w\n        return g()", 3, 21),
        (
            "def f():\n        function g() { return .w }\n        return g()",
            3,
            32,
        ),
        ("def f() => [i * .w for i in [1, 2]]", 2, 22),
        ("def f() => [i for i in [1, 2] if i > .w]", 2, 43),
        ("def f() => [1, 5].filter(> .w)", 2, 33),
        ("def f() => `${[1].map(i -> .w)}`", 2, 33),
        ("def f() => [y -> .w for y in [1]]", 2, 23),
        (
            "def f():\n        if true:\n            def g() => .w\n        return 1",
            4,
            25,
        ),
        ("def f() => [1].map(i -> [i, .w])", 2, 34),
        ("def f():\n        x = [1].map(i -> (y = .w))", 3, 32),
        ("def f() => [(x * .w) for x in [1]]", 2, 23),
        ("def f() => [[.w][0] for x in [1]]", 2, 19),
        ("def f() => [{a: .w} for x in [1]]", 2, 22),
        ("def f() => [1, [.w for x in .xs]]", 2, 22),
    ] {
        let source = format!("class R(this.w, this.xs):\n    {body}\n");
        assert_eq!(
            syntax_error(&source),
            (DOT.to_string(), Position::new(line, column)),
            "{source}"
        );
    }
    for (source, line, column) in [
        ("class C(this.x): def f() => .x", 1, 30),
        (
            "class R(w):\n    this.w = w\n    ys = [1].map(i -> .w)",
            3,
            24,
        ),
        (
            "class R:\n    def init(x):\n        ys = [1].map(i -> .x)",
            3,
            28,
        ),
        (
            "class R(w):\n    this.w = w\n    this.f = function() { x = .w }",
            3,
            32,
        ),
    ] {
        assert_eq!(
            syntax_error(source),
            (DOT.to_string(), Position::new(line, column)),
            "{source}"
        );
    }
    for (read, written) in [
        (
            "def f() => [x * 2 for x in .xs]",
            "def f() => [x * 2 for x in this.xs]",
        ),
        (
            "def f() => [x for x in [y for y in .xs]]",
            "def f() => [x for x in [y for y in this.xs]]",
        ),
        (
            "def f() => [[y for y in .xs], .w]",
            "def f() => [[y for y in this.xs], this.w]",
        ),
        (
            "def f() => [.w, x for x in .xs]",
            "def f() => [.w, x for x in .xs]",
        ),
    ] {
        let source = format!("class R(this.w, this.xs):\n    {read}");
        if read == written {
            assert_eq!(
                syntax_error(&source),
                (DOT.to_string(), Position::new(2, 18)),
                "{source}"
            );
        } else {
            same_tree(
                &source,
                &format!("class R(this.w, this.xs):\n    {written}"),
            );
        }
    }
    assert_eq!(
        syntax_error("class R(this.xs, this.k):\n    def f() => [x for x in .xs if x > .k]"),
        (DOT.to_string(), Position::new(2, 40))
    );
    behaves_alike(&[
        (
            "class R(this.w):\n    def twice() => [1, 2].map(i -> i * this.w)\nr = R(3)",
            "class R(this.w):\n    def twice() => [1, 2].map(i -> i * this.w)\nr = R(3)",
        ),
        ("r.twice()", "r.twice()"),
    ]);
}

#[test]
fn kept_readings_in_class_bodies_keep_their_accepted_diagnostics() {
    for (body, message, line, column) in [
        ("def f():\n        .w = 9", DOT, 3, 10),
        ("def f():\n        .xs.push(1)", DOT, 3, 10),
        ("def f() => String(.w + 1)", DOT, 2, 24),
        ("def f() => .xs.map(.w + 1)", DOT, 2, 25),
        ("def f() => new R(.w + 1, 2)", DOT, 2, 23),
        ("def f() => super(.w + 1)", DOT, 2, 23),
        ("def f():\n        assert: .w, 3", DOT, 3, 18),
        (
            "def f():\n        while .w > 0:\n            this.w = 0",
            "Unexpected ':'",
            3,
            21,
        ),
        (
            "def f():\n        x is 2 * .w\n        return x",
            DOT,
            3,
            19,
        ),
        ("def f():\n        this.p = (x = .w)", DOT, 3, 24),
        (
            "def f():\n        ys = [{}]\n        ys[0].p = .w",
            DOT,
            4,
            20,
        ),
        ("def f():\n        this.p@first = .w", DOT, 3, 25),
        ("def f():\n        ys = [1]\n        ys[0] = .w", DOT, 4, 18),
        ("def f() => (.w = 1)", DOT, 2, 18),
        ("def f() => (1 is .w)", DOT, 2, 23),
        ("def f():\n        $R = .w", DOT, 3, 15),
        (
            "def f():\n        suppose .w = 5: this.w",
            "Unexpected ':'",
            3,
            25,
        ),
        ("def f() => .", DOT, 2, 16),
        ("def f() => ..w", DOT, 2, 17),
        ("def f():\n        return this.w\n    .w", DOT, 4, 6),
        (
            "def f():\n        s = suppose this.w = 5: .w * 2",
            DOT,
            3,
            34,
        ),
        ("def f():\n        if this.w > 1: .w", DOT, 3, 25),
    ] {
        let source = format!("class R(this.w, this.xs):\n    {body}\n");
        assert_eq!(
            syntax_error(&source),
            (message.to_string(), Position::new(line, column)),
            "{source}"
        );
    }
    for (source, message, line, column) in [
        ("def f() => .a", DOT, 1, 13),
        ("o.f = function() { x = .a }", DOT, 1, 25),
        ("x = .a", DOT, 1, 6),
        ("class R:\n    pass\nx = .a", DOT, 3, 6),
    ] {
        assert_eq!(
            syntax_error(source),
            (message.to_string(), Position::new(line, column)),
            "{source}"
        );
    }
    for (source, expected) in [
        (
            "class R(this.w):\n    def f() => [this].map(.w)\nR(2).f()",
            "[2.0]",
        ),
        (
            "class R(this.w):\n    def f(x) => x .a\nR(2).f({a: 4})",
            "4.0",
        ),
        ("class R(this.w):\n    def f() => .5\nR(2).f()", "0.5"),
        (
            "class R(this.w):\n    def f() => [this.w\n.a]\nR(2).f()",
            "[null]",
        ),
        (
            "class R(this.w):\n    def f() => {...{a: 2}}\nR(2).f()",
            "{\"a\":2.0}",
        ),
        (
            "class Inner(this.v)\nclass R(this.w):\n    def f() => Inner(.w).v({w: 7})\nR(2).f()",
            "7.0",
        ),
        (
            "o = {w: 2}\no.f = function() { return .w * 10 }\no.f()",
            "20.0",
        ),
        ("class R(this.w)\n$R.a = .w * 2\nR(3).a", "6.0"),
        (
            "class R(this.w):\n    def f():\n        this.w = .w + 1\n        return this.w\nR(1).f()",
            "2.0",
        ),
        (
            "class R(this.w):\n    def f():\n        this.b is .a\n        return 1\nR(1).f()",
            "ReferenceError: is is not defined at Some(Position { line: 5, column: 1 })",
        ),
    ] {
        assert_eq!(outcome(&mut Runtime::new(), source), expected, "{source}");
    }
    assert_eq!(
        statements("class R(this.w):\n    def f() => 1 is not .w"),
        statements("class R(this.w):\n    def f() => 1\n    is\n    not this.w")
    );
}

#[test]
fn juxtaposed_headers_in_methods_separate_as_their_written_out_spellings() {
    for header in ["while", "s = suppose"] {
        for (read, written) in [
            ("1 + .n is a: b", "1 + this.n is a: b"),
            ("0 + .k * .n is a: b", "0 + this.k * this.n is a: b"),
            ("1 + (y = .n) is a: b", "1 + (y = this.n) is a: b"),
            (".k + .n is a: b", ".k + this.n is a: b"),
            (
                "0 < .n:\n            this.n = 0",
                "0 < this.n:\n            this.n = 0",
            ),
        ] {
            let read = format!(
                "class R(this.n, this.k):\n    def f():\n        {header} {read}\n        return 1\nr = R(3, 1)\nr.f()"
            );
            let written = format!(
                "class R(this.n, this.k):\n    def f():\n        {header} {written}\n        return 1\nr = R(3, 1)\nr.f()"
            );
            same_tree(&read, &written);
            assert_eq!(
                outcome(&mut Runtime::new(), &read),
                outcome(&mut Runtime::new(), &written),
                "{read}"
            );
        }
    }
    same_tree(
        "class R(this.n, this.k):\n    def f():\n        while .k + .n is a: b\n        return 1",
        "class R(this.n, this.k):\n    def f():\n        while.k + this.n\n        is\n        a: b\n        return 1",
    );
    assert_eq!(
        outcome(
            &mut Runtime::new(),
            "class R(this.n):\n    def f():\n        while 1 + .n is a: b\n        return 1\nR(3).f()"
        ),
        "ReferenceError: while is not defined at Some(Position { line: 5, column: 1 })"
    );
    assert_eq!(
        outcome(
            &mut Runtime::new(),
            "o = {x: 1}\no.f = function() { while 1 + .x is a: b\nreturn 1 }\no.f()"
        ),
        "1.0"
    );
    for (header, expected) in [
        ("while", "1.0"),
        (
            "s = suppose",
            "ReferenceError: b is not defined at Some(Position { line: 6, column: 1 })",
        ),
    ] {
        let source = format!(
            "o = {{y: 1}}\nclass R(this.n):\n    def f():\n        {header} 1 + (o.f = function() {{ class C: if .y: this.z = 1 }}) is a: b\n        return 1\nR(3).f()"
        );
        assert_eq!(outcome(&mut Runtime::new(), &source), expected, "{source}");
    }
}

#[test]
fn a_class_inside_a_property_value_keeps_the_receiver_of_that_value() {
    assert_eq!(
        outcome(
            &mut Runtime::new(),
            "o = {y: 4}\no.f = function() { class C: if .y: this.z = 1\nreturn C().z }\no.f()"
        ),
        "1.0"
    );
    same_tree(
        "o.f = function() { class C: if .y: this.z = .y }",
        "o.f = function() { class C: if o.y: this.z = this.y }",
    );
    same_tree(
        "class R(this.w):\n    def f() => [1].map(i -> function() { class C(this.v): if .v: this.z = 1 })",
        "class R(this.w):\n    def f() => [1].map(i -> function() { class C(this.v): if this.v: this.z = 1 })",
    );
}

#[test]
fn reads_render_explain_depend_and_update_as_their_written_out_spellings() {
    let outcomes = behaves_alike(&[
        (
            "class Rect(this.w, this.h):\n    def area() => .w * .h\n    def label() => `${.w}x${.h}`",
            "class Rect(this.w, this.h):\n    def area() => this.w * this.h\n    def label() => `${this.w}x${this.h}`",
        ),
        (
            "r = Rect(2, 3)\na = r.area()",
            "r = Rect(2, 3)\na = r.area()",
        ),
        ("r.w = 5\n[a, r.label()]", "r.w = 5\n[a, r.label()]"),
        ("(why a)[0].rule", "(why a)[0].rule"),
        (
            "(affects r.w).map(step -> step.node)",
            "(affects r.w).map(step -> step.node)",
        ),
        (
            "$Rect.size = .area()\nr.size",
            "$Rect.size = $Rect.area()\nr.size",
        ),
        (
            "s = suppose r.h = 10: a\n[s, a]",
            "s = suppose r.h = 10: a\n[s, a]",
        ),
        (
            "try:\n    r.w = 4\n    throw \"UNDO\"\ncatch error:\n    error\n[r.w, a]",
            "try:\n    r.w = 4\n    throw \"UNDO\"\ncatch error:\n    error\n[r.w, a]",
        ),
    ]);
    assert_eq!(outcomes[2], "[15.0,\"5x3\"]");
    assert_eq!(outcomes[3], "\"a = r.area()\"");
    assert_eq!(outcomes[6], "[50.0,15.0]");
    assert_eq!(outcomes[7], "[5.0,15.0]");
    behaves_alike(&[
        (
            "class Tank(this.level, this.size):\n    def status():\n        if .level <= 0:\n            return \"empty\"\n        return `${.level} of ${.size}`",
            "class Tank(this.level, this.size):\n    def status():\n        if this.level <= 0:\n            return \"empty\"\n        return `${this.level} of ${this.size}`",
        ),
        (
            "t = Tank(12, 20)\nassert(t.status(), \"12 of 20\")",
            "t = Tank(12, 20)\nassert(t.status(), \"12 of 20\")",
        ),
        ("t.level = 0", "t.level = 0"),
        (
            "class Basket(this.rate):\n    def price():\n        rate = .rate\n        return rate\nb = Basket(10)\np = b.price()",
            "class Basket(this.rate):\n    def price():\n        rate = this.rate\n        return rate\nb = Basket(10)\np = b.price()",
        ),
        ("b.rate = 2\np", "b.rate = 2\np"),
        (
            "class Shape(this.w)\nclass Box(this.d) extends Shape:\n    def volume() => .w * .d\nx = Box(4)\nx.w = 2\nv = x.volume()",
            "class Shape(this.w)\nclass Box(this.d) extends Shape:\n    def volume() => this.w * this.d\nx = Box(4)\nx.w = 2\nv = x.volume()",
        ),
        ("x.d = 5\nv", "x.d = 5\nv"),
    ]);
    assert_eq!(
        generate_all(&statements("class R(this.w):\n    def f() => .w * 2")),
        generate_all(&statements("class R(this.w):\n    def f() => this.w * 2"))
    );
}

#[test]
fn errors_through_reads_are_those_of_their_written_out_spellings() {
    for (read, written) in [
        ("def f() => .w.x.y", "def f() => this.w.x.y"),
        ("def f() => .g()", "def f() => this.g()"),
        ("def f() => .value", "def f() => this.value"),
        (
            "def f():\n        this.value = .w\n        return 1",
            "def f():\n        this.value = this.w\n        return 1",
        ),
    ] {
        let mut runtime = Runtime::new();
        let mut spelled = Runtime::new();
        let tail = "\nR(1).f()";
        assert_eq!(
            renumbered(&outcome(
                &mut runtime,
                &format!("class R(this.w):\n    {read}{tail}")
            )),
            renumbered(&outcome(
                &mut spelled,
                &format!("class R(this.w):\n    {written}{tail}")
            )),
            "{read}"
        );
    }
    for (source, written, message, line, column) in [
        (
            "class R(this.w):\n    def f() => .w +\n",
            "class R(this.w):\n    def f() => this.w +\n",
            "Unexpected end of line",
            3,
            1,
        ),
        (
            "class R(this.w):\n    def f() => .w +",
            "class R(this.w):\n    def f() => this.w +",
            "Unexpected the end of a block",
            2,
            20,
        ),
        (
            "class R(this.w):\n    def f() => .w.\n",
            "class R(this.w):\n    def f() => this.w.\n",
            "Expected a property but found end of line",
            3,
            1,
        ),
        (
            "class R(this.w):\n    def f() => `${.w +}`",
            "class R(this.w):\n    def f() => `${this.w +}`",
            "Unexpected end of input",
            2,
            23,
        ),
        (
            "class R(this.xs):\n    def f() => .xs.map(.w + 1)",
            "class R(this.xs):\n    def f() => this.xs.map(.w + 1)",
            DOT,
            2,
            25,
        ),
        (
            "class R(this.xs):\n    def f() => [1, .w for x in .xs]",
            "class R(this.xs):\n    def f() => [1, this.w for x in this.xs]",
            "Expected ']' but found keyword 'for'",
            2,
            23,
        ),
        (
            "class R(this.w):\n    def f() => [.w",
            "class R(this.w):\n    def f() => [this.w",
            "Expected ']' but found the end of a block",
            2,
            19,
        ),
    ] {
        let (read, position) = syntax_error(source);
        assert_eq!(
            (read.as_str(), position),
            (message, Position::new(line, column)),
            "{source}"
        );
        assert_eq!(syntax_error(written).0, read, "{written}");
    }
}

#[test]
fn a_read_counts_toward_the_chain_bound_as_this_does() {
    let suffixes = ".b".repeat(126);
    same_tree(
        &format!("{RECT}    def f() => .a{suffixes}"),
        &format!("{RECT}    def f() => this.a{suffixes}"),
    );
    same_tree(
        &format!("{RECT}    def f() => new .a{suffixes}"),
        &format!("{RECT}    def f() => new this.a{suffixes}"),
    );
    for source in [
        format!("{RECT}    def f() => .a{suffixes}.b"),
        format!("{RECT}    def f() => this.a{suffixes}.b"),
        format!("{RECT}    def f() => new .a{suffixes}.b"),
    ] {
        assert_eq!(
            syntax_error(&source).0,
            "Expression has too many parts",
            "{source}"
        );
    }
}

#[test]
fn accepted_spellings_keep_their_trees() {
    for (source, spelled) in [
        (
            "class R(this.w):\n    def f(x) => x .a",
            "class R(this.w):\n    def f(x) => x.a",
        ),
        (
            "class R(this.w):\n    this.k = .w * 2",
            "class R(this.w):\n    this.k = this.w * 2",
        ),
        (
            "class R(this.w):\n    def f():\n        this.b is .a",
            "class R(this.w):\n    def f():\n        this.b\n        is.a",
        ),
        (
            "class R(this.w):\n    def f() => [this].map(.w)",
            "class R(this.w):\n    def f() => [this].map(.w)",
        ),
    ] {
        same_tree(source, spelled);
    }
    assert!(parse_expression(".w").is_err());
    assert!(parse_expression("x = .w").is_err());
    assert_eq!(
        parse_expression("function() { class C(this.w): if .w: this.k = 1 }").unwrap(),
        parse_expression("function() { class C(this.w): if this.w: this.k = 1 }").unwrap()
    );
}

fn deepest(build: impl Fn(usize) -> String) -> usize {
    let started = Instant::now();
    for depth in 1..80 {
        let checked = Runtime::check(&build(depth));
        assert!(started.elapsed() < Duration::from_secs(20), "depth {depth}");
        if let Err(error) = checked {
            assert!(
                matches!(
                    error.message().as_str(),
                    "Expressions are nested too deeply" | "Statements are nested too deeply"
                ),
                "depth {depth}: {error}"
            );
            return depth;
        }
    }
    panic!("the nesting limit is never reached")
}

#[test]
fn nested_reads_reach_the_nesting_limit_at_the_depth_of_their_written_out_spellings() {
    for (opener, closer) in [("(", ")"), ("[", "]"), ("{k: ", "}"), ("f(1 + ", ")")] {
        let read = deepest(|depth| {
            format!(
                "{RECT}    def f() => {}.w{}",
                opener.repeat(depth),
                closer.repeat(depth)
            )
        });
        let written = deepest(|depth| {
            format!(
                "{RECT}    def f() => {}this.w{}",
                opener.repeat(depth),
                closer.repeat(depth)
            )
        });
        assert!(read > 15, "{opener} stops at {read}");
        assert_eq!(read, written, "{opener}");
    }
    let read = deepest(|depth| {
        let mut source = String::new();
        for level in 0..depth {
            let indent = "    ".repeat(2 * level);
            source.push_str(&format!(
                "{indent}class C{level}(this.w):\n{indent}    def m():\n"
            ));
        }
        source.push_str(&"    ".repeat(2 * depth));
        source.push_str("return .w");
        source
    });
    assert!(read > 15, "classes stop at {read}");
}

struct SourceFile(PathBuf);

impl SourceFile {
    fn new(label: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "nucleoid-method-receivers-{label}-{}.nuc",
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
fn files_graphs_and_repl_sessions_run_reads_as_their_written_out_spellings() {
    let program = "class Rect(this.w, this.h):\n    def area() => .w * .h\n    def label():\n        return `${.w}x${.h}`\nr = Rect(2, 3)\na = r.area()\nr.w = 5\n[a, r.label()]";
    let written = program
        .replace("=> .w * .h", "=> this.w * this.h")
        .replace("`${.w}x${.h}`", "`${this.w}x${this.h}`");
    let file = SourceFile::new("file");
    for options in [
        &[][..],
        &["--json"][..],
        &["--graph"][..],
        &["--json", "--graph"][..],
    ] {
        let read = file.run(program, options);
        assert!(read.status.success(), "{read:?}");
        assert!(read.stderr.is_empty(), "{read:?}");
        let spelled = file.run(&written, options);
        assert_eq!(read.stdout, spelled.stdout, "{options:?}");
    }
    assert_eq!(
        String::from_utf8(file.run(program, &["--json"]).stdout).unwrap(),
        "[15.0,\"5x3\"]\n"
    );
    let failed = file.run("class R(this.w):\n    def f() => [1].map(i -> .w)", &[]);
    assert!(!failed.status.success(), "{failed:?}");
    assert_eq!(
        String::from_utf8(failed.stderr).unwrap(),
        "SyntaxError: Unexpected '.' (line 2, column 30)\n"
    );
    let session = "class Rect(this.w, this.h):\n    def area() => .w * .h\n\nr = Rect(2, 3)\na = r.area()\nr.w = 5\na\nclass Q:\n    def f() => [1].map(i -> .w)\n\n.exit\n";
    let spelled = session.replace("=> .w * .h", "=> this.w * this.h");
    for options in [
        &["--plain"][..],
        &["--plain", "--json"][..],
        &["--json"][..],
    ] {
        for newline in ["\n", "\r\n"] {
            let read = repl(&session.replace('\n', newline), options);
            let written = repl(&spelled.replace('\n', newline), options);
            assert!(read.status.success(), "{read:?}");
            assert_eq!(read.stdout, written.stdout, "{options:?} {newline:?}");
            assert_eq!(read.stderr, written.stderr, "{options:?} {newline:?}");
            let stdout = String::from_utf8(read.stdout).unwrap();
            assert!(stdout.contains("15"), "{stdout}");
            assert_eq!(
                String::from_utf8(read.stderr).unwrap(),
                "SyntaxError: Unexpected '.' (line 2, column 30)\n"
            );
        }
    }
}
