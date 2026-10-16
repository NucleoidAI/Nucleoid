//! Ordered reactive remapping beside exact accepted controls.
//! All ordinary regressions, including real CLI children, need no Git/session files.

mod common;
#[path = "support/named_arrow_headers.rs"]
mod named_arrow_headers;

use std::collections::HashMap;
use std::fs;
use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Stdio};

use nucleoid::graph::NodeKey;
use nucleoid::lang::ast::{Ast, Stmt, parse, parse_expression};
use nucleoid::scope::Scope;
use nucleoid::{ErrorKind, ObjectId, Position, Runtime, Value};
use serde::Deserialize;

fn checked(runtime: &mut Runtime, source: &str, expected: Option<&str>) -> Value {
    let count = common::assertions::protected_assertions(&parse(source).unwrap()).len();
    let before = runtime.assertions_run();
    let value = common::check_with_count(
        runtime,
        &common::Case {
            title: "Math.remap regression".to_string(),
            source: source.to_string(),
            expected: expected.map(str::to_string),
        },
        count,
    )
    .unwrap_or_else(|reason| panic!("{source}\n{reason}"));
    assert_eq!(runtime.assertions_run() - before, count);
    value
}

fn run(source: &str) -> Runtime {
    let mut runtime = Runtime::new();
    checked(&mut runtime, source, None);
    runtime
}

fn dependencies<'a>(runtime: &'a Runtime, name: &str) -> Vec<&'a str> {
    runtime
        .graph
        .retrieve(&NodeKey::variable(name))
        .unwrap()
        .dependencies
        .iter()
        .map(NodeKey::as_str)
        .collect()
}

fn assert_numeric_same(actual: &Value, expected: &Value) {
    let (Value::Number(actual), Value::Number(expected)) = (actual, expected) else {
        panic!("numeric controls required: {actual:?}, {expected:?}");
    };
    if expected.is_nan() {
        assert!(actual.is_nan(), "{actual} is not NaN");
    } else {
        assert_eq!(actual.to_bits(), expected.to_bits());
    }
}

