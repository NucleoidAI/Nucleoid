use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Stdio};

use nucleoid::lang::ast::generator::generate_all;
use nucleoid::lang::ast::{Stmt, parse, parse_program};
use nucleoid::{ErrorKind, Position, Runtime};

fn equivalent(extends: &str, colon: &str) {
    let extends_ast = parse(extends).unwrap_or_else(|error| panic!("{extends}: {error}"));
    let colon_ast = parse(colon).unwrap_or_else(|error| panic!("{colon}: {error}"));
    assert_eq!(extends_ast, colon_ast, "{extends}");
    assert_eq!(
        generate_all(&extends_ast),
        generate_all(&colon_ast),
        "{extends}"
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

#[test]
fn colon_clauses_declare_exactly_the_classes_of_the_colon_spelling() {
    for (extends, colon) in [
        (
            "class Dog extends Animal:\n    pass",
            "class Dog: Animal\n    pass",
        ),
        (
            "class Dog extends Animal:\n    this.sound = \"woof\"",
            "class Dog: Animal\n    this.sound = \"woof\"",
        ),
        (
            "class Dog(name, breed) extends Animal:\n    this.name = name\n    this.breed = breed",
            "class Dog(name, breed): Animal\n    this.name = name\n    this.breed = breed",
        ),
        (
            "class Dog(name: str) extends Animal:\n    breed: str\n    def init(name: str, breed: str):\n        super(name)\n        this.breed = breed\n    def speak() => \"woof\"",
            "class Dog(name: str): Animal\n    breed: str\n    def init(name: str, breed: str):\n        super(name)\n        this.breed = breed\n    def speak() => \"woof\"",
        ),
        (
            "class Dog(name = \"rex\", ...tags) extends Animal:\n    this.tags = tags",
            "class Dog(name = \"rex\", ...tags): Animal\n    this.tags = tags",
        ),
        (
            "class Dog(\n    name,\n    breed,\n) extends Animal:\n    this.name = name",
            "class Dog(\n    name,\n    breed,\n): Animal\n    this.name = name",
        ),
        (
            "class Dog extends Animal: # parent\n    pass",
            "class Dog: Animal # parent\n    pass",
        ),
        (
            "class Dog extends Animal:\n\n    # body\n    pass",
            "class Dog: Animal\n\n    # body\n    pass",
        ),
        (
            "class Dog extends Animal: this.sound = \"woof\"",
            "class Dog: Animal\n    this.sound = \"woof\"",
        ),
        (
            "class Dog extends Animal: sound = \"woof\"",
            "class Dog: Animal\n    sound = \"woof\"",
        ),
        ("class Dog extends Animal:", "class Dog: Animal"),
        ("class Dog extends Animal:\n", "class Dog: Animal\n"),
        (
            "class Dog extends Animal:\nnext = 1",
            "class Dog: Animal\nnext = 1",
        ),
        (
            "class Dog extends extends:\n    pass",
            "class Dog: extends\n    pass",
        ),
        (
            "class extends extends Animal:\n    pass",
            "class extends: Animal\n    pass",
        ),
        (
            "if ready:\n    class Dog extends Animal:\n        pass\nnext = 1",
            "if ready:\n    class Dog: Animal\n        pass\nnext = 1",
        ),
        (
            "class Outer:\n    class Inner extends Outer:\n        pass\n    this.count = 1",
            "class Outer:\n    class Inner: Outer\n        pass\n    this.count = 1",
        ),
    ] {
        equivalent(extends, colon);
    }
}

#[test]
fn bodyless_clauses_declare_the_pass_bodied_colon_classes() {
    for (extends, colon) in [
        ("class Dog extends Animal", "class Dog: Animal\n    pass"),
        (
            "class Dog() extends Animal",
            "class Dog(): Animal\n    pass",
        ),
        (
            "class Dog(name: str, breed) extends Animal",
            "class Dog(name: str, breed): Animal\n    pass",
        ),
        (
            "class Dog extends Animal\nnext = 1",
            "class Dog: Animal\n    pass\nnext = 1",
        ),
        (
            "class Dog extends Animal # bodyless\n\n# next statement\nnext = 1",
            "class Dog: Animal\n    pass\nnext = 1",
        ),
        (
            "class Dog extends Animal; next = 1",
            "class Dog: Animal\n    pass\nnext = 1",
        ),
        (
            "{ class Dog extends Animal }",
            "{\n    class Dog: Animal\n        pass\n}",
        ),
        (
            "{ class Dog extends Animal; next = 1 }",
            "{\n    class Dog: Animal\n        pass\n    next = 1\n}",
        ),
        (
            "if active:\n    class Dog extends Animal",
            "if active:\n    class Dog: Animal\n        pass",
        ),
        (
            "if active:\n    class Dog extends Animal\nelse:\n    next = 1",
            "if active:\n    class Dog: Animal\n        pass\nelse:\n    next = 1",
        ),
        (
            "if active: class Dog extends Animal; next = 1",
            "if active:\n    class Dog: Animal\n        pass\n    next = 1",
        ),
        (
            "for item of items:\n    class Dog extends Animal\nnext = 1",
            "for item of items:\n    class Dog: Animal\n        pass\nnext = 1",
        ),
        (
            "try:\n    class Dog extends Animal\ncatch error:\n    pass\nnext = 1",
            "try:\n    class Dog: Animal\n        pass\ncatch error:\n    pass\nnext = 1",
        ),
        (
            "def setup():\n    class Dog extends Animal\n    return 1",
            "def setup():\n    class Dog: Animal\n        pass\n    return 1",
        ),
        (
            "class Outer:\n    class Inner extends Outer\n    this.count = 1",
            "class Outer:\n    class Inner: Outer\n        pass\n    this.count = 1",
        ),
    ] {
        equivalent(extends, colon);
    }

    let statements = parse("class Dog(name) extends Animal").unwrap();
    let [Stmt::Class(class)] = statements.as_slice() else {
        panic!("expected one class declaration");
    };
    assert_eq!(class.name, "Dog");
    assert_eq!(class.parent.as_deref(), Some("Animal"));
    assert_eq!(class.parameters.len(), 1);
    assert!(class.fields.is_empty());
    assert!(class.constructor.is_empty());
    assert!(class.methods.is_empty());
    assert_eq!(generate_all(&statements), "class Dog");
}

#[test]
fn bodyless_clauses_keep_following_statements_and_positions_outside_the_class() {
    let source = "class Animal\nclass Dog extends Animal\nnext = missing";
    let program = parse_program(source).unwrap();
    assert_eq!(
        program.positions,
        [
            Position::new(1, 1),
            Position::new(2, 1),
            Position::new(3, 1)
        ]
    );

    let error = Runtime::new().run(source).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "missing is not defined");
    assert_eq!(error.position(), Some(Position::new(3, 1)));

    let mut runtime = Runtime::new();
    let value = runtime
        .run("class Animal(name):\n    this.name = name\nclass Dog extends Animal\nrex = Dog(\"Rex\")\n[rex.name, Animal.length, Dog.length]")
        .unwrap();
    assert_eq!(runtime.serialize_json(&value).unwrap(), "[\"Rex\",0.0,1.0]");

    let mut runtime = Runtime::new();
    let error = runtime
        .run("class Animal(name):\n    this.name = name\nclass Dog extends Animal:\nrex = Dog(\"Rex\")\nrex")
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "rex is not defined");
}

