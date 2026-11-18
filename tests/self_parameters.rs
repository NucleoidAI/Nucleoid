mod common;

use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::lexer::{Keyword, Token, tokenize};
use nucleoid::lang::ast::{ClassDecl, Expr, Stmt, parse, parse_expression};
use nucleoid::{ErrorKind, Position, Runtime};

const SPEC_TITLES: [&str; 5] = [
    "Nucleoid declares the receiver of methods as a self parameter",
    "Nucleoid reads self parameters in arrow and same line methods",
    "Nucleoid constructs subclasses through self parameter initializers and super",
    "Nucleoid applies class level rules to instances of self parameter methods",
    "Nucleoid rolls back failed self parameter method calls and constructions",
];
const REFERENCE_TITLE: &str = "Nucleoid accepts self parameters in methods and constructors";
const README_TITLE: &str = "Methods may declare self as Python methods do";
const EXAMPLES_TITLE: &str = "Python-style methods keep a tank farm's rules live";

const PARAMETER: &str = "Expected a parameter but found keyword 'this'";
const BARE: &str = "Expected '(' but found keyword 'this'";

const BANNER: &str = "🌿 Nucleoid - Logic Language for World Models\n🌎 Inspired by Nature\nType .help for available commands.\n";

fn statements(source: &str) -> Vec<Stmt> {
    parse(source).unwrap_or_else(|error| panic!("{source}: {error}"))
}

fn single_class(source: &str) -> ClassDecl {
    match statements(source).as_slice() {
        [Stmt::Class(class)] => class.clone(),
        other => panic!("{source} parses to {other:?}"),
    }
}

fn equivalent(receiver: &str, written: &str) {
    let receiver_tree = statements(receiver);
    let written_tree = statements(written);
    assert_eq!(receiver_tree, written_tree, "{receiver}");
    assert_eq!(
        generate_all(&receiver_tree),
        generate_all(&written_tree),
        "{receiver}"
    );
}

fn graph_shape(runtime: &Runtime) -> Vec<String> {
    runtime
        .graph
        .nodes()
        .map(|node| {
            let reads: Vec<String> = node.dependencies.iter().map(ToString::to_string).collect();
            let updates: Vec<String> = node.dependents.iter().map(ToString::to_string).collect();
            format!("{} [{}] {reads:?} {updates:?}", node.key, node.kind)
        })
        .collect()
}

fn result(runtime: &mut Runtime, source: &str) -> Result<String, String> {
    match runtime.run(source) {
        Ok(value) => Ok(runtime.serialize_json(&value).unwrap()),
        Err(error) => Err(error.to_string()),
    }
}

fn same_runs(receiver: &str, written: &str, tail: &str) -> Result<String, String> {
    let mut receiver_runtime = Runtime::new();
    let mut written_runtime = Runtime::new();
    assert_eq!(
        result(&mut receiver_runtime, receiver),
        result(&mut written_runtime, written),
        "{receiver}"
    );
    let receiver_result = result(&mut receiver_runtime, tail);
    let written_result = result(&mut written_runtime, tail);
    assert_eq!(receiver_result, written_result, "{receiver}\n{tail}");
    assert_eq!(
        graph_shape(&receiver_runtime),
        graph_shape(&written_runtime),
        "{receiver}\n{tail}"
    );
    receiver_result
}

fn fresh(source: &str, expected: &str) {
    let assertions = common::assertions::protected_assertions(&statements(source)).len();
    let case = common::Case {
        title: "self parameter regression".to_string(),
        source: source.to_string(),
        expected: Some(expected.to_string()),
    };
    common::check_with_count(&mut Runtime::new(), &case, assertions)
        .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
}

fn syntax_error(source: &str) -> (String, Position) {
    let error = Runtime::check(source).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
    (error.message(), error.position().unwrap())
}

fn spelled(source: &str) -> String {
    source
        .replace("def __init__(self, ", "def init(")
        .replace("def __init__(self)", "def init()")
        .replace("def __init__(this, ", "def init(")
        .replace("(self, ", "(")
        .replace("(this, ", "(")
        .replace("(self)", "()")
        .replace("(this)", "()")
}

fn receiver_windows(source: &str) -> usize {
    let tokens: Vec<Token> = tokenize(source)
        .unwrap()
        .into_iter()
        .map(|spanned| spanned.token)
        .collect();
    tokens
        .windows(5)
        .filter(|window| {
            matches!(
                window,
                [
                    Token::Keyword(Keyword::Def),
                    Token::Identifier(_),
                    Token::LeftParen,
                    Token::Keyword(Keyword::This),
                    Token::Comma | Token::RightParen
                ]
            )
        })
        .count()
}