fn assert_numeric_parity(left: &Runtime, right: &Runtime, name: &str) {
    assert_numeric_same(
        left.state.variable(name).unwrap(),
        right.state.variable(name).unwrap(),
    );
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SealedCase {
    document: String,
    title: String,
    source: String,
    expected: Option<String>,
}

// Exact complete sources from the caller's pre-executed, immutable accepted
// counterparts. JSON is embedded data, not a session or Git dependency.
const ACCEPTED: &str = r###"[
  {
    "document": "spec",
    "title": "Nucleoid keeps range remapping reactive",
    "source": "# Nucleoid keeps range remapping reactive\n\nstart = 2\nend = 10\ntargetStart = 10\ntargetEnd = 30\nsample = 4\nmapped = Math.lerp(targetStart, targetEnd, Math.inverseLerp(start, end, sample))\nunit = Math.inverseLerp(start, end, sample)\nassert(mapped, 15)\nassert(unit, 0.25)\nsample = 6\nassert(mapped, 20)\nassert(unit, 0.5)\nsample = 14\nassert(mapped, 40)\nassert(unit, 1.5)\nsample = -2\nassert(mapped, 0)\nassert(unit, -0.5)\nstart = 10\nend = 2\nsample = 8\nassert(mapped, 15)\nassert(unit, 0.25)\ntargetStart = 30\ntargetEnd = 10\nassert(mapped, 25)\nassert(unit, 0.25)\nsample = 6\nassert(mapped, 20)\nassert(unit, 0.5)\nmapped",
    "expected": "20"
  },
  {
    "document": "spec",
    "title": "Nucleoid preserves intrinsic conversion during range remapping",
    "source": "# Nucleoid preserves intrinsic conversion during range remapping\n\nstart = \" 2 \"\nend = \"10\"\ntargetStart = \"10\"\ntargetEnd = \"30\"\nsample = \"4\"\nmapped = Math.lerp(targetStart, targetEnd, Math.inverseLerp(start, end, sample))\nassert(mapped, 15)\nassert(start, \" 2 \")\nstart = false\nend = true\ntargetStart = false\ntargetEnd = true\nsample = false\nassert(mapped, 0)\nsample = true\nassert(mapped, 1)\nsample = [0.5]\nassert(mapped, 0.5)\nsample = []\nassert(mapped, 0)\nsample = \"not-a-number\"\nassert(Number.isNaN(mapped), true)\nsample = [1, 2]\nassert(Number.isNaN(mapped), true)\nstart = Date(0)\nend = Date(8)\ntargetStart = Date(10)\ntargetEnd = Date(30)\nsample = Date(2)\nassert(mapped, 15)\nsample = Date(6)\nassert(mapped, 25)\nstart = []\nend = [\"8\"]\ntargetStart = []\ntargetEnd = [\"8\"]\nsample = [\"2\"]\nassert(mapped, 2)\nsample = false\nassert(mapped, 0)",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid preserves remapping endpoints nonfinite inputs and omissions",
    "source": "# Nucleoid preserves remapping endpoints nonfinite inputs and omissions\n\nstart = 0\nend = 8\ntargetStart = 10\ntargetEnd = 30\nsample = 2\nmapped = Math.lerp(targetStart, targetEnd, Math.inverseLerp(start, end, sample))\nassert(mapped, 15)\nsample = start\nassert(mapped, 10)\nsample = end\nassert(mapped, 30)\nsample = 16\nassert(mapped, 50)\nsample = -8\nassert(mapped, -10)\nstart = 2\nend = 2\nsample = 2\nassert(Number.isNaN(mapped), true)\nsample = 3\nassert(mapped, Number.POSITIVE_INFINITY)\nsample = 1\nassert(mapped, Number.NEGATIVE_INFINITY)\nstart = 0\nend = Number.POSITIVE_INFINITY\nsample = 8\nassert(mapped, 10)\nsample = Number.POSITIVE_INFINITY\nassert(Number.isNaN(mapped), true)\nend = 8\nsample = Number.NEGATIVE_INFINITY\nassert(mapped, Number.NEGATIVE_INFINITY)\nstart = Number.NEGATIVE_INFINITY\nend = Number.POSITIVE_INFINITY\nsample = 0\nassert(Number.isNaN(mapped), true)\nstart = -Number.MAX_VALUE\nend = Number.MAX_VALUE\nsample = 0\nassert(mapped, 10)\nstart = 0\nend = 8\nsample = Number.NaN\nassert(Number.isNaN(mapped), true)\ntargetStart = 4\ntargetEnd = 4\nsample = 2\nassert(mapped, 4)\nsample = Number.POSITIVE_INFINITY\nassert(Number.isNaN(mapped), true)\ntargetStart = -Number.MAX_VALUE\ntargetEnd = Number.MAX_VALUE\nsample = 2\nassert(mapped, Number.POSITIVE_INFINITY)\ntargetStart = 0\ntargetEnd = 1\nassert(mapped, 0.25)\nempty = Math.lerp()\nassert(Number.isNaN(empty), true)\none = Math.lerp(0)\nassert(Number.isNaN(one), true)\ntwo = Math.lerp(0, 8)\nassert(Number.isNaN(two), true)\nthree = Math.lerp(0, 1, Math.inverseLerp(0, 8))\nassert(Number.isNaN(three), true)\nfour = Math.lerp(0, 1, Math.inverseLerp(0, 8))\nassert(Number.isNaN(four), true)\nliteral = Math.lerp(0, 1, Math.inverseLerp(false, true, null))\nassert(literal, 0)\nwithExtra = Math.lerp(10, 30, Math.inverseLerp(0, 8, 2), 99)\nassert(withExtra, 15)\nignored = Number.NaN\nwithIgnored = Math.lerp(10, 30, Math.inverseLerp(0, 8, 2), ignored)\nassert(withIgnored, 15)\nignored = \"not-a-number\"\nassert(withIgnored, 15)\nignored = null\nassert(withIgnored, null)\nignored = []\nassert(withIgnored, 15)",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid applies range remapping to live class rules",
    "source": "# Nucleoid applies range remapping to live class rules\n\nclass Gauge\n$Gauge.mapped = Math.lerp($Gauge.targetStart, $Gauge.targetEnd, Math.inverseLerp($Gauge.start, $Gauge.end, $Gauge.sample))\nfirst = Gauge()\nsecond = Gauge()\nfirst.start = 0\nfirst.end = 8\nfirst.targetStart = 10\nfirst.targetEnd = 30\nfirst.sample = 2\nsecond.start = 10\nsecond.end = 2\nsecond.targetStart = 30\nsecond.targetEnd = 10\nsecond.sample = 8\nassert(first.mapped, 15)\nassert(second.mapped, 25)\nfirst.sample = 12\nassert(first.mapped, 40)\nassert(second.mapped, 25)\nfirst.targetEnd = 50\nassert(first.mapped, 70)\nthird = Gauge()\nassert(third.mapped, null)\nthird.start = 0\nthird.end = 4\nthird.targetStart = 0\nthird.targetEnd = 100\nthird.sample = 1\nassert(third.mapped, 25)\ndelete first.sample\nassert(first.mapped, null)\nfirst.sample = -4\nassert(first.mapped, -10)\nassert(Gauge.length, 3)",
    "expected": null
  },
  {
    "document": "spec",
    "title": "Nucleoid preserves deferred range remapping and rollback",
    "source": "# Nucleoid preserves deferred range remapping and rollback\n\nclass Sensor\nsensor = Sensor()\nstart = sensor.lower\nend = sensor.upper\ntargetStart = sensor.outputLower\ntargetEnd = sensor.outputUpper\nsample = sensor.reading\nmapped = Math.lerp(targetStart, targetEnd, Math.inverseLerp(start, end, sample))\nwithDeferredExtra = Math.lerp(targetStart, targetEnd, Math.inverseLerp(start, end, sample), sensor.optional)\nassert(mapped, null)\nassert(withDeferredExtra, null)\nsensor.lower = 0\nsensor.upper = 8\nsensor.outputLower = 10\nsensor.outputUpper = 30\nsensor.reading = 2\nassert(mapped, 15)\nassert(withDeferredExtra, null)\nsensor.optional = 99\nassert(withDeferredExtra, 15)\ndelete sensor.reading\nassert(mapped, null)\nassert(withDeferredExtra, null)\nsensor.reading = 6\nassert(mapped, 25)\nassert(withDeferredExtra, 25)\ntry:\n    sensor.reading = 4\n    throw \"ROLLBACK\"\ncatch error:\n    assert(error, \"ROLLBACK\")\nassert(sensor.reading, 6)\nassert(mapped, 25)\nassert(withDeferredExtra, 25)\ntry:\n    sensor.reading = missing\ncatch error:\n    assert(error, ReferenceError(\"missing is not defined\"))\nassert(sensor.reading, 6)\ndelete sensor.optional\nassert(withDeferredExtra, null)\nassert(mapped, 25)\nsensor.optional = [1, 2]\nassert(withDeferredExtra, 25)\nsensor.reading = 2\nassert(mapped, 15)\nassert(withDeferredExtra, 15)\nmapped",
    "expected": "15"
  },
  {
    "document": "reference",
    "title": "Nucleoid preserves range remapping relationships",
    "source": "# Nucleoid preserves range remapping relationships\n\nstart = 2\nend = 10\ntargetStart = 10\ntargetEnd = 30\nsample = 4\nmapped = Math.lerp(targetStart, targetEnd, Math.inverseLerp(start, end, sample))\nassert(mapped, 15)\nsample = 6\nassert(mapped, 20)\ntargetStart = 0\nassert(mapped, 15)\ntargetEnd = 40\nassert(mapped, 20)\nsample = 18\nassert(mapped, 80)",
    "expected": null
  },
  {
    "document": "readme",
    "title": "Range remapping keeps live values unclamped",
    "source": "start = 0\nend = 8\ntargetStart = 0\ntargetEnd = 100\nsample = 2\nmapped = Math.lerp(targetStart, targetEnd, Math.inverseLerp(start, end, sample))\nassert(mapped, 25)\nsample = 12\nassert(mapped, 150)\nend = 16\nassert(mapped, 75)",
    "expected": null
  },
  {
    "document": "examples",
    "title": "Range remapping handles descending live source and target ranges",
    "source": "start = 10\nend = 2\ntargetStart = 30\ntargetEnd = 10\nsample = 8\nmapped = Math.lerp(targetStart, targetEnd, Math.inverseLerp(start, end, sample))\nassert(mapped, 25)\nsample = 4\nassert(mapped, 15)\nsample = 14\nassert(mapped, 40)\nstart = 18\nassert(mapped, 25)",
    "expected": null
  }
]"###;

const REWRITES: &[(&str, &str)] = &[
    (
        "mapped = Math.lerp(targetStart, targetEnd, Math.inverseLerp(start, end, sample))",
        "mapped = Math.remap(start, end, targetStart, targetEnd, sample)",
    ),
    (
        "$Gauge.mapped = Math.lerp($Gauge.targetStart, $Gauge.targetEnd, Math.inverseLerp($Gauge.start, $Gauge.end, $Gauge.sample))",
        "$Gauge.mapped = Math.remap($Gauge.start, $Gauge.end, $Gauge.targetStart, $Gauge.targetEnd, $Gauge.sample)",
    ),
    ("empty = Math.lerp()", "empty = Math.remap()"),
    ("one = Math.lerp(0)", "one = Math.remap(0)"),
    ("two = Math.lerp(0, 8)", "two = Math.remap(0, 8)"),
    (
        "three = Math.lerp(0, 1, Math.inverseLerp(0, 8))",
        "three = Math.remap(0, 8, 0)",
    ),
    (
        "four = Math.lerp(0, 1, Math.inverseLerp(0, 8))",
        "four = Math.remap(0, 8, 0, 1)",
    ),
    (
        "literal = Math.lerp(0, 1, Math.inverseLerp(false, true, null))",
        "literal = Math.remap(false, true, 0, 1, null)",
    ),
    (
        "withExtra = Math.lerp(10, 30, Math.inverseLerp(0, 8, 2), 99)",
        "withExtra = Math.remap(0, 8, 10, 30, 2, 99)",
    ),
    (
        "withIgnored = Math.lerp(10, 30, Math.inverseLerp(0, 8, 2), ignored)",
        "withIgnored = Math.remap(0, 8, 10, 30, 2, ignored)",
    ),
    (
        "withDeferredExtra = Math.lerp(targetStart, targetEnd, Math.inverseLerp(start, end, sample), sensor.optional)",
        "withDeferredExtra = Math.remap(start, end, targetStart, targetEnd, sample, sensor.optional)",
    ),
];

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