#[test]
fn malformed_clauses_report_exact_syntax_errors_at_their_tokens() {
    for (source, message, line, column) in [
        (
            "class Dog extends",
            "Expected a parent class name but found end of input",
            1,
            18,
        ),
        (
            "class Dog extends\nnext = 1",
            "Expected a parent class name but found end of line",
            1,
            11,
        ),
        (
            "class Dog extends;",
            "Expected a parent class name but found ';'",
            1,
            18,
        ),
        (
            "class Dog extends: Animal",
            "Expected a parent class name but found ':'",
            1,
            18,
        ),
        (
            "class Dog extends 5",
            "Expected a parent class name but found number 5",
            1,
            19,
        ),
        (
            "class Dog extends \"A\"",
            "Expected a parent class name but found a string",
            1,
            19,
        ),
        (
            "class Dog extends $Animal",
            "Expected a parent class name but found '$Animal'",
            1,
            19,
        ),
        (
            "class Dog extends this",
            "Expected a parent class name but found keyword 'this'",
            1,
            19,
        ),
        (
            "class Dog extends (Animal)",
            "Expected a parent class name but found '('",
            1,
            19,
        ),
        (
            "class Dog extends Animal: Cat",
            "Class 'Dog' already extends 'Animal'",
            1,
            27,
        ),
        (
            "class Dog extends Animal: Cat\n    pass",
            "Class 'Dog' already extends 'Animal'",
            1,
            27,
        ),
        (
            "class Dog(name) extends Animal: Animal",
            "Class 'Dog' already extends 'Animal'",
            1,
            33,
        ),
        (
            "class Dog extends Animal Cat",
            "Expected ':' but found identifier 'Cat'",
            1,
            26,
        ),
        (
            "class Dog extends Animal(x)",
            "Expected ':' but found '('",
            1,
            25,
        ),
        (
            "class Dog extends Animal, Cat",
            "Expected ':' but found ','",
            1,
            25,
        ),
        (
            "class Dog extends Animal {}",
            "Expected ':' but found '{'",
            1,
            26,
        ),
        (
            "class Dog extends Animal.Base",
            "Expected ':' but found '.'",
            1,
            25,
        ),
        (
            "class Dog extends Animal implements Pet",
            "Expected ':' but found identifier 'implements'",
            1,
            26,
        ),
        (
            "class Dog Extends Animal",
            "Expected ':' but found identifier 'Extends'",
            1,
            11,
        ),
        (
            "class Dog extend Animal",
            "Expected ':' but found identifier 'extend'",
            1,
            11,
        ),
        (
            "class Dog extends Animal\n    pass",
            "Expected ':' but found end of line",
            1,
            19,
        ),
        (
            "class Dog extends Animal\n\n    # still an indented body\n    pass",
            "Expected ':' but found end of line",
            1,
            19,
        ),
        (
            "class Dog extends Animal;\n    pass",
            "Expected ':' but found ';'",
            1,
            25,
        ),
        (
            "if ready:\n    class Dog extends Animal:\nnext = 1",
            "Unexpected the end of a block",
            3,
            1,
        ),
    ] {
        let error = Runtime::check(source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax, "{source}");
        assert_eq!(error.message(), message, "{source}");
        assert_eq!(
            error.position(),
            Some(Position::new(line, column)),
            "{source}"
        );
    }
}