fn repl(input: &str, options: &[&str]) -> Output {
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
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn receiver_methods_parse_to_the_trees_of_their_spellings_without_the_receiver() {
    for (receiver, written) in [
        (
            "class P:\n    def m(self):\n        return 1",
            "class P:\n    def m():\n        return 1",
        ),
        (
            "class P:\n    def m(this, a):\n        return a",
            "class P:\n    def m(a):\n        return a",
        ),
        (
            "class P:\n    def m(self, a, b): return a + b",
            "class P:\n    def m(a, b): return a + b",
        ),
        (
            "class P:\n    def area(self) => self.w * self.h",
            "class P:\n    def area() => self.w * self.h",
        ),
        (
            "class P:\n    def area(this) -> this.w * this.h",
            "class P:\n    def area() -> this.w * this.h",
        ),
        (
            "class P:\n    def area(self) =>\n        self.w * self.h",
            "class P:\n    def area() =>\n        self.w * self.h",
        ),
        (
            "class P:\n    def m(self, a = 1, b: int = 2, c: str, ...rest) => rest",
            "class P:\n    def m(a = 1, b: int = 2, c: str, ...rest) => rest",
        ),
        (
            "class P:\n    def m(self,) => 1\n    def n(self, a,) => a",
            "class P:\n    def m() => 1\n    def n(a,) => a",
        ),
        (
            "class P:\n    def m(\n        self,\n        a\n    ) => a",
            "class P:\n    def m(\n        a\n    ) => a",
        ),
        (
            "class P:\n    def m( self ) => 1\n    def n( self , a ) => a",
            "class P:\n    def m( ) => 1\n    def n( a ) => a",
        ),
        (
            "class P:\n    # methods\n    def m(self): # header\n        return 1 # body",
            "class P:\n    # methods\n    def m(): # header\n        return 1 # body",
        ),
        (
            "class P:\n    def m(self) => 1\n    def n(a) => a\n    def o(this) => 2",
            "class P:\n    def m() => 1\n    def n(a) => a\n    def o() => 2",
        ),
        (
            "class P(h):\n    this.h = h\n    def m(self) => this.h",
            "class P(h):\n    this.h = h\n    def m() => this.h",
        ),
        (
            "class Q extends P:\n    def m(self) => super",
            "class Q: P\n    def m() => super",
        ),
        (
            "class Q: P\n    def m(self, a):\n        return a",
            "class Q: P\n    def m(a):\n        return a",
        ),
        (
            "make = function() {\n    class P:\n        def m(self) => 1\n    return P\n}",
            "make = function() {\n    class P:\n        def m() => 1\n    return P\n}",
        ),
        (
            "class Outer:\n    def build(self):\n        class Inner:\n            def get(self) => 2\n        return Inner",
            "class Outer:\n    def build():\n        class Inner:\n            def get() => 2\n        return Inner",
        ),
        (
            "class Outer:\n    class Inner:\n        def m(self) => 3",
            "class Outer:\n    class Inner:\n        def m() => 3",
        ),
        (
            "if ready:\n    class P:\n        def m(self) => 1",
            "if ready:\n    class P:\n        def m() => 1",
        ),
        (
            "def factory():\n    class P:\n        def m(self, a) => a\n    return P",
            "def factory():\n    class P:\n        def m(a) => a\n    return P",
        ),
        (
            "{\n    class P:\n        def m(self) => 1\n}",
            "{\n    class P:\n        def m() => 1\n}",
        ),
        (
            "class P:\n    x: int\n    pass\n    def m(self) => this.x",
            "class P:\n    x: int\n    pass\n    def m() => this.x",
        ),
    ] {
        equivalent(receiver, written);
    }
    let class = single_class("class P:\n    def m(self, a, b) => a");
    let [method] = class.methods.as_slice() else {
        panic!("one method");
    };
    assert_eq!(method.name.as_deref(), Some("m"));
    let names: Vec<&str> = method
        .parameters
        .iter()
        .map(|parameter| parameter.name.as_str())
        .collect();
    assert_eq!(names, ["a", "b"]);
}

#[test]
fn init_and_dunder_init_with_the_receiver_declare_the_init_constructor() {
    for (receiver, written) in [
        (
            "class P:\n    def __init__(self, a):\n        this.a = a",
            "class P:\n    def init(a):\n        this.a = a",
        ),
        (
            "class P:\n    def __init__(this):\n        this.a = 1",
            "class P:\n    def init():\n        this.a = 1",
        ),
        (
            "class P:\n    def init(self, a):\n        this.a = a",
            "class P:\n    def init(a):\n        this.a = a",
        ),
        (
            "class P:\n    def __init__(self, a): this.a = a",
            "class P:\n    def init(a): this.a = a",
        ),
        (
            "class P:\n    def __init__(self, a) => 5",
            "class P:\n    def init(a) => 5",
        ),
        (
            "class P:\n    def __init__(self, a: int = 1, ...rest):\n        pass",
            "class P:\n    def init(a: int = 1, ...rest):\n        pass",
        ),
        (
            "class P:\n    def __init__(self, this.a, self.b = 2, c):\n        this.c = c",
            "class P:\n    def init(this.a, self.b = 2, c):\n        this.c = c",
        ),
        (
            "class P:\n    def __init__(self, ...this.rest) => 0",
            "class P:\n    def init(...this.rest) => 0",
        ),
        (
            "class P extends Q:\n    def __init__(self, a):\n        super(a)",
            "class P: Q\n    def init(a):\n        super(a)",
        ),
    ] {
        equivalent(receiver, written);
    }
    let class = single_class("class P:\n    def __init__(self, a):\n        this.a = a");
    let [method] = class.methods.as_slice() else {
        panic!("one method");
    };
    assert_eq!(method.name.as_deref(), Some("init"));
    fresh(
        "class Account:\n    def __init__(self, owner, balance = 0):\n        self.owner = owner\n        self.balance = balance\na = Account(\"ada\", 5)\nb = Account(balance=9, owner=\"bo\")\nc = Account(\"cy\")\nassert([a.balance, b.owner, b.balance, c.balance], [5, \"bo\", 9, 0])\nAccount.length",
        "3",
    );
}

#[test]
fn dunder_init_without_the_receiver_stays_an_ordinary_method() {
    let class =
        single_class("class P:\n    def __init__(balance):\n        this.balance = balance");
    let [method] = class.methods.as_slice() else {
        panic!("one method");
    };
    assert_eq!(method.name.as_deref(), Some("__init__"));
    fresh(
        "class Account:\n    def __init__(balance):\n        this.balance = balance\na = Account(100)\nassert(a.balance, null)\na.__init__(5)\nassert(a.balance, 5)\na.balance",
        "5",
    );
    fresh(
        "class C:\n    def __str__(self) => \"C\"\nC().__str__()",
        "\"C\"",
    );
}

#[test]
fn constructors_bind_and_adopt_as_their_spellings() {
    let header = "class P:\n    def __init__(self, x, y = 7, ...rest):\n        this.x = x\n        this.y = y\n        this.rest = rest";
    let written = spelled(header);
    for (call, expected) in [
        ("P(1)", "[1.0,7.0,[]]"),
        ("P(1, 2, 3, 4)", "[1.0,2.0,[3.0,4.0]]"),
        ("P(y=2, x=1)", "[1.0,2.0,[]]"),
        ("P(5, y=6)", "[5.0,6.0,[]]"),
        ("P()", "[null,7.0,[]]"),
    ] {
        assert_eq!(
            same_runs(header, &written, &format!("p = {call}\n[p.x, p.y, p.rest]")),
            Ok(expected.to_string()),
            "{call}"
        );
    }
    for (call, error) in [
        (
            "P(z=1)",
            "TypeError: Unknown constructor argument 'z' for P",
        ),
        (
            "P(1, x=2)",
            "TypeError: Duplicate constructor argument 'x' for P",
        ),
        (
            "P(rest=[1])",
            "TypeError: Rest parameter 'rest' cannot be a named constructor argument for P",
        ),
    ] {
        assert_eq!(
            same_runs(header, &written, &format!("p = {call}")),
            Err(format!("{error} (line 1, column 1)")),
            "{call}"
        );
    }
    for (receiver, tail, expected) in [
        (
            "class P:\n    def init(self, x):\n        this.x = x\n    def __init__(self, y):\n        this.y = y",
            "p = P(1)\n[p.x, p.y]",
            "[null,1.0]",
        ),
        (
            "class P:\n    def __init__(self, y):\n        this.y = y\n    def init(self, x):\n        this.x = x",
            "p = P(1)\n[p.x, p.y]",
            "[1.0,null]",
        ),
        (
            "class P(a):\n    this.a = a\n    def __init__(self, b):\n        this.b = b",
            "p = P(1)\np.init(2)\n[p.a, p.b]",
            "[1.0,2.0]",
        ),
        (
            "class P:\n    def __init__(self, this.x, self.y = 3):\n        this.sum = x + y",
            "p = P(1)\n[p.x, p.y, p.sum]",
            "[1.0,3.0,4.0]",
        ),
        (
            "class P:\n    def __init__(self, a):\n        this.a = a",
            "p = P(1)\nclass P:\n    def init(a, b):\n        this.a = a\n        this.b = b\nq = P(2, 3)\n[P.length, p.b, q.b]",
            "[2.0,null,3.0]",
        ),
    ] {
        let written = spelled(receiver);
        assert_eq!(
            same_runs(receiver, &written, tail),
            Ok(expected.to_string()),
            "{receiver}"
        );
    }
}

#[test]
fn subclasses_reach_their_parents_through_super() {
    let receiver = "class Animal:\n    def __init__(self, name):\n        self.name = name\n    def speak(self) => self.name + \" speaks\"\nclass Dog extends Animal:\n    def __init__(self, name, breed):\n        super(name)\n        self.breed = breed\n    def speak(self) => self.name + \" barks\"\nclass Pup: Dog\n    def tag(this) => \"pup \" + this.name";
    let written = spelled(receiver);
    equivalent(receiver, &written);
    assert_eq!(
        same_runs(
            receiver,
            &written,
            "rex = Dog(\"Rex\", \"lab\")\nbit = Pup(\"Bit\", \"pug\")\ncat = Animal(\"Cat\")\n[rex.name, rex.breed, rex.speak(), cat.speak(), bit.name, bit.tag(), Dog.length]"
        ),
        Ok("[\"Rex\",\"lab\",\"Rex barks\",\"Cat speaks\",\"Bit\",\"pup Bit\",1.0]".to_string())
    );
    fresh(
        "class Base:\n    def __init__(self):\n        this.ready = true\nclass Child extends Base:\n    def __init__(self):\n        super\n        this.code = 7\nc = Child()\nassert([c.ready, c.code], [true, 7])\nc.code",
        "7",
    );
}

#[test]
fn python_forms_outside_the_lowering_keep_their_accepted_behavior() {
    fresh(
        "class Animal:\n    def __init__(self, name):\n        self.name = name\nclass Dog extends Animal:\n    def __init__(self, name, breed):\n        super().__init__(name)\n        self.breed = breed\nd = Dog(\"Rex\", \"lab\")\nassert([d.name, d.breed], [null, \"lab\"])\nd.breed",
        "\"lab\"",
    );
    let class =
        single_class("class Dog(Animal):\n    def __init__(self, name):\n        self.name = name");
    assert_eq!(class.parent, None);
    assert_eq!(class.parameters.len(), 1);
    assert_eq!(class.parameters[0].name, "Animal");
    fresh(
        "class Animal:\n    def __init__(self, name):\n        self.name = name\nclass Dog(Animal):\n    def bark(self) => \"woof\"\nd = Dog(\"Rex\")\nassert([d.name, d.bark(), Animal.length, Dog.length], [null, \"woof\", 0, 1])\nd.bark()",
        "\"woof\"",
    );
}

#[test]
fn every_other_parameter_list_keeps_its_accepted_error_and_position() {
    for (source, message, line, column) in [
        (
            "class P:\n    def m(self: P):\n        return 1",
            PARAMETER,
            2,
            15,
        ),
        ("class P:\n    def m(this: P) => 1", PARAMETER, 2, 15),
        ("class P:\n    def m(self = 1) => 1", PARAMETER, 2, 16),
        ("class P:\n    def m(...self) => 1", PARAMETER, 2, 18),
        ("class P:\n    def m(self.x) => 1", PARAMETER, 2, 15),
        ("class P:\n    def m(self, self) => 1", PARAMETER, 2, 15),
        ("class P:\n    def m(self, this) => 1", PARAMETER, 2, 15),
        ("class P:\n    def m(self, a, self) => 1", PARAMETER, 2, 15),
        ("class P:\n    def m(self, self.x) => 1", PARAMETER, 2, 15),
        ("class P:\n    def m(x, self) => 1", PARAMETER, 2, 18),
        ("def f(self) => 1", PARAMETER, 1, 11),
        ("def f(self, x) => x", PARAMETER, 1, 11),
        ("def f(this):\n    return 1", PARAMETER, 1, 11),
        (
            "def outer():\n    def inner(self) => 1\n    return 1",
            PARAMETER,
            2,
            19,
        ),
        (
            "class P:\n    def m(self):\n        def inner(self) => 1\n        return 1",
            PARAMETER,
            2,
            15,
        ),
        ("f = function(self) { return 1 }", PARAMETER, 1, 18),
        ("f = (self) => 1", PARAMETER, 1, 10),
        ("f = (self, x) -> x", PARAMETER, 1, 10),
        ("f = lambda self: 1", "Unexpected ':'", 1, 18),
        ("f = lambda self, x: x", "Unexpected ','", 1, 18),
        (
            "class P:\n    function m(self) { return 1 }",
            PARAMETER,
            2,
            20,
        ),
        (
            "class P:\n    function init(self, x) { this.x = x }",
            PARAMETER,
            2,
            23,
        ),
        (
            "class P:\n    function __init__(self) { return 1 }",
            PARAMETER,
            2,
            27,
        ),
        (
            "class P:\n    if true:\n        def m(self) => 1",
            PARAMETER,
            3,
            19,
        ),
        (
            "class P:\n    if true:\n        def init(self, x):\n            this.x = x",
            PARAMETER,
            3,
            22,
        ),
        (
            "class P:\n    {\n        def m(self) => 1\n    }",
            PARAMETER,
            3,
            19,
        ),
        (
            "class P:\n    for i of [1]:\n        def m(self) => 1",
            PARAMETER,
            3,
            19,
        ),
        ("class P: def m(self): 1", PARAMETER, 1, 20),
        ("class P: def init(self, x): this.x = x", PARAMETER, 1, 23),
        ("class P(a): def m(self) => 1", PARAMETER, 1, 23),
        ("class P(self)", PARAMETER, 1, 13),
        ("class P(self, x)", PARAMETER, 1, 13),
        ("class P(this):\n    pass", PARAMETER, 1, 13),
        ("def f self:\n    return 1", BARE, 1, 7),
        ("def f self, x => x", BARE, 1, 7),
        ("class P:\n    def m self:\n        return 1", BARE, 2, 11),
        (
            "class P:\n    def init self, x:\n        this.x = x",
            BARE,
            2,
            14,
        ),
        (
            "class P:\n    def __init__(self.x):\n        pass",
            PARAMETER,
            2,
            22,
        ),
        (
            "class P:\n    def __init__(x, self):\n        pass",
            PARAMETER,
            2,
            25,
        ),
        (
            "z = (1 if function(){\nclass P:\n    def m(self):\n        1\n} else 2)\nz",
            "Expected ')' but found keyword 'if'",
            1,
            8,
        ),
        (
            "z = [function() {\nclass P:\n    def m(self): 1\nreturn P().m()\n}]\nz",
            PARAMETER,
            3,
            15,
        ),
    ] {
        assert_eq!(
            syntax_error(source),
            (message.to_string(), Position::new(line, column)),
            "{source}"
        );
    }
    fresh(
        "class P:\n    def init(self.x, this.y) => 0\np = P(3, 4)\n[p.x, p.y]",
        "[3,4]",
    );
}

#[test]
fn when_the_receiver_parse_fails_too_the_accepted_error_is_reported() {
    for (source, message, line, column) in [
        ("class P:\n    def m(self) => 1\nx = )", PARAMETER, 2, 15),
        (
            "class P:\n    def m(self) => 1\ndef f(self) => 2",
            PARAMETER,
            2,
            15,
        ),
        (
            "class P:\n    def m(self):\n        return )",
            PARAMETER,
            2,
            15,
        ),
        ("class P:\n    def m(self, a: 5) => 1", PARAMETER, 2, 15),
        ("class P:\n    def m(self, a = b) => 1", PARAMETER, 2, 15),
        ("class P:\n    def m(self, ...a, b) => 1", PARAMETER, 2, 15),
        ("class P:\n    def m(self,, a) => 1", PARAMETER, 2, 15),
        ("class P:\n    def m(self)", PARAMETER, 2, 15),
        ("class P:\n    def m(self\n", PARAMETER, 3, 1),
        (
            "y = 2\nt = `a ${function() { class Q:\n    def m(self): 1 }}`\ny",
            PARAMETER,
            3,
            15,
        ),
    ] {
        assert_eq!(
            syntax_error(source),
            (message.to_string(), Position::new(line, column)),
            "{source}"
        );
    }
}

#[test]
fn accepted_programs_holding_receiver_methods_keep_their_parse() {
    for (source, head, value) in [
        (
            "`${1 if function(){ class P: def m(self): 1 } else 2}`",
            "`${1}`",
            "\"1\"",
        ),
        (
            "`${1 if function(){\nclass P:\n    def m(self): 1\n} else 2}`",
            "`${1}`",
            "\"1\"",
        ),
        (
            "r = `${1 if function(){ class P:\n    def m(self): 1\n} else 2}`\nr",
            "r = `${1}`\nr",
            "\"1\"",
        ),
        (
            "`${1 if function(){ def m(self): 1 } else 2}`",
            "`${1}`",
            "\"1\"",
        ),
        (
            "`${2 if function(){\nclass P:\n    def m(self): 1\n}() else 3}`",
            "`${2}`",
            "\"2\"",
        ),
    ] {
        assert_eq!(statements(source), statements(head), "{source}");
        assert_eq!(result(&mut Runtime::new(), source), Ok(value.to_string()));
    }
    for source in [
        "1 if function(){\nclass P:\n    def m(self): 1\n} else 2",
        "1 if function(){ class P: def m(self): 1 } else 2",
    ] {
        assert_eq!(
            parse_expression(source).unwrap(),
            Expr::Number(1.0),
            "{source}"
        );
    }
}

#[test]
fn a_failing_program_reads_receivers_everywhere_templates_included() {
    let mixed = "class A:\n    def m(self): 1\n`${2 if function(){\nclass P:\n    def m(self): 1\n}() else 3}`";
    equivalent(mixed, &spelled(mixed));
    assert_eq!(result(&mut Runtime::new(), mixed), Ok("\"2\"".to_string()));
    for source in [
        "s = `${function() {\nclass P:\n    def m(self) => 4\nreturn P().m()\n}()}`\ns",
        "s = `a${1}b${function() {\nclass P:\n    def m(self, x) => x\nreturn P().m(7)\n}()}`\ns",
        "`${2}` + `${function(){\nclass P:\n    def m(self): 1\n}}`",
    ] {
        equivalent(source, &spelled(source));
    }
    assert_eq!(
        result(
            &mut Runtime::new(),
            "s = `${function() {\nclass P:\n    def m(self) => 4\nreturn P().m()\n}()}`\ns"
        ),
        Ok("\"4\"".to_string())
    );
}

#[test]
fn tentative_statements_take_the_reading_of_their_spelling() {
    for (source, expected) in [
        (
            "{k} = {k: function() {\n    class P:\n        def m(self) => 1\n    return P().m()\n}}\nk()",
            "1",
        ),
        (
            "{k: v} = {k: function() {\n    class P:\n        def m(this) => 2\n    return P().m()\n}}\nv()",
            "2",
        ),
        (
            "{k, j} = {j: 3, k: function() {\n    class P:\n        def m(self, a) => a\n    return P().m(j)\n}}\nk()",
            "3",
        ),
        (
            "a, b = function() {\n    class P:\n        def m(self) => [4, 5]\n    return P().m()\n}()\n[a, b]",
            "[4,5]",
        ),
        (
            "n = 0\nwhile n < function() {\n    class P:\n        def m(self) => 2\n    return P().m()\n}():\n    n += 1\nn",
            "2",
        ),
        (
            "suppose f = function() {\n    class P:\n        def m(self) => 3\n    return P().m()\n}:\n    v = f()\n    assert(v, 3)\n1",
            "1",
        ),
        (
            "x = 1 if function() {\n    class P:\n        def m(self) => true\n    return P().m()\n}() else 2\nx",
            "1",
        ),
        (
            "if function() {\n    class P:\n        def m(self) => true\n    return P().m()\n}():\n    r = 5\nr",
            "5",
        ),
        (
            "class A:\n    def m(self) => 1\nx = 1\nflag = (x is 1)\nflag",
            "true",
        ),
        (
            "class A:\n    def m(self, xs) => [v * 2 for v in xs]\nA().m([1, 2])",
            "[2,4]",
        ),
        (
            "class A(n):\n    this.n = n\n    def m(self) => 5\n$A.double = .n * 2\n[A(3).double, A(1).m()]",
            "[6,5]",
        ),
        (
            "o = {make: function() {\n    class P:\n        def m(self) => 1\n    return P().m()\n}}\no.make()",
            "1",
        ),
        (
            "try:\n    class P:\n        def m(self) => 1\n    r = P().m()\ncatch e:\n    r = 0\nr",
            "1",
        ),
        (
            "assert: function() {\n    class P:\n        def m(self) => 1\n    return P().m()\n}(), 1\n2",
            "2",
        ),
    ] {
        equivalent(source, &spelled(source));
        fresh(source, expected);
    }
    for source in [
        "class A:\n    def m(self) => 1\nv = 1\nwhile v is a: b\nv",
        "o = {}\no.f = function() {\n    class P:\n        def m(self) => 1\n    while 1 + .x is a: b\n    return P().m()\n}\n1",
    ] {
        equivalent(source, &spelled(source));
    }
    for (source, error) in [
        (
            "class A:\n    def m(self) => 1\nflag = (1 is 1)\nwhile function() {\n    class P:\n        def m(self) => 1\n    return 1\n}() is a: b\nflag",
            "ReferenceError: while is not defined (line 4, column 1)",
        ),
        (
            "class A:\n    def m(self) => 1\nflag = (1 is 1)\nsuppose function() {\n    class P:\n        def m(self) => 1\n    return 1\n}() is a: b\nflag",
            "ReferenceError: suppose is not defined (line 4, column 1)",
        ),
    ] {
        equivalent(source, &spelled(source));
        assert_eq!(result(&mut Runtime::new(), source), Err(error.to_string()));
        assert_eq!(
            result(&mut Runtime::new(), &spelled(source)),
            Err(error.to_string())
        );
    }
    fresh(
        "class A:\n    def m(self) => 1\nflag = (1 is 1)\nn = 0\nwhile n < function() {\n    class P:\n        def m(self) => 2\n    return P().m()\n}() is true:\n    n += 1\n[flag, n]",
        "[true,2]",
    );
}

#[test]
fn methods_explain_render_and_update_as_their_spellings() {
    let receiver = "class Tank:\n    def __init__(self, level, capacity):\n        self.level = level\n        self.capacity = capacity\n    def pump(self, amount):\n        self.level = self.level + amount\n        return self.level\n    def free(self) => self.capacity - self.level";
    let written = spelled(receiver);
    let mut receiver_runtime = Runtime::new();
    let mut written_runtime = Runtime::new();
    for (receiver_source, written_source) in [
        (receiver, written.as_str()),
        (
            "$Tank.full = $Tank.level >= $Tank.capacity",
            "$Tank.full = $Tank.level >= $Tank.capacity",
        ),
        ("east = Tank(70, 80)", "east = Tank(70, 80)"),
        (
            "spare = east.capacity - east.level",
            "spare = east.capacity - east.level",
        ),
        ("east.pump(10)", "east.pump(10)"),
        (
            "[east.full, spare, east.free()]",
            "[east.full, spare, east.free()]",
        ),
        ("why east.full", "why east.full"),
        ("affects east.level", "affects east.level"),
        ("model |> why", "model |> why"),
        ("(why east.full)[0].rule", "(why east.full)[0].rule"),
    ] {
        let receiver_result = result(&mut receiver_runtime, receiver_source);
        let written_result = result(&mut written_runtime, written_source);
        assert_eq!(receiver_result, written_result, "{receiver_source}");
        assert_eq!(
            graph_shape(&receiver_runtime),
            graph_shape(&written_runtime),
            "{receiver_source}"
        );
    }
    assert_eq!(generate_all(&statements(receiver)), "class Tank");
    let defaults = "class Tank:\n    def __init__(self, level = 0):\n        self.level = level\n    def pump(self, amount = 1) => self.level + amount";
    let rendered = generate_all(&statements(defaults));
    assert_eq!(rendered, generate_all(&statements(&spelled(defaults))));
    assert_eq!(
        rendered,
        "class Tank():\n    def init(level=0):\n        this.level=level\n    def pump(amount=1):\n        return this.level+amount"
    );
}

#[test]
fn failed_calls_and_constructions_roll_back_as_their_spellings() {
    let receiver = "class Account:\n    def __init__(self, balance):\n        if balance < 0:\n            throw \"NEGATIVE\"\n        self.balance = balance\n    def withdraw(self, amount):\n        self.balance = self.balance - amount\n        if self.balance < 0:\n            throw \"OVERDRAWN\"\n        return self.balance";
    let written = spelled(receiver);
    for (tail, expected) in [
        (
            "a = Account(5)\na.withdraw(9)",
            Err("OVERDRAWN (line 2, column 1)".to_string()),
        ),
        (
            "a = Account(5)\nr = \"\"\ntry:\n    a.withdraw(9)\ncatch e:\n    r = e\n[r, a.balance]",
            Ok("[\"OVERDRAWN\",5.0]".to_string()),
        ),
        (
            "r = \"\"\ntry:\n    Account(-1)\ncatch e:\n    r = e\n[r, Account.length]",
            Ok("[\"NEGATIVE\",0.0]".to_string()),
        ),
        (
            "Account(-1)",
            Err("NEGATIVE (line 1, column 1)".to_string()),
        ),
    ] {
        assert_eq!(same_runs(receiver, &written, tail), expected, "{tail}");
    }
}

#[test]
fn the_cli_runs_a_file_of_receiver_methods() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("self-parameters.nuc");
    let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .args(["--json", "--graph"])
        .arg(fixture)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.lines().next(), Some("[80.0,true,0.0]"));
    assert!(stdout.contains("$Tank [class]"), "{stdout}");
    assert!(stdout.contains("east.full [property]"), "{stdout}");
}