// Length-delimited FNV-1a seals are regression fingerprints, not token counts
// or security digests. A separate read-only audit compares the actual bytes
// and complete ASTs directly against the immutable accepted snapshot.
fn seal_field(hash: &mut u64, bytes: &[u8]) {
    for byte in (bytes.len() as u64).to_le_bytes().iter().chain(bytes) {
        *hash = (*hash ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3);
    }
}

fn history_signatures(cases: &[common::Case]) -> [u64; 3] {
    let mut hashes = [0xcbf2_9ce4_8422_2325; 3];
    for case in cases {
        // NUC40 changes only qualified block-header commas. Reconstruct that
        // accepted spelling, preserving the original raw-source fingerprints,
        // boundary whitespace and every body byte rather than refreshing seals.
        let source = named_arrow_headers::legacy_block_headers(&case.source);
        for text in [&case.title, &source, &format!("{:?}", case.expected)] {
            seal_field(&mut hashes[0], text.as_bytes());
        }
        let ast = parse(&case.source).unwrap();
        let restored_ast = parse(&source).unwrap();
        assert_eq!(ast, restored_ast, "{}", case.title);
        assert_eq!(
            nucleoid::lang::ast::generator::generate_all(&ast),
            nucleoid::lang::ast::generator::generate_all(&restored_ast)
        );
        seal_field(&mut hashes[1], format!("{ast:?}").as_bytes());
        for assertion in common::assertions::protected_assertions(&ast) {
            seal_field(&mut hashes[2], assertion.as_bytes());
        }
    }
    hashes
}

#[test]
fn frozen_history_keeps_boundary_whitespace_full_asts_returns_and_actual_prefix_coverage() {
    // Pinned from a direct accepted-snapshot/candidate equality audit at
    // e75194047a2bf0912c28c7b7f161997d6b7164a7. Future tails remain extendable.
    let expected = [
        (
            368,
            373,
            [
                756_201_041_972_111_351,
                7_163_576_774_319_617_178,
                10_531_902_229_809_350_683,
            ],
        ),
        (
            74,
            75,
            [
                1_442_963_248_971_915_969,
                17_012_481_147_454_982_509,
                12_386_672_469_646_665_352,
            ],
        ),
        (
            25,
            26,
            [
                2_721_320_878_773_376_749,
                1_640_508_625_233_297_920,
                17_484_169_508_249_249_642,
            ],
        ),
        (
            80,
            81,
            [
                7_781_374_582_744_196_263,
                6_211_037_734_568_771_901,
                9_663_638_398_408_713_342,
            ],
        ),
    ];
    let mut totals = (0, 0, 0, 0);
    for ((document, cases), (history, prefix, signatures)) in documents().into_iter().zip(expected)
    {
        assert!(
            cases.len() >= prefix,
            "{document}: protected prefix removed"
        );
        assert_eq!(
            history_signatures(&cases[..history]),
            signatures,
            "{document}"
        );
        totals.0 += history;
        totals.1 += prefix;
        for case in &cases[..prefix] {
            let assertions =
                common::assertions::protected_assertions(&parse(&case.source).unwrap());
            let mut runtime = Runtime::new();
            common::check_with_count(&mut runtime, case, assertions.len())
                .unwrap_or_else(|reason| panic!("{document}/{}: {reason}", case.title));
            assert_eq!(runtime.assertions_run(), assertions.len(), "{}", case.title);
            totals.2 += runtime.assertions_run();
            totals.3 += usize::from(case.expected.is_some());
        }
    }
    assert_eq!(totals, (547, 555, 2480, 39));
}

#[test]
fn all_synthesized_and_arc_training_bytes_stay_frozen_without_session_fixtures() {
    let training: &[(&str, &[u8], usize, u64)] = &[
        (
            "arc",
            include_bytes!("../dataset/arc.jsonl"),
            377_621,
            17_746_989_155_762_170_233,
        ),
        (
            "01",
            include_bytes!("../dataset/nucleoid.synth.01.jsonl"),
            24_688,
            13_987_384_607_468_208_805,
        ),
        (
            "02",
            include_bytes!("../dataset/nucleoid.synth.02.jsonl"),
            27_159,
            2_298_257_892_763_360_176,
        ),
        (
            "03",
            include_bytes!("../dataset/nucleoid.synth.03.jsonl"),
            17_720,
            13_699_682_580_045_772_616,
        ),
        (
            "04",
            include_bytes!("../dataset/nucleoid.synth.04.jsonl"),
            24_742,
            3_279_035_365_058_038_904,
        ),
        (
            "05",
            include_bytes!("../dataset/nucleoid.synth.05.jsonl"),
            23_011,
            2_306_183_564_990_993_488,
        ),
        (
            "06",
            include_bytes!("../dataset/nucleoid.synth.06.jsonl"),
            23_492,
            17_297_691_789_024_304_489,
        ),
        (
            "07",
            include_bytes!("../dataset/nucleoid.synth.07.jsonl"),
            24_239,
            16_475_628_791_941_205_335,
        ),
        (
            "08",
            include_bytes!("../dataset/nucleoid.synth.08.jsonl"),
            23_901,
            13_753_751_003_716_780_770,
        ),
        (
            "09",
            include_bytes!("../dataset/nucleoid.synth.09.jsonl"),
            24_334,
            5_050_742_324_820_675_435,
        ),
        (
            "10",
            include_bytes!("../dataset/nucleoid.synth.10.jsonl"),
            23_375,
            14_984_257_241_296_558_110,
        ),
        (
            "11",
            include_bytes!("../dataset/nucleoid.synth.11.jsonl"),
            24_071,
            11_080_934_392_415_303_877,
        ),
        (
            "12",
            include_bytes!("../dataset/nucleoid.synth.12.jsonl"),
            23_972,
            17_203_317_493_619_065_240,
        ),
        (
            "13",
            include_bytes!("../dataset/nucleoid.synth.13.jsonl"),
            23_853,
            16_046_214_043_181_485_617,
        ),
        (
            "14",
            include_bytes!("../dataset/nucleoid.synth.14.jsonl"),
            23_959,
            8_663_428_887_139_518_929,
        ),
        (
            "15",
            include_bytes!("../dataset/nucleoid.synth.15.jsonl"),
            24_198,
            9_048_832_254_363_867_553,
        ),
        (
            "16",
            include_bytes!("../dataset/nucleoid.synth.16.jsonl"),
            23_759,
            12_038_231_736_532_964_140,
        ),
        (
            "17",
            include_bytes!("../dataset/nucleoid.synth.17.jsonl"),
            23_910,
            3_136_279_688_781_069_991,
        ),
        (
            "18",
            include_bytes!("../dataset/nucleoid.synth.18.jsonl"),
            23_801,
            3_045_623_875_884_870_937,
        ),
        (
            "19",
            include_bytes!("../dataset/nucleoid.synth.19.jsonl"),
            25_038,
            9_846_606_845_834_782_896,
        ),
        (
            "20",
            include_bytes!("../dataset/nucleoid.synth.20.jsonl"),
            23_276,
            18_043_811_992_111_428_142,
        ),
        (
            "21",
            include_bytes!("../dataset/nucleoid.synth.21.jsonl"),
            60_090,
            1_345_645_721_628_158_366,
        ),
    ];
    let mut records = (0, 0);
    for &(name, bytes, length, expected) in training {
        // Only the explicit top-level code field is restored; non-code bytes
        // and all old expected lengths/FNV seals remain protected unchanged.
        let restored = named_arrow_headers::legacy_block_training_bytes(bytes);
        let bytes = restored.as_slice();
        assert_eq!(bytes.len(), length, "{name}");
        let mut signature = 0xcbf2_9ce4_8422_2325;
        seal_field(&mut signature, bytes);
        assert_eq!(signature, expected, "{name}");
        let count = bytes
            .split(|byte| *byte == b'\n')
            .filter(|line| !line.is_empty())
            .count();
        if name == "arc" {
            records.0 += count;
        } else {
            records.1 += count;
        }
    }
    assert_eq!(records, (109, 1081));
}