#[test]
fn malformed_clauses_do_not_run_earlier_statements_or_declare_classes() {
    let mut runtime = Runtime::new();
    runtime.run("saved = 7").unwrap();

    for invalid in [
        "class Dog extends",
        "class Dog extends 5",
        "class Dog extends Animal: Cat",
        "class Dog extends Animal Cat",
        "class Dog extends Animal\n    pass",
    ] {
        let error = runtime.run(&format!("saved = 3\n{invalid}")).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax, "{invalid}");
        assert_eq!(runtime.run("saved").unwrap().to_string(), "7");
        assert_eq!(runtime.run("Class.length").unwrap().to_string(), "0");
    }
}

#[test]
fn extends_stays_an_ordinary_name_outside_the_clause() {
    for (source, expected) in [
        ("extends = 3\nextends + 1", "4.0"),
        ("record = {extends: 2}\nrecord.extends", "2.0"),
        ("extends = 2\nrecord = {extends}\nrecord.extends", "2.0"),
        ("def grow(extends) => extends + 1\ngrow(2)", "3.0"),
        ("def extends(value) => value * 2\nextends(4)", "8.0"),
        (
            "class Gate:\n    def extends(step) => step + 1\ngate = Gate()\ngate.extends(4)",
            "5.0",
        ),
        ("class extends\nextends()\nextends.length", "1.0"),
        (
            "class extends(size):\n    this.size = size\nitem = extends(3)\nitem.size",
            "3.0",
        ),
        (
            "class extends\nclass Probe: extends\n    pass\nProbe()\nProbe.length",
            "1.0",
        ),
        ("class Dog\nextends = 1\nextends", "1.0"),
        ("class Dog; extends = 1\nextends", "1.0"),
        ("class Dog(size)\nextends = 1\nextends", "1.0"),
        ("{\n    class Dog\n    extends = 1\n}\nextends", "1.0"),
        (
            "seen = []\nfor extends of \"ab\":\n    seen.push(extends)\nseen",
            "[\"a\",\"b\"]",
        ),
        (
            "class Item(n):\n    this.n = n\nItem(1)\nItem(2)\nseen = []\nfor extends of Item:\n    seen.push(extends.n)\nseen",
            "[1.0,2.0]",
        ),
        (
            "try:\n    throw 5\ncatch extends:\n    caught = extends\ncaught",
            "5.0",
        ),
        ("double = extends -> extends * 2\ndouble(4)", "8.0"),
        ("next = lambda extends: extends + 1\nnext(1)", "2.0"),
        ("class Dog:\nextends = 1\nDog.length", "0.0"),
    ] {
        let mut runtime = Runtime::new();
        let value = runtime
            .run(source)
            .unwrap_or_else(|error| panic!("{source}: {error}"));
        assert_eq!(
            runtime.serialize_json(&value).unwrap(),
            expected,
            "{source}"
        );
    }

    let error = Runtime::new().run("class Dog\nextends Animal").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "extends is not defined");

    let error = Runtime::check("ready = true\nif ready extends:\n    x = 1").unwrap_err();
    assert_eq!(
        error.message(),
        "Expected ':' but found identifier 'extends'"
    );
    assert_eq!(error.position(), Some(Position::new(2, 10)));
}