#[test]
fn the_native_repl_runs_receiver_methods_as_their_spellings() {
    let session = "class Tank:\n    def __init__(self, level):\n        self.level = level\n    def pump(self, amount):\n        self.level = self.level + amount\n        return self.level\n\ntank = Tank(3)\ntank.pump(4)\ntank.level\n.exit\n";
    for options in [&["--plain"][..], &["--plain", "--json"][..]] {
        for input in [session.to_string(), session.replace('\n', "\r\n")] {
            let receiver = repl(&input, options);
            let written = repl(&spelled(&input), options);
            assert!(receiver.status.success(), "{receiver:?}");
            assert_eq!(receiver.stdout, written.stdout, "{options:?}");
            assert_eq!(receiver.stderr, written.stderr, "{options:?}");
        }
    }
    let output = repl(session, &["--plain", "--json"]);
    assert_eq!(String::from_utf8(output.stderr).unwrap(), "");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("{BANNER}null\n{{\"id\":\"tank\",\"level\":3.0}}\n7.0\n7.0\n")
    );
}

fn nested_methods(depth: usize, receiver: &str) -> String {
    let mut source = String::new();
    for level in 0..depth {
        let indent = "    ".repeat(2 * level);
        source.push_str(&format!(
            "{indent}class C{level}:\n{indent}    def m({receiver}):\n"
        ));
    }
    source.push_str(&"    ".repeat(2 * depth));
    source.push('1');
    source
}

