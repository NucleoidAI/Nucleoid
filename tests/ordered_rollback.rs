use nucleoid::{ObjectId, Runtime, Value};
use serde_json::{Value as Json, json};

const KEYS: [&str; 4] = ["a", "b", "c", "d"];
const OBJECTS: [&str; 2] = ["o", "p"];
const PROGRAMS: u64 = 300;
const NESTING: usize = 3;

struct SplitMix(u64);

impl SplitMix {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut mixed = self.0;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        mixed ^ (mixed >> 31)
    }

    fn below(&mut self, bound: usize) -> usize {
        let bound = u64::try_from(bound).expect("bounds fit in u64");
        usize::try_from(self.next() % bound).expect("indices fit in usize")
    }

    fn one_in(&mut self, odds: usize) -> bool {
        self.below(odds) == 0
    }
}

type Entries = Vec<(String, String)>;

enum Item {
    Set {
        object: usize,
        key: &'static str,
        value: u64,
    },
    Delete {
        object: usize,
        key: &'static str,
    },
    Try {
        body: Vec<Item>,
        throws: bool,
    },
    Suppose {
        body: Vec<Item>,
    },
}

struct Generator {
    random: SplitMix,
    next_value: u64,
}

impl Generator {
    fn value(&mut self) -> u64 {
        self.next_value += 1;
        self.next_value
    }

    fn body(&mut self, depth: usize) -> Vec<Item> {
        let length = 1 + self.random.below(8);
        (0..length).map(|_| self.item(depth)).collect()
    }

    fn item(&mut self, depth: usize) -> Item {
        if depth < NESTING && self.random.one_in(4) {
            let body = self.body(depth + 1);
            return if self.random.one_in(3) {
                Item::Suppose { body }
            } else {
                Item::Try {
                    body,
                    throws: !self.random.one_in(3),
                }
            };
        }
        let object = self.random.below(OBJECTS.len());
        let key = KEYS[self.random.below(KEYS.len())];
        if self.random.one_in(2) {
            Item::Delete { object, key }
        } else {
            Item::Set {
                object,
                key,
                value: self.value(),
            }
        }
    }
}

fn render(items: &[Item], indent: usize, source: &mut String) {
    let pad = " ".repeat(indent);
    for item in items {
        match item {
            Item::Set { object, key, value } => {
                source.push_str(&format!("{pad}{}.{key} = {value}\n", OBJECTS[*object]));
            }
            Item::Delete { object, key } => {
                source.push_str(&format!("{pad}delete {}.{key}\n", OBJECTS[*object]));
            }
            Item::Try { body, throws } => {
                source.push_str(&format!("{pad}try:\n"));
                render(body, indent + 4, source);
                if *throws {
                    source.push_str(&format!("{pad}    throw \"undo\"\n"));
                }
                source.push_str(&format!("{pad}catch error:\n{pad}    pass\n"));
            }
            Item::Suppose { body } => {
                source.push_str(&format!("{pad}suppose:\n"));
                render(body, indent + 4, source);
            }
        }
    }
}

fn apply(items: &[Item], world: &mut [Entries]) {
    for item in items {
        match item {
            Item::Set { object, key, value } => {
                let entries = &mut world[*object];
                match entries.iter_mut().find(|(name, _)| name == key) {
                    Some((_, held)) => *held = value.to_string(),
                    None => entries.push(((*key).to_string(), value.to_string())),
                }
            }
            Item::Delete { object, key } => world[*object].retain(|(name, _)| name != key),
            Item::Try { body, throws } => {
                let before = world.to_vec();
                apply(body, world);
                if *throws {
                    world.clone_from_slice(&before);
                }
            }
            Item::Suppose { body } => {
                let before = world.to_vec();
                apply(body, world);
                world.clone_from_slice(&before);
            }
        }
    }
}

fn object_of(runtime: &Runtime, name: &str) -> ObjectId {
    match runtime.state.variable(name) {
        Some(Value::Object(id)) => id.clone(),
        other => panic!("{name} holds {other:?}"),
    }
}

fn held(runtime: &Runtime, name: &str) -> Entries {
    runtime
        .state
        .object(&object_of(runtime, name))
        .unwrap_or_else(|| panic!("{name} exists"))
        .properties
        .iter()
        .map(|(key, value)| (key.clone(), value.to_string()))
        .collect()
}