#[test]
fn extends_and_colon_classes_keep_identical_results_graphs_and_explanations() {
    let mut colon = Runtime::new();
    let mut extends = Runtime::new();
    for (colon_source, extends_source) in [
        (
            "class Animal(name, legs = 4):\n    this.name = name\n    this.legs = legs",
            "class Animal(name, legs = 4):\n    this.name = name\n    this.legs = legs",
        ),
        (
            "$Animal.walks = $Animal.legs > 0",
            "$Animal.walks = $Animal.legs > 0",
        ),
        (
            "class Dog(name, breed = \"mutt\"): Animal\n    this.name = name\n    this.breed = breed",
            "class Dog(name, breed = \"mutt\") extends Animal:\n    this.name = name\n    this.breed = breed",
        ),
        ("class Cat: Animal\n    pass", "class Cat extends Animal"),
        (
            "class Puppy: Dog\n    def init(name):\n        super(name, \"pup\")\n        this.young = true",
            "class Puppy extends Dog:\n    def init(name):\n        super(name, \"pup\")\n        this.young = true",
        ),
        ("rex = Dog(\"Rex\")", "rex = Dog(\"Rex\")"),
        ("tom = Cat(name=\"Tom\")", "tom = Cat(name=\"Tom\")"),
        ("bit = Puppy(\"Bit\")", "bit = Puppy(\"Bit\")"),
        (
            "[rex, tom, bit, Animal.length, Dog.length, Cat.length, Puppy.length, Class.length]",
            "[rex, tom, bit, Animal.length, Dog.length, Cat.length, Puppy.length, Class.length]",
        ),
        ("why tom.walks", "why tom.walks"),
        ("why bit.breed", "why bit.breed"),
        ("affects tom.legs", "affects tom.legs"),
        ("[tom.walks, rex.walks]", "[tom.walks, rex.walks]"),
        ("tom.legs = 0", "tom.legs = 0"),
        ("[tom.walks, rex.walks]", "[tom.walks, rex.walks]"),
        ("model |> why", "model |> why"),
        ("class Loop: Loop\n    pass", "class Loop extends Loop"),
        (
            "class Animal: Puppy\n    pass",
            "class Animal extends Puppy",
        ),
        ("Class.length", "Class.length"),
        (
            "[typeof $Dog, typeof Dog, typeof $Cat]",
            "[typeof $Dog, typeof Dog, typeof $Cat]",
        ),
    ] {
        match (colon.run(colon_source), extends.run(extends_source)) {
            (Ok(colon_value), Ok(extends_value)) => assert_eq!(
                colon.serialize_json(&colon_value).unwrap(),
                extends.serialize_json(&extends_value).unwrap(),
                "{extends_source}"
            ),
            (Err(colon_error), Err(extends_error)) => {
                assert_eq!(colon_error, extends_error, "{extends_source}");
                assert_eq!(colon_error.kind(), ErrorKind::Type, "{extends_source}");
                assert_eq!(colon_error.message(), "Circular Inheritance");
            }
            (colon_result, extends_result) => panic!(
                "different results for {extends_source}: {colon_result:?}, {extends_result:?}"
            ),
        }
        assert_eq!(
            graph_shape(&colon),
            graph_shape(&extends),
            "{extends_source}"
        );
    }
    extends
        .run("assert(bit.young, true)\nassert(bit.breed, \"pup\")\nassert(rex.breed, \"mutt\")\nassert(tom.name, \"Tom\")\nassert(tom.walks, false)\nassert(Animal.length, 0)")
        .unwrap();
    assert_eq!(extends.assertions_run(), 6);
    let failures = extends.take_assertions();
    assert!(failures.is_empty(), "{failures:?}");
}