fn deepest(build: impl Fn(usize) -> String) -> (usize, String) {
    let started = Instant::now();
    for depth in 1..80 {
        let checked = Runtime::check(&build(depth));
        assert!(
            started.elapsed() < Duration::from_secs(20),
            "depth {depth} takes too long"
        );
        if let Err(error) = checked {
            assert_eq!(error.kind(), ErrorKind::Syntax, "depth {depth}: {error}");
            return (depth, error.message());
        }
    }
    panic!("no depth limit was reached");
}

#[test]
fn nested_and_long_programs_parse_receivers_as_their_spellings() {
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(nested_and_long_programs)
        .unwrap()
        .join()
        .unwrap();
}

fn nested_and_long_programs() {
    let templated = |receiver: &'static str| {
        move |depth| {
            format!(
                "s = `${{function() {{\n{}\n}}}}`",
                nested_methods(depth, receiver)
            )
        }
    };
    for (receiver, written) in [
        (
            deepest(|depth| nested_methods(depth, "self")),
            deepest(|depth| nested_methods(depth, "")),
        ),
        (deepest(templated("self")), deepest(templated(""))),
    ] {
        assert!(receiver.0 > 15, "{receiver:?}");
        assert_eq!(receiver.0, written.0);
        assert_eq!(receiver.1, PARAMETER);
        assert!(written.1.ends_with("nested too deeply"), "{written:?}");
    }
    let methods: String = (0..2000)
        .map(|index| format!("    def m{index}(self, a) => a + {index}\n"))
        .collect();
    let class = format!("class Wide:\n{methods}w = Wide()\n[w.m0(1), w.m1999(1)]");
    equivalent(&class, &spelled(&class));
    fresh(&class, "[1,2000]");
    let lines: String = (0..5000)
        .map(|index| format!("v{index} = {index}\n"))
        .collect();
    let long = format!("{lines}class Last:\n    def m(self) => v4999\nLast().m()");
    equivalent(&long, &spelled(&long));
    fresh(&long, "4999");
}