fn rewrite_ast_rhs(statements: &mut [Stmt]) {
    for &(accepted, candidate) in REWRITES {
        let accepted = parse(accepted).unwrap();
        let candidate = parse(candidate).unwrap();
        let [
            Stmt::Assign {
                target: old_target,
                value: old_value,
            },
        ] = accepted.as_slice()
        else {
            panic!("a designated assignment is required");
        };
        let [
            Stmt::Assign {
                target: new_target,
                value: new_value,
            },
        ] = candidate.as_slice()
        else {
            panic!("a designated assignment is required");
        };
        assert_eq!(old_target, new_target);
        for statement in &mut *statements {
            if let Stmt::Assign { target, value } = statement {
                if target == old_target && value == old_value {
                    *value = new_value.clone();
                }
            }
        }
    }
}

#[test]
fn eight_exact_sealed_counterparts_keep_only_seventeen_rhs_changes_and_all_actual_coverage() {
    let sealed: Vec<SealedCase> = serde_json::from_str(ACCEPTED).unwrap();
    let documents = documents();
    assert_eq!(sealed.len(), 8);
    let mut previous = HashMap::new();
    let mut totals = (0, 0, 0);
    for (index, sealed) in sealed.iter().enumerate() {
        let cases = &documents
            .iter()
            .find(|(name, _)| *name == sealed.document)
            .unwrap()
            .1;
        let matching = cases
            .iter()
            .enumerate()
            .filter(|(_, case)| case.title == sealed.title)
            .collect::<Vec<_>>();
        assert_eq!(matching.len(), 1, "{}", sealed.title);
        let (position, case) = matching[0];
        if let Some(before) = previous.insert(&sealed.document, position) {
            assert!(before < position, "{}: relative order", sealed.title);
        }
        assert_eq!(case.expected, sealed.expected);
        let mut source = sealed.source.clone();
        let mut changes = 0;
        for &(accepted, intrinsic) in REWRITES {
            changes += source.matches(accepted).count();
            source = source.replace(accepted, intrinsic);
        }
        assert_eq!(changes, [1, 1, 9, 1, 2, 1, 1, 1][index]);
        assert_eq!(case.source.trim(), source, "{}", sealed.title);
        assert_eq!(changes, source.matches("Math.remap(").count());
        let actual_ast = parse(case.source.trim()).unwrap();
        let mut accepted_ast = parse(&sealed.source).unwrap();
        assert_ne!(actual_ast, accepted_ast);
        let protected = common::assertions::protected_assertions(&accepted_ast);
        assert_eq!(protected.len(), [14, 12, 29, 10, 20, 5, 3, 4][index]);
        assert_eq!(
            common::assertions::protected_assertions(&actual_ast),
            protected
        );
        rewrite_ast_rhs(&mut accepted_ast);
        assert_eq!(actual_ast, accepted_ast, "full non-RHS/typed metadata");
        let control = common::Case {
            title: sealed.title.clone(),
            source: sealed.source.clone(),
            expected: sealed.expected.clone(),
        };
        let mut runtimes = [Runtime::new(), Runtime::new()];
        for (runtime, case) in runtimes.iter_mut().zip([case, &control]) {
            common::check_with_count(runtime, case, protected.len())
                .unwrap_or_else(|reason| panic!("{}: {reason}", sealed.title));
            assert_eq!(runtime.assertions_run(), protected.len());
        }
        if sealed.title.contains("class rules") {
            for name in ["first", "second", "third"] {
                let id = ObjectId::named(name);
                assert_numeric_same(
                    runtimes[0].state.property(&id, "mapped").unwrap(),
                    runtimes[1].state.property(&id, "mapped").unwrap(),
                );
                for runtime in &runtimes {
                    for input in ["start", "end", "targetStart", "targetEnd", "sample"] {
                        assert!(runtime.graph.reaches(
                            &NodeKey::property(&id, input),
                            &NodeKey::property(&id, "mapped"),
                        ));
                    }
                }
            }
        } else {
            assert_numeric_parity(&runtimes[0], &runtimes[1], "mapped");
            for runtime in &runtimes {
                for input in ["start", "end", "targetStart", "targetEnd", "sample"] {
                    assert!(
                        runtime
                            .graph
                            .reaches(&NodeKey::variable(input), &NodeKey::variable("mapped"),)
                    );
                }
            }
            if sealed.expected.as_deref() == Some("20") {
                assert_eq!(
                    source
                        .matches("unit = Math.inverseLerp(start, end, sample)")
                        .count(),
                    1
                );
                assert_numeric_parity(&runtimes[0], &runtimes[1], "unit");
            }
            if sealed.expected.as_deref() == Some("15") {
                assert_numeric_parity(&runtimes[0], &runtimes[1], "withDeferredExtra");
                for runtime in &runtimes {
                    assert!(runtime.graph.reaches(
                        &NodeKey::property(&ObjectId::named("sensor"), "optional"),
                        &NodeKey::variable("withDeferredExtra"),
                    ));
                }
            }
        }
        totals.0 += protected.len();
        totals.1 += usize::from(sealed.expected.is_some());
        totals.2 += changes;
    }
    assert_eq!(totals, (97, 2, 17));
}