fn listed(runtime: &Runtime, name: &str) -> String {
    held(runtime, name)
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn looped(runtime: &mut Runtime, name: &str) -> String {
    let source = format!(
        "seen = []\nfor key, value of {name}: seen.push(key + \"=\" + value)\nseen.join(\" \")"
    );
    runtime
        .run(&source)
        .unwrap_or_else(|error| panic!("{source}\n{error}"))
        .to_string()
}

fn ran(runtime: &mut Runtime, source: &str) -> Value {
    runtime
        .run(source)
        .unwrap_or_else(|error| panic!("{source}\n{error}"))
}

fn json_of(runtime: &mut Runtime, source: &str) -> Json {
    let value = ran(runtime, source);
    serde_json::from_str(&runtime.serialize_json(&value).unwrap()).unwrap()
}

fn assert_restored(runtime: &mut Runtime, name: &str, expected: &str) {
    assert_eq!(listed(runtime, name), expected, "entries of {name}");
    assert_eq!(looped(runtime, name), expected, "entry loop over {name}");
}

#[test]
fn random_programs_restore_entry_order_at_every_rollback() {
    let mut failed_runs = 0_usize;
    let mut rolled_back_blocks = 0_usize;
    for seed in 0..PROGRAMS {
        let mut generator = Generator {
            random: SplitMix(seed),
            next_value: 100,
        };
        let mut remaining: Vec<&str> = KEYS.to_vec();
        let mut literal: Entries = Vec::new();
        while !remaining.is_empty() && !generator.random.one_in(4) {
            let key = remaining.remove(generator.random.below(remaining.len()));
            literal.push((key.to_string(), generator.value().to_string()));
        }
        let fields: Vec<String> = literal
            .iter()
            .map(|(key, value)| format!("{key}: {value}"))
            .collect();
        let setup = format!(
            "class Item(a, b):\n    this.a = a\n    this.b = b\no = {{{}}}\np = Item(1, 2)",
            fields.join(", ")
        );
        let mut world: Vec<Entries> = vec![
            literal,
            vec![
                ("id".to_string(), "p".to_string()),
                ("a".to_string(), "1".to_string()),
                ("b".to_string(), "2".to_string()),
            ],
        ];
        let mut runtime = Runtime::new();
        ran(&mut runtime, &setup);
        let mut history = setup.clone();

        for _ in 0..(3 + generator.random.below(6)) {
            let body = generator.body(0);
            rolled_back_blocks += count_rollbacks(&body);
            let mut source = String::new();
            render(&body, 0, &mut source);
            if generator.random.one_in(3) {
                source.push_str("throw \"failed\"\n");
                let error = runtime.run(&source).expect_err("the run fails");
                assert_eq!(
                    error.thrown_value(),
                    Some(&Value::String("failed".to_string())),
                    "seed {seed}: {error}\n{source}"
                );
                failed_runs += 1;
            } else {
                runtime
                    .run(&source)
                    .unwrap_or_else(|error| panic!("seed {seed}: {error}\n{source}"));
                apply(&body, &mut world);
            }
            history.push_str("\n---\n");
            history.push_str(&source);
            for (index, name) in OBJECTS.iter().enumerate() {
                assert_eq!(
                    held(&runtime, name),
                    world[index],
                    "seed {seed}: entries of {name} after\n{history}"
                );
            }
        }
        for (index, name) in OBJECTS.iter().enumerate() {
            let expected: Vec<String> = world[index]
                .iter()
                .map(|(key, value)| format!("{key}={value}"))
                .collect();
            assert_eq!(
                looped(&mut runtime, name),
                expected.join(" "),
                "seed {seed}: entry loop over {name} after\n{history}"
            );
        }
    }
    assert!(failed_runs > 300, "{failed_runs} failed runs");
    assert!(rolled_back_blocks > 1_000, "{rolled_back_blocks} rollbacks");
}

fn count_rollbacks(items: &[Item]) -> usize {
    items
        .iter()
        .map(|item| match item {
            Item::Try { body, throws } => usize::from(*throws) + count_rollbacks(body),
            Item::Suppose { body } => 1 + count_rollbacks(body),
            Item::Set { .. } | Item::Delete { .. } => 0,
        })
        .sum()
}

#[test]
fn rolled_back_additions_and_overwrites_leave_every_entry_in_place() {
    let mut runtime = Runtime::new();
    ran(
        &mut runtime,
        "o = {a: 1, b: 2, c: 3}\ntry:\n    o.d = 4\n    o.b = 9\n    o.e = 5\n    o.b = 10\n    throw \"undo\"\ncatch error:\n    pass",
    );
    assert_restored(&mut runtime, "o", "a=1 b=2 c=3");
    ran(
        &mut runtime,
        "try:\n    o.a = 7\n    o.c = 8\n    o.a = 9\n    throw \"undo\"\ncatch error:\n    pass",
    );
    assert_restored(&mut runtime, "o", "a=1 b=2 c=3");
}

#[test]
fn rolled_back_deletions_return_where_they_stood() {
    let mut runtime = Runtime::new();
    ran(
        &mut runtime,
        "o = {a: 1, b: 2, c: 3, d: 4}\ntry:\n    delete o.a\n    delete o.c\n    delete o.d\n    throw \"undo\"\ncatch error:\n    pass",
    );
    assert_restored(&mut runtime, "o", "a=1 b=2 c=3 d=4");
    ran(
        &mut runtime,
        "try:\n    delete o.b\n    o.b = 5\n    delete o.b\n    o.b = 6\n    delete o.a\n    o.a = 7\n    throw \"undo\"\ncatch error:\n    pass",
    );
    assert_restored(&mut runtime, "o", "a=1 b=2 c=3 d=4");
    ran(
        &mut runtime,
        "try:\n    delete o.d\n    delete o.c\n    delete o.b\n    delete o.a\n    o.a = 0\n    throw \"undo\"\ncatch error:\n    pass",
    );
    assert_restored(&mut runtime, "o", "a=1 b=2 c=3 d=4");
}

#[test]
fn an_overwrite_followed_by_deletions_restores_the_original_order() {
    let mut runtime = Runtime::new();
    ran(
        &mut runtime,
        "o = {x: 1, k: 2, y: 3}\ntry:\n    o.x = 5\n    delete o.k\n    delete o.x\n    throw \"undo\"\ncatch error:\n    pass",
    );
    assert_restored(&mut runtime, "o", "x=1 k=2 y=3");
    ran(
        &mut runtime,
        "try:\n    o.x = 9\n    delete o.k\n    delete o.x\n    o.x = 6\n    o.k = 7\n    throw \"undo\"\ncatch error:\n    pass",
    );
    assert_restored(&mut runtime, "o", "x=1 k=2 y=3");
}

#[test]
fn nested_savepoints_restore_the_order_their_own_work_changed() {
    let mut runtime = Runtime::new();
    ran(
        &mut runtime,
        "o = {a: 1, b: 2, c: 3, d: 4}\ntry:\n    delete o.b\n    try:\n        delete o.a\n        o.b = 9\n        throw \"inner\"\n    catch error:\n        pass\n    delete o.c\n    o.c = 8\ncatch error:\n    pass",
    );
    assert_restored(&mut runtime, "o", "a=1 d=4 c=8");

    let mut runtime = Runtime::new();
    ran(
        &mut runtime,
        "o = {a: 1, b: 2, c: 3, d: 4}\ntry:\n    delete o.b\n    try:\n        delete o.a\n        o.b = 9\n        throw \"inner\"\n    catch error:\n        pass\n    delete o.c\n    o.a = 7\n    suppose:\n        delete o.d\n        o.c = 5\n    throw \"outer\"\ncatch error:\n    pass",
    );
    assert_restored(&mut runtime, "o", "a=1 b=2 c=3 d=4");

    let mut runtime = Runtime::new();
    assert_eq!(
        json_of(
            &mut runtime,
            "o = {a: 1, b: 2, c: 3}\nseen = suppose:\n    delete o.a\n    inner = suppose:\n        delete o.b\n        o.a = 4\n        listing = []\n        for key, value of o: listing.push(key + value)\n        listing\n    after = []\n    for key, value of o: after.push(key + value)\n    [inner, after]\nlisting = []\nfor key, value of o: listing.push(key + value)\n[seen, listing]"
        ),
        json!([[["c3", "a4"], ["b2", "c3"]], ["a1", "b2", "c3"]])
    );
}

#[test]
fn failed_top_level_statements_restore_entry_order() {
    let mut runtime = Runtime::new();
    ran(&mut runtime, "o = {a: 1, b: 2, c: 3}");
    let error = runtime
        .run("delete o.a\no.a = 5\ndelete o.b\no.d = 4\nthrow \"failed\"")
        .expect_err("the run fails");
    assert_eq!(
        error.thrown_value(),
        Some(&Value::String("failed".to_string()))
    );
    assert_restored(&mut runtime, "o", "a=1 b=2 c=3");

    let error = runtime
        .run("o.b = 7\ndelete o.b\nmissing.x")
        .expect_err("the run fails");
    assert_eq!(error.message(), "missing is not defined");
    assert_restored(&mut runtime, "o", "a=1 b=2 c=3");

    let error = runtime
        .run("delete o.c\nfor key, value of [1]: pass")
        .expect_err("the run fails");
    assert_eq!(error.message(), "[1] is not an object");
    assert_restored(&mut runtime, "o", "a=1 b=2 c=3");
}

#[test]
fn instances_and_class_level_rules_restore_entry_order() {
    let mut runtime = Runtime::new();
    ran(
        &mut runtime,
        "class Room(w, d):\n    this.w = w\n    this.d = d\n$Room.area = $Room.w * $Room.d\nr = Room(2, 3)\ntry:\n    delete r.w\n    r.w = 5\n    delete r.id\n    s = Room(7, 1)\n    throw \"undo\"\ncatch error:\n    pass",
    );
    assert_restored(&mut runtime, "r", "id=r w=2 d=3 area=6");
    assert_eq!(ran(&mut runtime, "Room.length"), Value::Number(1.0));

    ran(
        &mut runtime,
        "try:\n    $Room.label = \"room\"\n    delete r.d\n    r.d = 4\n    throw \"undo\"\ncatch error:\n    pass",
    );
    assert_restored(&mut runtime, "r", "id=r w=2 d=3 area=6");
    assert_eq!(
        json_of(&mut runtime, "[Room(1, 1).label, r.label]"),
        json!([null, null])
    );

    let mut runtime = Runtime::new();
    ran(
        &mut runtime,
        "class Item(a, b):\n    this.a = a\n    this.b = b\np = Item(1, 2)\np.c = 3\ntry:\n    delete p.a\n    p = Item(5, 6)\n    delete p.b\n    p.a = 9\n    throw \"undo\"\ncatch error:\n    pass",
    );
    assert_restored(&mut runtime, "p", "id=p a=1 b=2 c=3");
    ran(
        &mut runtime,
        "try:\n    p = Item(5, 6)\n    p.c = 7\n    p.d = 8\n    delete p.d\n    p = Item(9, 10)\n    delete p.b\n    throw \"undo\"\ncatch error:\n    pass",
    );
    assert_restored(&mut runtime, "p", "id=p a=1 b=2 c=3");

    let mut runtime = Runtime::new();
    ran(
        &mut runtime,
        "class Wide(a, b):\n    this.a = a\n    this.b = b\nq = Wide(1, 2)\ntry:\n    q = Wide(3, 4)\n    delete q.id\n    q.id = \"z\"\n    delete q.id\n    q = Wide(5, 6)\n    throw \"undo\"\ncatch error:\n    pass",
    );
    assert_restored(&mut runtime, "q", "id=q a=1 b=2");
    ran(
        &mut runtime,
        "try:\n    q = Wide(3, 4)\n    delete q.a\n    delete q\n    throw \"undo\"\ncatch error:\n    pass",
    );
    assert_restored(&mut runtime, "q", "id=q a=1 b=2");
    assert_eq!(
        json_of(&mut runtime, "[Wide.length, q.b]"),
        json!([1.0, 2.0])
    );
}

#[test]
fn deleted_instances_come_back_with_their_entries_and_population_order() {
    let mut runtime = Runtime::new();
    ran(
        &mut runtime,
        "class Tag\nt = Tag()\nu = Tag()\nv = Tag()\ntry:\n    delete u\n    delete t\n    throw \"undo\"\ncatch error:\n    pass",
    );
    assert_restored(&mut runtime, "t", "id=t");
    assert_restored(&mut runtime, "u", "id=u");
    assert_eq!(
        json_of(&mut runtime, "Tag.map(tag -> tag.id)"),
        json!(["t", "u", "v"])
    );
}

#[test]
fn state_versions_survive_ordered_rollback() {
    let mut runtime = Runtime::new();
    ran(
        &mut runtime,
        "o = {a: 1, b: 2}\no.a = 3\ntry:\n    delete o.a\n    o.a = 4\n    delete o.b\n    throw \"undo\"\ncatch error:\n    pass",
    );
    assert_restored(&mut runtime, "o", "a=3 b=2");
    assert_eq!(
        json_of(
            &mut runtime,
            "[o.a@first, o.a@previous, o.a@last, o.b@first]"
        ),
        json!([1.0, 1.0, 3.0, 2.0])
    );
}

#[test]
fn suppose_values_keep_the_hypothetical_order_and_the_record_keeps_its_own() {
    let mut runtime = Runtime::new();
    assert_eq!(
        json_of(
            &mut runtime,
            "rec = {a: 1, b: 2}\ncopy = suppose:\n    delete rec.a\n    rec.c = 3\n    rec\nlog = []\nfor key, value of copy: log.push(key + value)\nlog"
        ),
        json!(["b2", "c3"])
    );
    assert_restored(&mut runtime, "rec", "a=1 b=2");
}

#[test]
fn graph_reflection_after_rollback_is_unchanged() {
    let mut runtime = Runtime::new();
    assert_eq!(
        json_of(
            &mut runtime,
            "class Rec\nrec = Rec()\nrec.a = 1\nrec.b = rec.a + 1\nrec.c = 3\ntotal = rec.b + rec.c\ntry:\n    delete rec.a\n    rec.a = 5\n    delete rec.c\n    throw \"undo\"\ncatch error:\n    pass\n[(model |> why).map(step => step.node + \":\" + step.holds), (why total).map(step => step.node)]"
        ),
        json!([
            [
                "$Rec:null",
                "rec:[object rec]",
                "rec.a:1",
                "rec.b:2",
                "rec.c:3",
                "total:5"
            ],
            ["total", "rec", "rec.b", "rec.c", "rec.a"]
        ])
    );

    let mut runtime = Runtime::new();
    assert_eq!(
        json_of(
            &mut runtime,
            "a = 1\nb = a + 1\nc = b * 2\ntry:\n    delete a\n    a = 7\n    throw \"undo\"\ncatch error:\n    pass\nd = c + 1\n[(model |> why).map(step => step.node + \"=\" + step.holds), (why d).map(step => step.node + \"<-\" + step.from.join(\",\"))]"
        ),
        json!([
            ["a=1", "b=2", "c=4", "d=5"],
            ["d<-c", "c<-b", "b<-a", "a<-"]
        ])
    );

    let mut runtime = Runtime::new();
    assert_eq!(
        json_of(
            &mut runtime,
            "class Room(w, d):\n    this.w = w\n    this.d = d\n$Room.area = $Room.w * $Room.d\nr = Room(2, 3)\ns = Room(4, 5)\ntry:\n    delete r.w\n    r.w = 9\n    delete $Room.area\n    throw \"undo\"\ncatch error:\n    pass\nplan = suppose:\n    delete s.d\n    s.d = 1\n    (model |> why).map(step => step.node)\n[plan, (model |> why).map(step => step.node), r.area, s.area]"
        ),
        json!([
            [
                "$Room", "r", "r.w", "r.d", "r.area", "s", "s.w", "s.d", "s.area"
            ],
            [
                "$Room", "r", "r.w", "r.d", "r.area", "s", "s.w", "s.d", "s.area", "plan"
            ],
            6.0,
            20.0
        ])
    );
}

#[test]
fn rolling_back_many_deletions_restores_large_objects() {
    let keys = 2_000;
    let declarations: Vec<String> = (0..keys)
        .map(|index| format!("big.k{index} = {index}"))
        .collect();
    let mut runtime = Runtime::new();
    ran(
        &mut runtime,
        &format!("big = Object()\n{}", declarations.join("\n")),
    );
    let before = held(&runtime, "big");
    assert_eq!(before.len(), keys);
    for order in [
        (0..keys).collect::<Vec<_>>(),
        (0..keys).rev().collect::<Vec<_>>(),
    ] {
        let deletions: Vec<String> = order
            .iter()
            .map(|index| format!("    delete big.k{index}"))
            .collect();
        ran(
            &mut runtime,
            &format!(
                "try:\n{}\n    big.k0 = -1\n    throw \"undo\"\ncatch error:\n    pass",
                deletions.join("\n")
            ),
        );
        assert_eq!(held(&runtime, "big"), before);
    }
}