#[test]
fn only_the_appended_cases_use_self_parameters_and_they_run_every_assertion() {
    let documents = [
        common::cases(include_str!("../nucleoid.spec.md")),
        common::cases(include_str!("reference.md")),
        common::snippets(include_str!("../README.md"), "README.md"),
        common::snippets(include_str!("../docs/examples.md"), "docs/examples.md"),
    ];
    let appended: Vec<&str> = SPEC_TITLES
        .into_iter()
        .chain([REFERENCE_TITLE, README_TITLE, EXAMPLES_TITLE])
        .collect();
    let mut cases = 0;
    let mut assertions = 0;
    for case in documents.iter().flatten() {
        cases += 1;
        let new = appended.contains(&case.title.as_str());
        assert_eq!(receiver_windows(&case.source) > 0, new, "{}", case.title);
        if new {
            let count = common::assertions::protected_assertions(&statements(&case.source)).len();
            assert!(count > 0, "{} asserts its behavior", case.title);
            common::check_with_assertions(case, count)
                .unwrap_or_else(|reason| panic!("{}: {reason}", case.title));
            assertions += count;
            let written = spelled(&case.source);
            assert_eq!(receiver_windows(&written), 0, "{}", case.title);
            assert_eq!(
                statements(&case.source),
                statements(&written),
                "{}",
                case.title
            );
        }
    }
    for title in &appended {
        let matching = documents
            .iter()
            .flatten()
            .filter(|case| case.title == *title)
            .count();
        assert_eq!(matching, 1, "{title}");
    }
    assert_eq!(assertions, 22);
    assert_eq!(cases, 1189);
}