#[test]
fn live_edges_and_repeated_nan_match_independent_accepted_composition_by_bits() {
    let template = "start = 0\nend = 8\ntargetStart = 10\ntargetEnd = 30\nsample = 2\n\
                    mapped = __MAPPED__\nunit = Math.inverseLerp(start, end, sample)\n\
                    reciprocal = 1 / mapped";
    let mut runtimes = [
        "Math.remap(start, end, targetStart, targetEnd, sample)",
        "Math.lerp(targetStart, targetEnd, Math.inverseLerp(start, end, sample))",
    ]
    .map(|expression| run(&template.replace("__MAPPED__", expression)));
    for source in [
        "assert(mapped, 15)",
        "sample = start\nassert(mapped, 10)",
        "sample = end\nassert(mapped, 30)",
        "sample = 16\nassert(mapped, 50)",
        "sample = -8\nassert(mapped, -10)",
        "start = 10\nend = 2\nsample = 8\nassert(mapped, 15)",
        "targetStart = 30\ntargetEnd = 10\nassert(mapped, 25)",
        "start = 2\nend = 2\nsample = 2\nassert(Number.isNaN(mapped), true)",
        "targetStart = 10\ntargetEnd = 30\nsample = 3\nassert(mapped, Number.POSITIVE_INFINITY)",
        "sample = 1\nassert(mapped, Number.NEGATIVE_INFINITY)",
        "start = -Number.MAX_VALUE\nend = Number.MAX_VALUE\nsample = 0\nassert(mapped, 10)",
        "sample = Number.MAX_VALUE\nassert(Number.isNaN(mapped), true)",
        "start = 0\nend = 8\ntargetStart = 4\ntargetEnd = 4\nsample = Number.POSITIVE_INFINITY\nassert(Number.isNaN(mapped), true)",
        "targetStart = -Number.MAX_VALUE\ntargetEnd = Number.MAX_VALUE\nsample = 2\nassert(mapped, Number.POSITIVE_INFINITY)",
        "sample = 0\nassert(Number.isNaN(mapped), true)",
        "targetStart = -0\ntargetEnd = -0\nstart = 0\nend = 1\nsample = -0",
        "targetStart = 0\ntargetEnd = -0",
        "start = -0\nsample = 0",
        "targetStart = 0\ntargetEnd = 1e-323\nsample = 0.5\nassert(mapped, 5e-324)",
        "end = Number.MAX_VALUE\nsample = 5e-324\nassert(mapped, 0)",
        "start = 10000000000000000\nend = start + 2\ntargetStart = -start\ntargetEnd = start\nsample = start",
        "start = 0\nend = 1\ntargetStart = -1\ntargetEnd = Number.EPSILON\nsample = 1 - Number.EPSILON",
        "start = \" 2 \"\nend = \"10\"\ntargetStart = \"10\"\ntargetEnd = \"30\"\nsample = \"4\"\nassert(mapped, 15)",
        "start = false\nend = true\ntargetStart = false\ntargetEnd = true\nsample = [0.5]\nassert(mapped, 0.5)",
        "start = Date(0)\nend = Date(8)\ntargetStart = Date(10)\ntargetEnd = Date(30)\nsample = Date(2)\nassert(mapped, 15)",
        "start = []\nend = [\"8\"]\ntargetStart = []\ntargetEnd = [\"8\"]\nsample = [\"2\"]\nassert(mapped, 2)",
        "sample = [1, 2]\nassert(Number.isNaN(mapped), true)",
        "sample = \"invalid\"\nassert(Number.isNaN(mapped), true)",
        "sample = Number.NaN\nassert(Number.isNaN(mapped), true)",
        "sample = Number.NaN\nassert(Number.isNaN(mapped), true)",
        "start = 0\nend = 8\ntargetStart = 10\ntargetEnd = 30\nsample = 2\nassert(mapped, 15)",
    ] {
        for runtime in &mut runtimes {
            checked(runtime, source, None);
        }
        for name in ["mapped", "unit", "reciprocal"] {
            assert_numeric_parity(&runtimes[0], &runtimes[1], name);
        }
    }
}

