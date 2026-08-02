---
license: apache-2.0
language:
  - en
tags:
  - nucleoid
  - logic-programming
  - declarative
  - neuro-symbolic
  - code
size_categories:
  - n<1K
task_categories:
  - text-generation
configs:
  - config_name: default
    data_files:
      - split: spec
        path: spec.jsonl
      - split: synth
        path: synth.*.jsonl
---

# Nucleoid

Nucleoid is a declarative logic programming language for LLMs. A program is a
set of statements that remain true: an assignment is not an instruction that
runs once and finishes, it is a relationship the runtime records and maintains.

This dataset is the language's specification and its synthesized use cases as
JSONL, one record per case. Each record is a complete program with the
natural-language description of every statement kept as comments, which is what
makes it a supervised pair: prose in, logic out.

## Splits

| Split | Records | Rendered from |
| --- | --- | --- |
| `spec` | 178 | `nucleoid.spec.md` — normative |
| `synth` | 1007 | `synth/nucleoid.spec.synth.01.md` through `.21.md` — derived, no independent authority |

## Fields

| Field | Type | Description |
| --- | --- | --- |
| `id` | string | Stable identifier, prefixed by the document: `spec-0001`, `synth-01-0001` |
| `source` | string | Path of the document the case was rendered from |
| `title` | string | The behaviour the case demonstrates |
| `code` | string | The Nucleoid program |
| `returns` | string or null | The value the program evaluates to, as the document writes it, when it declares one |

```json
{
  "id": "spec-0001",
  "source": "nucleoid.spec.md",
  "title": "Nucleoid runs a statement in the state",
  "code": "# i is 1\ni = 1\n\nassert(i == 1, true)",
  "returns": null
}
```

## Loading

```python
from datasets import load_dataset

dataset = load_dataset(
    "json",
    data_files={"spec": "spec.jsonl", "synth": "synth.*.jsonl"},
)
```

Once published to the Hub, load it by its repository id instead.

## How it is built

The records are not written by hand. They are rendered from the documents in
the [Nucleoid repository](https://github.com/NucleoidAI/Nucleoid), and the test
suite fails when the committed JSONL is not what those documents render to, so
the dataset cannot drift from the specification it publishes.

Every `code` in this dataset is a program the language's own test suite runs:
the assertions in it hold, and each one parses. The comment that titles a case
in the source document is promoted to the `title` field rather than left in the
program, so a model trained on this does not learn to write it back.

`nucleoid.spec.md` is normative. The `synth` split is derived from it and has no
authority of its own; where the two disagree, the specification wins.

## Copyright

Copyright 2020 Nucleoid

This dataset is licensed under the Apache License, Version 2.0.