#[test]
fn the_appended_cases_stand_after_nuc_106_and_before_nuc_103() {
    let spec = common::cases(include_str!("../nucleoid.spec.md"));
    let titles: Vec<&str> = spec.iter().map(|case| case.title.as_str()).collect();
    assert_eq!(
        titles[744],
        "Nucleoid restores intersections after failed changes"
    );
    assert_eq!(titles[745..750], SPEC_TITLES);
    assert_eq!(titles[793], "Nucleoid flattens nested lists by one level");
    for (cases, title, previous, following) in [
        (
            common::cases(include_str!("reference.md")),
            REFERENCE_TITLE,
            "Nucleoid compares two explanations with intersect and except",
            "Nucleoid flattens nested lists with flat",
        ),
        (
            common::snippets(include_str!("../README.md"), "README.md"),
            README_TITLE,
            "Intersect and except tell apart what two symptoms rest on",
            "Flat turns nested lists into one live list",
        ),
        (
            common::snippets(include_str!("../docs/examples.md"), "docs/examples.md"),
            EXAMPLES_TITLE,
            "Except shows what a steel delay moves and a glass delay does not",
            "Flattening keeps the dirty tiles of every room counted",
        ),
    ] {
        let index = cases.iter().position(|case| case.title == title).unwrap();
        assert_eq!(cases[index - 1].title, previous);
        assert_eq!(cases[index + 12].title, following);
    }
}

#[test]
fn the_language_reference_shows_the_reference_case_verbatim() {
    let reference = include_str!("../docs/README.md").replace("\r\n", "\n");
    let (_, section) = reference
        .split_once("### Python-style self parameters\n")
        .expect("the language reference documents self parameters");
    let (_, example) = section
        .split_once("\n```\n")
        .expect("the section shows an example");
    let (example, _) = example.split_once("\n```").unwrap();
    let case = common::cases(include_str!("reference.md"))
        .into_iter()
        .find(|case| case.title == REFERENCE_TITLE)
        .expect("the reference case is appended");
    let (_, source) = case.source.trim().split_once("\n\n").unwrap();
    assert_eq!(example.trim(), source.trim());
    assert_eq!(case.expected.as_deref(), Some("14"));
    assert!(reference.contains("[NUC 107](nuc-0107.md)"));
}
