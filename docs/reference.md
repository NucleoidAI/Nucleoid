# Nucleoid Language Reference

Nucleoid is a logic programming language for LLMs.

A Nucleoid program is a set of statements that remain true. An assignment is not
an instruction that runs once and finishes; it is a relationship the runtime
records and maintains. Writing `c = a + b` states that `c` is the sum of `a` and
`b`, and that stays true as `a` and `b` change.

This document is the consolidated reference, assembled from the NUC documents.
See [NUC 0](nuc-0000.md) for the index and [NUC 1](nuc-0001.md) for the
conventions. `nucleoid.spec.md` is normative; where this document and the
specification disagree, the specification wins.

## Contents

1. [Statements and State](#1-statements-and-state)
2. [Variables](#2-variables)
3. [Expressions](#3-expressions)
4. [Types and Instances](#4-types-and-instances)
5. [Properties](#5-properties)
6. [Class-Level Rules](#6-class-level-rules)
7. [Blocks and Scope](#7-blocks-and-scope)
8. [Control Flow](#8-control-flow)
9. [Functions](#9-functions)
10. [Transactions](#10-transactions)
11. [Built-in Objects](#11-built-in-objects)
12. [Error Reference](#12-error-reference)
13. [Syntax Summary](#13-syntax-summary)

---

## 1. Statements and State

- A statement is run against the state and changes it.
- An assignment records a relationship, not just a result.
- An expression statement returns its value.
- Statements are applied in the order received; when two assign the same target, the last one holds.
- Every statement runs in a transaction.

```
i = 1
```

```
k = 1
k
```

- Returns `1`.

Full detail: [NUC 2](nuc-0002.md).

---

## 2. Variables

### Assignment

- A variable is created by assigning to it; no keyword is required.
- An assignment evaluates to the assigned value.
- A declaration with a type but no value is rejected.

```
a: int
```

- Raises `ReferenceError("Missing definition")`.

### Dependency

- Naming a variable on the right-hand side establishes a dependency.
- Dependents re-evaluate when a dependency changes.
- Dependencies are transitive.

```
a = 1
b = 2
c = a + b
```

- `c` is `3`; after `a = 2`, `c` is `4`.

### Re-assignment

- Assigning again discards the previous dependencies and records the new ones.

```
c = a + 3
c = b + 3
```

- `c` now follows `b`, not `a`.

### Self-reference

- A variable's own occurrence on the right is read as its current value.
- This is not a cycle.

```
radius = 10
radius = radius + 10
```

- `radius` is `20`.

### The value property

- `.value` reads the current value and records no dependency.
- It breaks what would otherwise be a cycle.
- The value of a `null` property is `0`.

```
goldenRatio = 1.618
altitude = 10
width = goldenRatio.value * altitude
```

- `width` is `16.18`; unchanged by `goldenRatio = 1.62`; becomes `161.8` after `altitude = 100`.

### Null and undefined

- Any `null` dependency yields `null`.
- Reading something undefined yields `null` and re-evaluates when it arrives.

### Deletion

- `delete` removes a variable and returns whether anything was removed.
- Dependents of a deleted variable become `null`.
- Using a deleted variable raises a reference error.

```
base = 10
tax = base * 0.1
total = base + tax

delete base
```

- `tax` and `total` are `null`; using `base` raises `ReferenceError("base is not defined")`.

Full detail: [NUC 2](nuc-0002.md).

---

## 3. Expressions

### Literals

- Numbers, including negatives and fractions.
- Strings in single quotes, double quotes, or backticks.
- Template literals interpolate with `${...}` and track their dependencies.
- `true`, `false`, `null`.
- Lists in brackets; objects in braces, with quoted or bare keys.
- Regular expressions between slashes.

```
host = "localhost"
port = 8080
url = `http://${host}:${port}`
```

- After `port = 9090`, `url` is `"http://localhost:9090"`.

### Operators

- Arithmetic: `+`, `-`, `*`, `/`, `%`.
- Comparison: `==`, `!=`, `<`, `>`, `<=`, `>=`.
- Logical: `and` / `&&`, `or` / `||`, `not` / `!`.
- Parentheses override precedence.
- `+` concatenates when an operand is a string.

```
assert(a + b * c, 14)
assert((a + b) * c, 20)
```

- A comparison may be assigned directly: `alarm = reading > threshold`.

### Indexing and length

- Lists and instance lists index with brackets; the index may be a variable.
- `.length` is a dependency on the collection.

Full detail: [NUC 10](nuc-0010.md).

---

## 4. Types and Instances

### Declaring a type

- `class` declares a type; an empty body is `pass`.
- `$Name` is the type; `Name` is the list of its instances.
- `typeof $Name` is `Class`; `typeof Name` is `List`.
- Every declared type is added to the global `Class` list.

```
class Entity:
    pass
```

### Constructors

- Parameters may be declared on the type, with optional annotations.
- An equivalent form declares attributes plus an `init` function.
- A missing argument leaves the attribute `null`.

```
class Shape(type: str):
    this.type = type
```

```
class Shape:
    type: str

    def init(type: str):
        this.type = type
```

### Instances

- Creating an instance registers it in the type's list and returns it.
- The variable name becomes the instance `id`.
- An instance created without a variable name gets a generated `id`.
- Instances are addressed by index `Name[0]`, by id `Name["id"]`, or by query `Name.find(...)`.

### Subtypes

- A subtype names its supertype after the colon; `super(...)` invokes its constructor.
- Class-level rules on the supertype apply to subtype instances.

```
class Truck: Vehicle
    def init(make, payload):
        super(make)
        this.payload = payload
```

### Redefining and deleting

- Re-declaring a type preserves the class list and existing instances.
- New attributes are `null` on existing instances.
- `delete` removes an instance and returns whether anything was removed.
- Deletion is refused while the instance still has properties.
- A variable name is reusable after its instance is deleted.

Full detail: [NUC 3](nuc-0003.md).

---

## 5. Properties

- A property is created by assigning to it, and behaves like a variable assignment.
- A property may be assigned from one that does not exist yet; the result is `null` until it does.
- Assigning again replaces the relationship; self-reference reads the current value.

```
order1.upc = "04061" + order1.barcode
```

- `null` until `order1.barcode` is set, then `"0406194067"`.

### Chains through references

- A property may read through another instance, and the dependency follows the reference.
- Replacing the referenced instance re-points the dependency.
- Writing through a reference records the rule on the referenced instance.

```
user1.plan = basic
user1.bill = user1.plan.rate * 12
```

- `120`; after `user1.plan = pro`, `360`; after `pro.rate = 40`, `480`.

### Null, value, and deletion

- Reading through a missing link yields `null`; writing through one raises.
- `value` may not be used as a property name.
- Deleting a property makes its dependents `null` and leaves others untouched.

Full detail: [NUC 4](nuc-0004.md).

---

## 6. Class-Level Rules

- `$Name.property = ...` states a rule for every instance of the type.
- Rules apply to instances created before and after them.
- Rules may chain, read variables, and read through references into other types.

```
class Human(name: str):
    this.name = name

$Human.mortal = true
```

- Every `Human` has `mortal` `true`.

### Conditionals, order, and overriding

- `if`, `else if`, and `else` may be written over `$Name`; each instance is evaluated independently.
- Instances whose inputs are missing are left untouched.
- Rules apply in the order received; when several assign the same property, the last matching one wins.
- Assigning the property on one instance overrides the rule for that instance only.
- A rule on a subtype overrides the supertype's.

```
if $Grade.score > 89:
    $Grade.letter = "A"
else if $Grade.score > 79:
    $Grade.letter = "B"
else:
    $Grade.letter = "C"
```

### Aggregation

- A rule may aggregate another type's list.

```
$Category.revenue = Product.filter(p => p.category == $Category).reduce((sum, p) => sum + p.price, 0)
```

Full detail: [NUC 5](nuc-0005.md).

---

## 7. Blocks and Scope

- A block is written with braces and introduces a scope.
- A name first assigned inside a block is local to it and does not survive it.
- Dependencies carried through a local do survive.
- Blocks nest; an inner block sees enclosing locals.
- A local shadows an outer name; lookup takes the lowest enclosing binding.

```
h = 1

{
    value = h * 2
    j = value * 2
}
```

- `j` is `4`; after `h = 2`, `j` is `8`.

### Instances, guards, and return

- Instances may be created in a block and assigned outward.
- A local may hold an instance from the state; properties assigned through it are recorded on that instance.
- `return` yields a value; the first `return` reached wins.
- A block may return a local, a property of a local, or an inline object or list.

```
{
    device = Device.find(d => d.code == "A0")
    if not device:
        throw "INVALID_DEVICE"
    return device
}
```

Full detail: [NUC 6](nuc-0006.md).

---

## 8. Control Flow

### Conditionals

- A conditional is a standing rule, re-evaluated when its condition or its branches change.
- A false condition leaves the target untouched rather than clearing it.
- Re-declaring a conditional replaces it.
- Conditionals apply in the order received; when several match, the last one holds.

```
if toy1.color == "RED":
    toy1.shape = "CIRCLE"
```

- `null` until `toy1.color` is `"RED"`.

### for ... of

- `for <name> of <source>` iterates a class list or a list value.
- The body runs once, over the values present at the time; it does not re-run later.
- Only values that are instances defined in the state are visited.
- Loops may nest, producing the cross product.

```
for question of Question:
    Summary(question)
```

Full detail: [NUC 7](nuc-0007.md).

---

## 9. Functions

- `def` defines a function in the state; a function with no `return` yields `null`.
- A call in an assignment depends on the function and its arguments.
- Redefining a function updates existing dependents.
- Functions may recurse and may have multiple `return` statements.

```
def generate(number):
    return number * 10

random = 10
number = generate(random)
```

- `100`; after `random = 20`, `200`.

### Lambdas

- Three equivalent forms; parameters shadow outer names.

```
list.find(function(element) { return element == 3 })
list.find(element => { return element == 2 })
list.find(element => element == 1)
```

- Calls over class lists depend on the population and the properties they read.

Full detail: [NUC 8](nuc-0008.md).

---

## 10. Transactions

- A statement and every rule it triggers form one transaction.
- If anything throws, the whole transaction is rolled back and the state is exactly as before.
- `throw` raises a value: a string, a number, or a resolved reference.
- Blocks, including nested blocks, are single transactions.
- An instance rejected at creation is not registered and its name is not bound.
- Instances created earlier in a failing loop are rolled back too.
- A class-level rule that throws for any existing instance is not installed at all.

```
a = 5

if a > 5:
    throw 'INVALID_VALUE'
```

- Assigning `a = 6` throws; afterwards `a` is `5`.

Full detail: [NUC 9](nuc-0009.md).

---

## 11. Built-in Objects

- Built-in calls participate in the dependency graph like any other expression.
- Mutating a list re-evaluates its dependents.

| Object | Members |
| --- | --- |
| `Object` | `Object()` -- a plain object, not an instance of any type |
| `Boolean` | `Boolean(value)` |
| `Number` | `Number.MAX_INTEGER` |
| `Math` | `pow`, `sqrt`, `floor`, `round`, `max`, `min` |
| `String` | `String.fromCharCode`; instance `charAt`, `length`, `lower`, `replace`, `split` |
| `Date` | `Date()`, `Date(string)`, `Date(number)`, `Date.now()`, `Date.parse()`; instance `getTime`, `getYear`, `toDateString` |
| `List` | `push`, `pop`, `filter`, `map`, `reduce`, `find`, `some`, `every`, `slice`, `sort`, `join`, `length` |
| `Class` | the global list of declared types; `Class.length` |

```
prices = [10, 20, 30]
rate = 2
doubled = prices.map(p => p * rate)
```

- `doubled[2]` is `60`; after `rate = 3`, `90`; after `prices.push(40)`, `doubled[3]` is `120`.

- Class lists support the same functions and depend on the population.

```
open = Task.filter(t => not t.done).length
```

- Regular expressions support `.test(...)` and may be used in any condition.

Full detail: [NUC 10](nuc-0010.md).

---

## 12. Error Reference

Messages are exact and are part of the observable behaviour of the language.

### ReferenceError

| Condition | Message |
| --- | --- |
| Reading an undefined variable | `<name> is not defined` |
| Throwing an undefined reference | `<name> is not defined` |
| Using a deleted variable or instance | `<name> is not defined` |
| Creating an instance of an undeclared type | `<Name> is not defined` |
| Writing through an undefined instance property | `<path> is not defined` |
| Writing through an undefined class-level reference | `<Name>.<property> is not defined` |
| Reading `.value` of an undefined property | `<path> is not defined` |
| Declaring a variable with no value | `Missing definition` |

### TypeError

| Condition | Message |
| --- | --- |
| An assignment that would close a dependency cycle | `Circular Dependency` |
| Deleting an instance that still has properties | `Cannot delete object '<id>'` |
| Using `value` as a property name | `Cannot use 'value' as a property` |
| Calling an unknown function of a built-in or list | `<Object>.<name> is not a function` |

### SyntaxError

| Condition | Message |
| --- | --- |
| Declaring a class-level rule in a non-class block | `Cannot define class declaration in non-class block` |

Full detail: [NUC 9](nuc-0009.md).

---

## 13. Syntax Summary

```
name = expression                      variable assignment
name: type                             declaration without value -- rejected
name.property = expression             property assignment
$Type.property = expression            class-level rule
delete name                            remove a variable, property, or instance

class Name:                            type declaration
    pass

class Name(param: type):               type with a constructor
    this.param = param

class Name:                            type with a typed attribute
    attribute: type

    def init(param: type):
        this.attribute = param

class Sub: Super                       subtype
    def init(a, b):
        super(a)
        this.b = b

if condition:                          conditional
    statement
else if condition:
    statement
else:
    statement

for name of source:                    iteration
    statement

def name(params):                      function
    return expression

{ statement; statement }               block

throw value                            raise a value
return expression                      yield from a block or function

name.value                             read without depending
Name[0]  Name["id"]  Name.find(...)    address an instance
```

---

## Copyright

This document is placed in the public domain or under the CC0-1.0-Universal
license, whichever is more permissive.