#[test]
fn canonical_rendering_keeps_the_colon_spelling() {
    assert_eq!(
        generate_all(&parse("class Dog extends Animal").unwrap()),
        "class Dog"
    );
    assert_eq!(
        generate_all(&parse("class Dog(breed = \"mutt\") extends Animal").unwrap()),
        generate_all(&parse("class Dog(breed = \"mutt\"): Animal\n    pass").unwrap())
    );
    let rendered = generate_all(
        &parse("class Dog(name, ...tags) extends Animal:\n    this.tags = tags").unwrap(),
    );
    assert!(
        rendered.starts_with("class Dog(name,...tags): Animal\n"),
        "{rendered}"
    );
    assert!(!rendered.contains("extends"), "{rendered}");
}

#[test]
fn extends_clauses_obey_the_statement_nesting_limit() {
    let allowed = format!(
        "{}class Dog extends Animal{}",
        "{".repeat(63),
        "}".repeat(63)
    );
    assert!(Runtime::check(&allowed).is_ok());
    let allowed = format!("{}pass", "class Dog extends Animal: ".repeat(62));
    assert!(Runtime::check(&allowed).is_ok());

    for depth in [64, 4096] {
        let source = format!(
            "{}class Dog extends Animal{}",
            "{".repeat(depth),
            "}".repeat(depth)
        );
        let error = Runtime::check(&source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax);
        assert_eq!(error.message(), "Statements are nested too deeply");

        let source = format!("{}pass", "class Dog extends Animal: ".repeat(depth));
        let error = Runtime::check(&source).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Syntax);
        assert_eq!(error.message(), "Statements are nested too deeply");
    }
}

#[test]
fn cli_runs_extends_clauses_in_files() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("extends-clauses.nuc");
    let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .args(["--json", "--graph"])
        .arg(fixture)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(stdout.lines().next(), Some("2.0"));
    assert!(stdout.contains("$Dog [class]"), "{stdout}");
    assert!(stdout.contains("$Cat [class]"), "{stdout}");
    assert!(!stdout.contains("extends"), "{stdout}");
}

#[test]
fn repl_runs_bodyless_clauses_at_once_and_buffers_colon_clauses() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(
            b"class Animal(name):\n    this.name = name\n\nclass Dog extends Animal\n.json on\nrex = Dog(\"Rex\")\nrex.name\nclass Cat extends Animal:\n    def speak() => \"meow\"\n\ntom = Cat(\"Tom\")\ntom.speak()\n[Dog.length, Cat.length, Animal.length]\n.exit\n",
        )
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("json on."), "{stdout}");
    assert!(stdout.lines().any(|line| line == "\"Rex\""), "{stdout}");
    assert!(stdout.lines().any(|line| line == "\"meow\""), "{stdout}");
    assert!(
        stdout.lines().any(|line| line == "[1.0,1.0,0.0]"),
        "{stdout}"
    );
}