#[test]
fn literal_value_kinds_in_all_five_slots_omissions_and_extras_keep_intrinsic_conversion() {
    for expression in [
        "null",
        "\" 2 \"",
        "\"\"",
        "true",
        "false",
        "[]",
        "[\"2\"]",
        "[[\"2\"]]",
        "Date(2)",
        "[1, 2]",
        "\"invalid\"",
        "{}",
        "/probe/",
        "value => value",
        "Probe",
        "Probe()",
    ] {
        for position in 0..5 {
            let mut arguments = ["0", "8", "10", "30", "2"];
            arguments[position] = expression;
            let [start, end, target_start, target_end, value] = arguments;
            let mut runtime = run("class Probe");
            let actual = checked(
                &mut runtime,
                &format!("Math.remap({})", arguments.join(", ")),
                None,
            );
            let expected = checked(
                &mut runtime,
                &format!(
                    "Math.lerp(({target_start}), ({target_end}), Math.inverseLerp(({start}), ({end}), ({value})))"
                ),
                None,
            );
            assert_numeric_same(&actual, &expected);
        }
    }
    run(r#"
assert(Number.isNaN(Math.remap()), true)
assert(Number.isNaN(Math.remap(0)), true)
assert(Number.isNaN(Math.remap(0, 8)), true)
assert(Number.isNaN(Math.remap(0, 8, 0)), true)
assert(Number.isNaN(Math.remap(0, 8, 0, 1)), true)
assert(Math.remap(null, 8, 10, 30, 2), 15)
assert(Math.remap(2, null, 10, 30, 1), 20)
assert(Math.remap(0, 8, null, 32, 2), 8)
assert(Math.remap(0, 8, 16, null, 2), 12)
assert(Math.remap(false, true, 0, 1, null), 0)
assert(Math.remap(0, 8, 10, 30, 2, Number.NaN, "invalid", {}, null), 15)
assert(Math.remap(0, 8, 10, 30, 2, [1, 2], Date(99), /probe/, value => value), 15)
class Probe
assert(Math.remap(0, 8, 10, 30, 2, Probe, Probe()), 15)
"#);
}

#[test]
fn every_expression_runs_once_in_new_call_order_even_after_undefined() {
    run(r#"
class Log
log = Log()
log.order = 0
mapped = Math.remap(
    log.order = log.order.value * 10 + 1,
    log.order = log.order.value * 10 + 2,
    log.order = log.order.value * 10 + 3,
    log.order = log.order.value * 10 + 4,
    log.order = log.order.value * 10 + 5,
    log.order = log.order.value * 10 + 6,
    log.order = log.order.value * 10 + 7
)
assert(log.order, 1234567)
assert(mapped, Math.lerp(123, 1234, Math.inverseLerp(1, 12, 12345)))
invalid = Math.remap(Number.NaN, 8, 10, 30, 2, log.order = 8)
assert(Number.isNaN(invalid), true)
assert(log.order, 8)
deferred = Math.remap(log.input, 8, 10, 30, 2, log.order = 7)
assert(deferred, null)
assert(log.order, 7)
log.input = 0
assert(deferred, 15)
assert(log.order, 7)
log.order = 0
values = [2, 4, 6].map(value => Math.remap(0, 8, 10, 30, value, log.order = log.order.value * 10 + 1))
assert(values, [15, 20, 25])
assert(log.order, 111)
"#);
    for position in 0..7 {
        let mut arguments = (1..=7)
            .map(|digit| format!("log.order = log.order.value * 10 + {digit}"))
            .collect::<Vec<_>>();
        arguments[position] = "log.absent".to_string();
        let expected = (1..=7)
            .filter(|digit| *digit != position + 1)
            .fold(0, |order, digit| order * 10 + digit);
        run(&format!(
            "class Log\nlog = Log()\nlog.order = 0\nmapped = Math.remap({})\n\
             assert(mapped, null)\nassert(log.order, {expected})",
            arguments.join(", ")
        ));
    }
}

#[test]
fn undefined_in_each_slot_and_extras_keeps_dependencies_why_deletion_and_recovery() {
    for (arguments, defined) in [
        ("probe.input, 8, 10, 30, 2", "0"),
        ("0, probe.input, 10, 30, 2", "8"),
        ("0, 8, probe.input, 30, 2", "10"),
        ("0, 8, 10, probe.input, 2", "30"),
        ("0, 8, 10, 30, probe.input", "2"),
        ("0, 8, 10, 30, 2, probe.input", "{}"),
        ("0, 8, 10, 30, 2, 99, probe.input", "Number.NaN"),
        ("0, 8, 10, 30, 2, 99, \"invalid\", probe.input", "[1, 2]"),
    ] {
        let runtime = run(&format!(
            "class Probe\nprobe = Probe()\nmapped = Math.remap({arguments})\n\
             assert(mapped, null)\nprobe.input = {defined}\nassert(mapped, 15)\n\
             delete probe.input\nassert(mapped, null)\nprobe.input = {defined}\nassert(mapped, 15)"
        ));
        assert!(
            runtime
                .graph
                .retrieve(&NodeKey::variable("mapped"))
                .unwrap()
                .dependencies
                .contains(&NodeKey::property(&ObjectId::named("probe"), "input"))
        );
    }
    let runtime = run(r#"
class Probe
probe = Probe()
start = 0
end = 8
targetStart = 10
targetEnd = 30
sample = 2
extra = 99
mapped = Math.remap(start, end, targetStart, targetEnd, sample, extra)
trace = why mapped
assert(trace[0].from, ["start", "end", "targetStart", "targetEnd", "sample", "extra"])
assert(trace[0].holds, 15)
sample = 12
assert(trace[0].holds, 40)
end = 16
assert(trace[0].holds, 25)
extra = Number.NaN
assert(mapped, 25)
assert(Math.unknown(probe.absent), null)
assert(Math.random(probe.absent), null)
"#);
    assert_eq!(
        dependencies(&runtime, "mapped"),
        [
            "start",
            "end",
            "targetStart",
            "targetEnd",
            "sample",
            "extra"
        ]
    );
    for input in [
        "start",
        "end",
        "targetStart",
        "targetEnd",
        "sample",
        "extra",
    ] {
        assert!(
            runtime
                .graph
                .reaches(&NodeKey::variable(input), &NodeKey::variable("mapped"))
        );
    }
    let runtime = run(r#"
class Probe
probe = Probe()
end = 8
targetStart = 10
targetEnd = 30
sample = 2
extra = 99
deferred = Math.remap(probe.absent, end, targetStart, targetEnd, sample, extra)
assert(deferred, null)
sample = 6
assert(deferred, null)
probe.absent = 0
assert(deferred, 25)
"#);
    for input in ["end", "targetStart", "targetEnd", "sample", "extra"] {
        assert!(
            runtime
                .graph
                .reaches(&NodeKey::variable(input), &NodeKey::variable("deferred"),)
        );
    }
}

#[test]
fn strict_failures_at_all_positions_after_undefined_preserve_errors_positions_and_rollback() {
    let mut runtime = run(r#"
class Log
log = Log()
log.order = 111
mapped = Math.remap(0, 8, 10, 30, 2)
try:
    mapped = Math.remap(log.order = 9, 8, 10, 30, 2, missing, log.order = 99)
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(log.order, 111)
assert(mapped, 15)
try:
    Math.remap(log.absent, 8, 10, 30, 2, missing)
catch error:
    assert(error, ReferenceError("missing is not defined"))
try:
    Math.remap(log.absent, 8, 10, 30, 2, Math.random())
catch error:
    assert(error, TypeError("Math.random is not supported"))
"#);
    for position in 0..8 {
        let mut arguments = ["0", "8", "10", "30", "2", "99", "99", "99"];
        arguments[position] = "missing";
        let keys = runtime.graph.keys().cloned().collect::<Vec<_>>();
        let error = runtime
            .run(&format!("Math.remap({})", arguments.join(", ")))
            .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Reference);
        assert_eq!(error.message(), "missing is not defined");
        assert_eq!(error.position(), Some(Position::new(1, 1)));
        assert_eq!(runtime.graph.keys().cloned().collect::<Vec<_>>(), keys);
    }
    for (source, kind, message) in [
        (
            "Math.remap(log.absent, 8, 10, 30, 2, missing)",
            ErrorKind::Reference,
            "missing is not defined",
        ),
        (
            "Math.remap(log.absent, 8, 10, 30, 2, Math.random())",
            ErrorKind::Type,
            "Math.random is not supported",
        ),
        (
            "Math.unknown()",
            ErrorKind::Type,
            "Math.unknown is not a function",
        ),
    ] {
        let error = runtime.run(source).unwrap_err();
        assert_eq!(error.kind(), kind);
        assert_eq!(error.message(), message);
        assert_eq!(error.position(), Some(Position::new(1, 1)));
    }
    let error = runtime
        .run("log.order = 9\nMath.remap(0, 8, 10, 30, 2, missing)")
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "missing is not defined");
    assert_eq!(error.position(), Some(Position::new(2, 1)));
    checked(
        &mut runtime,
        "assert(log.order, 111)\nassert(mapped, 15)",
        None,
    );
    assert!(!runtime.graph.contains(&NodeKey::variable("missing")));
}

#[test]
fn intrinsic_literal_null_does_not_override_live_or_parameter_null_deferral() {
    let mut runtime = run(r#"
start = 0
end = 8
targetStart = 10
targetEnd = 30
sample = 2
extra = 99
mapped = Math.remap(start, end, targetStart, targetEnd, sample, extra)
def converted(start: Numeric, end: Numeric, targetStart: Numeric, targetEnd: Numeric, sample: Numeric) => Math.remap(start, end, targetStart, targetEnd, sample)
omitted = converted()
fewer = converted(0, 8, 10, 30)
first = converted(null, 8, 10, 30, 2)
second = converted(0, null, 10, 30, 2)
third = converted(0, 8, null, 30, 2)
fourth = converted(0, 8, 10, null, 2)
fifth = converted(0, 8, 10, 30, null)
assert(omitted, null)
assert(fewer, null)
assert(first, null)
assert(second, null)
assert(third, null)
assert(fourth, null)
assert(fifth, null)
assert(converted(0, 8, 10, 30, 2, 99), 15)
assert(Number.isNaN(Math.remap(0, 8, 10, 30)), true)
"#);
    for (name, restored) in [
        ("start", "0"),
        ("end", "8"),
        ("targetStart", "10"),
        ("targetEnd", "30"),
        ("sample", "2"),
        ("extra", "99"),
    ] {
        checked(
            &mut runtime,
            &format!(
                "{name} = null\nassert(mapped, null)\n{name} = {restored}\nassert(mapped, 15)"
            ),
            None,
        );
    }
    assert!(!runtime.graph.contains(&NodeKey::variable("Numeric")));
}

#[test]
fn intrinsic_conversion_shadowing_method_value_reads_and_ordinary_members_keep_lookup() {
    let runtime = run(r#"
def Number(value) => 999
def remap(start, end, targetStart, targetEnd, value) => 999
def lerp(start, end, weight) => 999
def inverseLerp(start, end, value) => 999
assert(Number("2"), 999)
assert(remap(0, 8, 10, 30, 2), 999)
sample = "2"
mapped = Math.remap("0", "8", "10", "30", sample)
assert(mapped, 15)
sample = "6"
assert(mapped, 25)
def numeric(Number):
    return Math.remap("0", "8", "10", "30", "2")
assert(numeric(null), 15)
assert(Math.lerp(10, 30, 0.25), 15)
assert(Math.inverseLerp(0, 8, 2), 0.25)
assert(Math.sign(-2), -1)
"#);
    assert_eq!(dependencies(&runtime, "mapped"), ["sample"]);
    let runtime = run(r#"
Number = 7
mapped = Math.remap("0", "8", "10", "30", "2")
assert(mapped, 15)
reference = Math.remap
indirect = reference(0, 8, 10, 30, 2)
assert(reference, null)
assert(indirect, null)
class Namespace:
    def remap(start, end, targetStart, targetEnd, value) => start + end + targetStart + targetEnd + value
class Holder:
    def Math(value) => value + 4
namespace = Namespace()
holder = Holder()
assert(holder.Math(2), 6)
holder.Math = namespace
assert(holder.Math.remap(1, 2, 3, 4, 5), 15)
def scoped(Math):
    return Math.remap(1, 2, 3, 4, 5)
assert(scoped(namespace), 15)
assert(scoped(null), null)
Math = namespace
assert(Math.remap(1, 2, 3, 4, 5), 15)
"#);
    assert!(dependencies(&runtime, "mapped").is_empty());
    let mut runtime = run("Math = 7");
    let error = runtime.run("Math.remap(0, 8, 10, 30, 2)").unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Type);
    assert_eq!(error.message(), "number.remap is not a function");
    assert_eq!(error.position(), Some(Position::new(1, 1)));
}

#[test]
fn typed_wrappers_aliases_snapshots_receivers_and_call_apis_preserve_frames_and_scope() {
    let runtime = run(r#"
input = 2
end = 8
def remapSample amount: ArbitraryType => Math.remap(0, end, 10, 30, amount)
alias = remapSample
frozen = remapSample.value
mapped = alias(input)
snapshot = mapped.value
assert(mapped, 15)
assert(snapshot, 15)
input = 6
assert(mapped, 25)
assert(snapshot, 15)
try:
    def remapSample amount: ArbitraryType => Math.remap(0, end, 10, 30, missing)
catch error:
    assert(error, ReferenceError("missing is not defined"))
assert(mapped, 25)
input = 4
assert(mapped, 20)
def remapSample amount: ArbitraryType => Math.remap(0, end, 10, 30, amount) + 1
assert(mapped, 21)
assert(frozen(2), 15)
class Reading:
    def init(start, end, targetStart, targetEnd, sample):
        this.start = start
        this.end = end
        this.targetStart = targetStart
        this.targetEnd = targetEnd
        this.sample = sample
    def mapped() => Math.remap(self.start, self.end, self.targetStart, self.targetEnd, self.sample)
reading = Reading(10, 2, 30, 10, 8)
assert(reading.mapped(), 25)
reading.sample = 4
assert(reading.mapped(), 15)
"#);
    for name in ["alias", "input", "end"] {
        assert!(dependencies(&runtime, "mapped").contains(&name), "{name}");
    }
    for name in ["ArbitraryType", "missing"] {
        assert!(!runtime.graph.contains(&NodeKey::variable(name)));
    }
    let statements = parse("Math.remap(start, end, targetStart, targetEnd, sample)").unwrap();
    let [Stmt::Expression(expression)] = statements.as_slice() else {
        panic!("ordinary call required");
    };
    let Ast::Call(call) = Ast::convert(expression) else {
        panic!("no new AST representation");
    };
    assert_eq!(call.arguments().len(), 5);
    let mut runtime = Runtime::new();
    let mut scope = Scope::new();
    scope.push();
    for (name, value) in [
        ("start", 0),
        ("end", 8),
        ("targetStart", 10),
        ("targetEnd", 30),
        ("sample", 2),
    ] {
        scope.declare(name, Value::number(value));
    }
    let receiver = ObjectId::named("caller");
    scope.set_this(Some(receiver.clone()));
    scope.set_instance(Some(receiver.clone()));
    assert_numeric_same(
        &call.resolve(&mut runtime, &mut scope).unwrap(),
        &Value::number(15),
    );
    scope.declare("sample", Value::Null);
    let accepted =
        parse_expression("Math.lerp(targetStart, targetEnd, Math.inverseLerp(start, end, sample))")
            .unwrap();
    assert_numeric_same(
        &call.resolve(&mut runtime, &mut scope).unwrap(),
        &Ast::convert(&accepted)
            .resolve(&mut runtime, &mut scope)
            .unwrap(),
    );
    assert_numeric_same(
        &call.resolve(&mut runtime, &mut scope).unwrap(),
        &Value::number(10),
    );
    let failed =
        parse_expression("Math.remap(start, end, targetStart, targetEnd, sample, missing)")
            .unwrap();
    let error = Ast::convert(&failed)
        .resolve(&mut runtime, &mut scope)
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Reference);
    assert_eq!(error.message(), "missing is not defined");
    assert_eq!(scope.depth(), 2);
    assert_eq!(scope.retrieve("sample"), Some(&Value::Null));
    assert_eq!(scope.this(), Some(&receiver));
    assert_eq!(scope.instance(), Some(&receiver));
}

#[test]
fn failed_relationship_and_class_rule_changes_restore_subscriptions_and_recovery() {
    let runtime = run(r#"
class Gauge
$Gauge.mapped = Math.remap($Gauge.start, $Gauge.end, $Gauge.targetStart, $Gauge.targetEnd, $Gauge.sample)
reading = Gauge()
reading.start = 0
reading.end = 8
reading.targetStart = 10
reading.targetEnd = 30
reading.sample = 2
sample = reading.sample
alternative = 4
mapped = Math.remap(0, 8, 10, 30, sample)
trace = why mapped
assert(mapped, 15)
assert(reading.mapped, 15)
try:
    mapped = Math.remap(0, 8, 10, 30, alternative)
    reading.sample = 6
    throw "ROLLBACK"
catch error:
    assert(error, "ROLLBACK")
assert(mapped, 15)
assert(reading.mapped, 15)
assert(trace[0].from, ["sample"])
try:
    $Gauge.mapped = Math.remap(missing, $Gauge.end, $Gauge.targetStart, $Gauge.targetEnd, $Gauge.sample)
catch error:
    assert(error, ReferenceError("missing is not defined"))
alternative = 12
assert(mapped, 15)
reading.sample = 6
assert(mapped, 25)
assert(reading.mapped, 25)
delete reading.sample
assert(sample, null)
assert(mapped, null)
assert(reading.mapped, null)
reading.sample = -4
assert(mapped, 0)
assert(reading.mapped, 0)
"#);
    assert_eq!(dependencies(&runtime, "mapped"), ["sample"]);
    assert!(runtime.graph.reaches(
        &NodeKey::property(&ObjectId::named("reading"), "sample"),
        &NodeKey::variable("mapped"),
    ));
    assert!(!runtime.graph.reaches(
        &NodeKey::variable("alternative"),
        &NodeKey::variable("mapped"),
    ));
}

#[test]
fn caught_and_uncaught_application_failures_restore_inputs_and_leave_propagation_usable() {
    let mut runtimes = [
        "Math.remap(start, end, targetStart, targetEnd, sample)",
        "Math.lerp(targetStart, targetEnd, Math.inverseLerp(start, end, sample))",
    ].map(|expression| run(&format!(
        "start = 0\nend = 8\ntargetStart = 10\ntargetEnd = 30\nsample = 2\nmapped = {expression}\n\
         if mapped > 30:\n    throw \"TOO_HIGH\""
    )));
    for runtime in &mut runtimes {
        let keys = runtime.graph.keys().cloned().collect::<Vec<_>>();
        let error = runtime.run("sample = 12").unwrap_err();
        assert_eq!(error.thrown_value(), Some(&Value::string("TOO_HIGH")));
        assert_eq!(error.position(), Some(Position::new(1, 1)));
        assert_eq!(runtime.graph.keys().cloned().collect::<Vec<_>>(), keys);
        checked(
            runtime,
            r#"
assert(sample, 2)
assert(mapped, 15)
try:
    sample = 16
catch error:
    assert(error, "TOO_HIGH")
assert(sample, 2)
assert(mapped, 15)
sample = 6
assert(mapped, 25)
"#,
            None,
        );
    }
    assert_numeric_parity(&runtimes[0], &runtimes[1], "mapped");
}

#[test]
fn native_cli_file_bytes_rounding_omissions_effects_errors_and_literal_eof_are_exact() {
    for (index, (source, success, stdout, stderr)) in [
        ("assert(Math.remap(0, 8, 10, 30, 2), 15)\nMath.remap(0, 8, 10, 30, 2)", true, "15.0\n", ""),
        ("Math.remap(0, 8, 10, 30, 12)", true, "40.0\n", ""),
        ("Math.remap(10, 2, 30, 10, 14)", true, "40.0\n", ""),
        ("Math.remap(0, 3, 0, 1, 1)", true, "0.3333333333333333\n", ""),
        ("Math.remap(-Number.MAX_VALUE, Number.MAX_VALUE, 10, 30, 0)", true, "10.0\n", ""),
        ("Math.remap(0, 8, -Number.MAX_VALUE, Number.MAX_VALUE, 2)", true, "null\n", ""),
        ("Math.remap(0, 1, 4, 4, Number.POSITIVE_INFINITY)", true, "null\n", ""),
        ("Math.remap(2, 2, 10, 30, 2)", true, "null\n", ""),
        ("Math.remap(2, 2, 10, 30, 3)", true, "null\n", ""),
        ("Math.remap(0, 1, 0, 1, 1)", true, "1.0\n", ""),
        ("String(Math.remap())", true, "\"NaN\"\n", ""),
        ("String(Math.remap(0))", true, "\"NaN\"\n", ""),
        ("String(Math.remap(0, 8))", true, "\"NaN\"\n", ""),
        ("String(Math.remap(0, 8, 0))", true, "\"NaN\"\n", ""),
        ("String(Math.remap(0, 8, 0, 1))", true, "\"NaN\"\n", ""),
        ("Math.remap(false, true, 0, 1, null)", true, "0.0\n", ""),
        ("Math.remap(0, 8, 10, 30, 2, Number.NaN, \"invalid\", {})", true, "15.0\n", ""),
        ("class Log\nlog = Log()\nlog.order = 0\nMath.remap(0, 8, 10, 30, 2, log.order = 9)\nlog.order",
         true, "9.0\n", ""),
        ("Math.remap(0, 8, 10, 30, 2, missing)", false, "",
         "ReferenceError: missing is not defined (line 1, column 1)\n"),
        ("class Probe\nprobe = Probe()\nMath.remap(probe.absent, 8, 10, 30, 2, Math.random())",
         false, "", "TypeError: Math.random is not supported (line 3, column 1)\n"),
        ("Math.remap(", false, "", "SyntaxError: Unexpected end of input (line 1, column 12)\n"),
        ("Math.remap(\n", false, "", "SyntaxError: Unexpected end of input (line 2, column 1)\n"),
    ].into_iter().enumerate() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("target").join(format!(
            "nuc39-remap-{}-{index}.nuc", std::process::id()
        ));
        fs::OpenOptions::new().write(true).create_new(true).open(&path).unwrap()
            .write_all(source.as_bytes()).unwrap();
        assert_eq!(fs::read(&path).unwrap(), source.as_bytes());
        let output = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
            .arg("--json").arg(&path).stdin(Stdio::null()).output();
        let cleanup = fs::remove_file(&path);
        let output = output.unwrap();
        cleanup.unwrap();
        assert_eq!(output.status.success(), success, "{source}: {output:?}");
        assert_eq!(output.stdout, stdout.as_bytes(), "{source}");
        assert_eq!(output.stderr, stderr.as_bytes(), "{source}");
    }
}

fn stdin_cli(source: &str) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_nucleoid"))
        .arg("--json")
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
fn native_stdin_eof_and_error_recovery_keep_exact_framing_and_live_effects() {
    let banner = "🌿 Nucleoid - Logic Language for World Models\n🌎 Inspired by Nature\nType .help for available commands.\n";
    let source = "start = 0\nend = 8\ntargetStart = 10\ntargetEnd = 30\nsample = 2\n\
                  mapped = Math.remap(start, end, targetStart, targetEnd, sample)\n\
                  Math.remap(0, 8, 10, 30, 2, missing)\nsample = 12\nmapped";
    assert!(!source.ends_with('\n'));
    let output = stdin_cli(source);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        output.stderr,
        b"ReferenceError: missing is not defined (line 1, column 1)\n"
    );
    assert_eq!(
        output.stdout,
        format!("{banner}0.0\n8.0\n10.0\n30.0\n2.0\n15.0\n12.0\n40.0\n").as_bytes()
    );
    // The inherited REPL appends a newline even to a final unterminated line;
    // file-mode tests above separately protect literal byte-for-byte EOF.
    for (source, stderr) in [
        (
            "Math.remap(",
            "SyntaxError: Unexpected end of input (line 2, column 1)\n",
        ),
        (
            "Math.remap(\n",
            "SyntaxError: Unexpected end of input (line 2, column 1)\n",
        ),
    ] {
        let output = stdin_cli(source);
        assert!(output.status.success(), "{output:?}");
        assert_eq!(output.stdout, banner.as_bytes());
        assert_eq!(output.stderr, stderr.as_bytes());
    }
    let output = stdin_cli(
        "class Log\nlog = Log()\nlog.order = 0\nMath.remap(0, 8, 10, 30, 2, log.order = 9)\nlog.order",
    );
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("{banner}null\n{{\"id\":\"log\"}}\n0.0\n15.0\n9.0\n")
    );
}
