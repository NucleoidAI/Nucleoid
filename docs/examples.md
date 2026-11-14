# Nucleoid Examples

Worked examples of the language, each one a complete program that runs as written.

The sections follow [README.md](README.md) one for one, so an example can
be read next to the behaviour it demonstrates. Every example is taken from
`nucleoid.spec.md`, which is normative; where this document and the
specification disagree, the specification wins. See [NUC 0](nuc-0000.md) for the
index of NUC documents.

The examples are executable. `cargo test --test docs` runs every block on
this page and checks the assertions in it, so an example that stops being true
fails the build.

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
12. [Reasoning Operations](#12-reasoning-operations)
13. [Error Reference](#13-error-reference)

[Syntax Summary](README.md#14-syntax-summary) is a table of forms rather than
a behaviour, so it has no example of its own.

---

## 1. Statements and State

**A statement runs against the state, and an expression statement returns its value**

- A statement is run against the state and changes it.
- An expression statement is evaluated against what the state already holds.

```nuc
i = 1

assert(i == 1, true)

j = 1

assert(j + 2, 3)
```

**Statements apply in the order received, and the last one to assign a target holds**

- All five conditionals are standing rules and all are kept.
- When `any` becomes 4 the first, second and fourth match; the last of them holds.

```nuc
any = 0

if any > 1:
    result = 1

if any > 2:
    result = 2

if any > 3:
    result = 3

if any > 2:
    result = 4

if any > 1:
    result = 5

any = 4

assert(result, 5)
```

Full detail: [README.md §1](README.md#1-statements-and-state), [NUC 2](nuc-0002.md).

---

## 2. Variables

**An assignment states a relationship, and stating it again replaces it**

- `c` is the sum of `a` and `b`, and stays the sum as they change.
- Assigning `c` again discards the dependency on `a` and records the one on `b`.

```nuc
a = 1
b = 2
c = a + b

assert(c, 3)

a = 2

assert(c, 4)

c = b + 3

assert(c, 5)

b = 4

assert(c, 7)
```

**Upstream dependencies settle first even when their declaration is newer**

- `total` reads both the head and the tail of the chain.
- Replacing `middle` does not let `total` combine a new head with an old tail.

```nuc
source = 1
middle = source + 1
last = middle + 1
total = source + last

middle = source + 2
source = 2

assert(middle, 4)
assert(last, 5)
assert(total, 7)
```

**A variable that reads itself reads only its value**

- The occurrence on the right is the current value, not a dependency.
- This is why it is not a cycle.

```nuc
radius = 10
radius += 10

assert(radius, 20)
```

**Compound assignments keep their reactive relationships**

- `amount += step` states the same relationship as `amount = amount + step`.
- Changing `step` re-evaluates that relationship against `amount`'s current value.
- Other arithmetic operators use the same lowering; `.value` on the right still takes a snapshot.
- These compact forms are specified by accepted [NUC 13](nuc-0013.md).

```nuc
amount = 10
step = 2
amount += step
assert(amount, 12)
step = 3
assert(amount, 15)
```

**The value property reads a variable without depending on it**

- `width` is fixed against `goldenRatio` and still follows `altitude`.
- It is how a relationship is broken deliberately rather than by accident.

```nuc
goldenRatio = 1.618
altitude = 10
width = goldenRatio.value * altitude

assert(width, 16.18)

goldenRatio = 1.62

assert(width, 16.18)

altitude = 100

assert(width, 161.8)
```

**Deleting a variable clears what depended on it**

- `delete` reports whether anything was removed.
- The statements that read the name are gone, and using it again is a reference error.

```nuc
t = 1
q = t + 1

assert(q, 2)
assert(delete q, true)

t = 2

try:
    q
catch error:
    assert(error, ReferenceError("q is not defined"))
```

Full detail: [README.md §2](README.md#2-variables), [NUC 2](nuc-0002.md).

---

## 3. Expressions

**A string may be written three ways, and a template interpolates what it reads**

- Single quotes, double quotes and backticks all give a string.
- `${...}` interpolates, and what it reads is a dependency like any other.

```nuc
a = 123

assert('New String', "New String")
assert("New String", "New String")
assert(`New String`, "New String")

assert(`New ${a} String`, "New 123 String")
```

**Logical operators have a word spelling and a symbol spelling**

- `and`, `or` and `not` are the same operators as `&&`, `||` and `!`.

```nuc
condition = false

assert(condition or true, true)
assert(condition || true, true)

assert(not condition and true, true)
assert(!condition && true, true)
```

**A list indexes with brackets**

```nuc
states = ["NY", "GA", "CT", "MI"]

assert(states[2], "CT")
```

**Length is a dependency like any other**

- `i1` follows the length of `str1`, and so does the condition below it.

```nuc
str1 = "ABC"
i1 = str1.length + 1

assert(i1, 4)

str1 = "ABCD"

assert(i1, 5)

if str1.length > 5:
    i2 = i1

str1 = "ABCDEF"

assert(i2, 7)
```

**Reusable readings keep named fields concise**

- This isolated [Draft NUC 17](nuc-0017.md) candidate returns an object whose
  ordinary identifier fields reuse their names.
- The function still reads its parameters; changing the argument updates the
  existing reactive call relationship.
- Named arrows may return `{ reading, unit }` directly; anonymous arrows
  returning an object still need `reading => ({ reading })`.

```nuc
reading = 7
unit = "C"

def pack reading unit =>
    { reading, unit }

report = pack(reading, unit)

assert(report.reading, 7)
assert(report.unit, "C")

reading = 9

assert(report.reading, 9)
```

Full detail: [README.md §3](README.md#3-expressions), [NUC 10](nuc-0010.md).

---

## 4. Types and Instances

The isolated [NUC 47 revision](nuc-0047.md) supports positional, named and
positional-then-named construction with ordinary headers and no star.
See its [paired reference example](README.md#default-named-constructors).
The positional programs below remain unchanged; explicit grouped assignments
are the escape for caller-write arguments rather than bare constructor labels.

**Empty types need no placeholder body**

- The header declares the same empty type as an indented pass-only body.
- Instance registration and class-level rules keep their existing behavior.

```nuc
class Entity
entity1 = Entity()
$Entity.enabled = true
assert(typeof Entity, List)
assert(entity1.enabled, true)
assert(Entity.length, 1)
```

**A type declares its constructor, and instances register themselves**

- `Shape` is the list of instances; `$Shape` is the type.
- The variable name becomes the instance `id`, so the instance is addressable by it.

```nuc
class Shape(type: str):
    this.type = type

shape1 = Shape("Square")

assert(shape1, { "id": "shape1", "type": "Square" })
assert(Shape["shape1"], { "id": "shape1", "type": "Square" })
assert(Shape.length, 1)
```

**`this` and `self` refer to the same instance**

- Either keyword can be used in constructors, methods, and class-level rules.

```nuc
class Counter(count: int):
    self.count = count

    def same_receiver() => this == self

    def set_this(count: int):
        this.count = count
        return self.count

    def set_self(count: int):
        self.count = count
        return this.count

$Counter.doubled = this.count + self.count

counter = Counter(2)

assert(counter.same_receiver(), true)
assert(counter.doubled, 4)
assert(counter.set_self(3), 3)
assert(counter.doubled, 6)
assert(counter.set_this(5), 5)
assert(counter.doubled, 10)
```

**A subtype calls its supertype's constructor**

- The supertype is named after the colon, and `super(...)` invokes it.

```nuc
class Person(name: str):
    this.name = name

class Student: Person
    def init name school:
        super(name)
        this.name = name
        this.school = school

student1 = Student("Emma", "Riverside High")

assert(student1, { "id": "student1", "name": "Emma", "school": "Riverside High" })
```

**Bare super preserves inherited reactive properties**

- `super` runs the zero-argument parent constructor on the existing child.
- Inherited class-level properties still update when their source changes.

```nuc
seed = 4

class Parent:
    def init():
        this.ready = true

$Parent.amount = seed

class Child: Parent
    def init():
        super

child = Child()

assert(child.ready, true)
assert(child.amount, 4)

seed = 6

assert(child.amount, 6)
```

**Re-declaring a type keeps its instances and its rules**

- The instances made before the new declaration are still there, and the rule still holds for them.
- An attribute the earlier instances never had is `null` on them.

```nuc
class Message

$Message.read = false

message1 = Message()

class Message(payload: str):
    this.payload = payload

assert(message1.read, false)
assert(message1.payload, null)

message2 = Message("MESSAGE")

assert(message2.read, false)
assert(message2.payload, "MESSAGE")
```

Full detail: [README.md §4](README.md#4-types-and-instances), [NUC 3](nuc-0003.md); candidate bare calls: [NUC 16](nuc-0016.md).

---

## 5. Properties

**A property may be assigned from one that does not exist yet**

- Reading something undefined yields `null` and re-evaluates when it arrives.

```nuc
class Order

order1 = Order()
order1.upc = "04061" + order1.barcode

assert(order1.upc, null)

order1.barcode = "94067"

assert(order1.upc, "0406194067")
```

**A property reads through a reference, and the dependency follows it**

- `server1.summary` reads `ip1` through `server1.ip`, so a change to `ip1` reaches it.

```nuc
class Server

class IP

server1 = Server()
server1.name = "HOST1"

ip1 = IP()
ip1.address = "10.0.0.1"

server1.ip = ip1
server1.summary = server1.name + "@" + server1.ip.address

assert(server1.summary, "HOST1@10.0.0.1")

ip1.address = "10.0.0.2"

assert(server1.summary, "HOST1@10.0.0.2")
```

**Deleting a property makes its dependents null**

- The properties the deleted one did not feed are left untouched.

```nuc
class Agent

agent = Agent()
agent.time = 52926163455
agent.location = "CITY"
agent.report = agent.time + "@" + agent.location

assert(agent.report, "52926163455@CITY")

delete agent.time

assert(agent.report, null)
assert(agent.location, "CITY")
```

Full detail: [README.md §5](README.md#5-properties), [NUC 4](nuc-0004.md).

---

## 6. Class-Level Rules

**A rule on a type holds for every instance of it**

- The rule reaches instances that already exist, not only later ones.
- Stating the rule again replaces it, and the instances are brought up to date.

```nuc
class Employee

employee1 = Employee()
employee1.id = 1

$Employee.username = "E" + $Employee.id

assert(employee1.username, "E1")

$Employee.username = "F" + $Employee.id

assert(employee1.username, "F1")

employee1.id = 2

assert(employee1.username, "F2")
```

**A conditional rule leaves the instances it does not match untouched**

- Each instance is evaluated on its own.
- A bare property as the condition is tested for presence, so a device without a profile does not match.

```nuc
class Device(profile: str):
    this.profile = profile

if $Device.profile:
    $Device.active = true

device1 = Device()
device2 = Device("PROFILE-1")

assert(device1.active, null)
assert(device2.active, true)
```

**A branch chain over a type follows its branches as well as its condition**

- `status` depends on the capacity that chooses the branch and on the value the branch assigns.

```nuc
class Storage

normal = "NORMAL"; low = "LOW"; empty = "EMPTY"

if $Storage.capacity > 25:
    $Storage.status = normal
else if $Storage.capacity > 0:
    $Storage.status = low
else
    $Storage.status = empty

storage1 = Storage()
storage1.capacity = 23

assert(storage1.status, "LOW")

low = "L"

assert(storage1.status, "L")
```

**A rule may read across types and is checked as it is declared**

- The rule aggregates the `Registration` list for each `User`.
- Both registrations already point at `user1`, so declaring it throws.

```nuc
class User

class Registration

user1 = User()

registration1 = Registration()
registration1.user = user1

registration2 = Registration()
registration2.user = user1

try:
    if Registration.filter(r => r.user == $User).length > 1:
        throw "USER_ALREADY_REGISTERED"
catch error:
    assert(error, "USER_ALREADY_REGISTERED")
```

Full detail: [README.md §6](README.md#6-class-level-rules), [NUC 5](nuc-0005.md).

---

## 7. Blocks and Scope

**A block's locals do not survive it, but the dependencies carried through them do**

- `value` is local to the block and gone after it.
- `j` was stated in terms of `value`, so it still follows `h`.

```nuc
h = 1

{
    value = h * 2
    j = value * 2
}

assert(j, 4)

h = 2

assert(j, 8)
```

**A local shadows an outer name, and the lowest binding wins**

- The `pi` inside the block is a different name from the one outside it.

```nuc
pi = 3.14
number = pi

{
    pi = 3.141
    number = pi
}

assert(number, 3.141)
```

**A block may guard its work and return a local**

- The query finds nothing, so the guard throws and the block is rolled back.

```nuc
class Device(code: str):
    this.code = code

device1 = Device("A0")
device2 = Device("B1")

try:
    {
        device = Device.find(d => d.code == "A1")
        if not device:
            throw "INVALID_DEVICE"
        return device
    }
catch error:
    assert(error, "INVALID_DEVICE")
```

Full detail: [README.md §7](README.md#7-blocks-and-scope), [NUC 6](nuc-0006.md).

---

## 8. Control Flow

**A conditional is a standing rule, not a branch taken once**

- The condition is false at first, which leaves `shape` alone rather than clearing it.
- The rule is re-evaluated when the colour it reads changes.

```nuc
class Toy

toy1 = Toy()
toy1.color = "BLUE"

if toy1.color == "RED":
    toy1.shape = "CIRCLE"

assert(toy1.shape, null)

toy1.color = "RED"

assert(toy1.shape, "CIRCLE")
```

**Re-declaring a conditional replaces it**

- The second conditional takes the place of the first, so `r` follows `s` from then on.

```nuc
p = 0.01
s = 0.02

if p < 1:
    r = p * 10

if p < 1:
    r = s * 10

assert(r, 0.2)

s = 0.03

assert(r, 0.3)
```

**Re-evaluating a conditional preserves its declaration priority**

- Updating `enabled` runs only the first rule, but does not make it the newest.
- When `trigger` changes, the second matching rule still wins.

```nuc
trigger = 0
enabled = false
result = 0

if enabled and trigger > 0:
    result = 1

if trigger > 0:
    result = 2

enabled = true
trigger = 1

assert(result, 2)
```

**A rule runs again when another input changes later in the same cascade**

- Neither conditional is discarded after its first turn.
- Derived values settle with the final inputs.

```nuc
first = 0
second = 0
total = first + second

if first > 0 and first < 4:
    second = first + 1

if second > 0 and second < 4:
    first = second + 1

first = 1

assert(first, 3)
assert(second, 4)
assert(total, 7)
```

**A conditional inside a loop is not a standing rule**

- The first loop sees `limit` at zero and leaves the instance untouched.
- Changing `limit` does not run that body again; another explicit loop does.

```nuc
class Item

item = Item()
item.amount = 0
limit = 0

for current of Item:
    if limit > 0:
        current.amount = limit

limit = 2

assert(item.amount, 0)

for current of Item:
    if limit > 0:
        current.amount = limit

assert(item.amount, 2)

limit = 3

assert(item.amount, 2)
```

**A for of statement runs its body once, over what is there at the time**

- The loop is carried out immediately; it does not re-run when `Question` grows.
- What it creates is declarative all the same, so the rule on `Summary` still reaches the instances it made.

```nuc
class Question

question1 = Question()

question2 = Question()
question2.archived = true

question3 = Question()

class Summary(question):
    this.question = question

$Summary.type = "DAILY"

for question of Question:
    if not question.archived:
        Summary(question)

assert(Summary.length, 2)
assert(Summary[0].question.id, "question1")
assert(Summary[1].question.id, "question3")

$Summary.type = "WEEKLY"

assert(Summary[0].type, "WEEKLY")
assert(Summary[1].type, "WEEKLY")
```

Full detail: [README.md §8](README.md#8-control-flow), [NUC 7](nuc-0007.md).

---

## 9. Functions

**A function call in an assignment depends on the function and its arguments**

```nuc
def generate number => number * 10

random = 10
number = generate(random)

assert(number, 100)

random = 20

assert(number, 200)
```

**Single parameter named arrows preserve live inputs**

```nuc
fee = 2

def total amount => amount + fee

invoice = total(5)

assert(invoice, 7)

fee = 3

assert(invoice, 8)
```

[NUC 18](nuc-0018.md) specifies parameter-parenthesis omission for a named
arrow definition or method with exactly one untyped ordinary identifier
parameter. `def total(amount) => amount + fee` remains valid; the optional
untyped multiple-parameter arrow spelling is demonstrated in
[bare multiple-parameter named arrows](#bare-multiple-parameter-named-arrows).
Calls retain their parentheses; typed/mixed multiple-parameter arrows have the
separate NUC29 spelling below. The expression still follows
[NUC 12](nuc-0012.md). [NUC 19](nuc-0019.md)
specifies the corresponding narrow indented-colon form, demonstrated in
[single-parameter indented block definitions](#single-parameter-indented-block-definitions).

**Named callbacks and aliases follow redefinition; value freezes the definition**

```nuc
def scale number => number * 2

callback = scale
snapshot = scale.value
result = callback(3)
frozen = snapshot(3)
mapped = [1, 2].map(scale)

def scale number => number * 3

assert(result, 9)
assert(frozen, 6)
assert(mapped, [3, 6])

scale = number => number * 4

assert(result, 12)
assert(frozen, 6)
assert(mapped, [4, 8])

def scale number => number * 5

assert(result, 15)
assert(frozen, 6)
assert(mapped, [5, 10])
```

**Callable properties follow replacement and deletion through either spelling**

```nuc
class Handler

handler = Handler()
handler.operation = number => number + 1
dotted = handler.operation(2)
indexed = handler["operation"](2)

handler.operation = null

assert(dotted, null)
assert(indexed, null)

handler.operation = number => number + 2

assert(dotted, 4)
assert(indexed, 4)

delete handler.operation

assert(dotted, null)
assert(indexed, null)

handler.operation = number => number + 3

assert(dotted, 5)
assert(indexed, 5)
```

**A null or non-callable parameter shadows an outer function**

```nuc
def operation number => 99

def apply operation number => operation(number)

callback = null
result = apply(callback, 3)

assert(result, null)

callback = number => number * 2

assert(result, 6)

failure = null

try:
    apply(7, 3)
catch error:
    failure = error

assert(failure, TypeError("operation is not a function"))
assert(operation(3), 99)
```

**A lambda has three equivalent forms**

```nuc
list = [1, 2, 3]

assert(list.find(function(element) { return element == 3 }), 3)
assert(list.find(element => { return element == 2 }), 2)
assert(list.find(element => element == 1), 1)
```

**A query over a class list depends on the population and on what it reads**

- The chain is re-run when either threshold changes, not when it is written.

```nuc
class Result(score: int):
    this.score = score

Result(10); Result(15); Result(20)

upperThreshold = 18
lowerThreshold = 12

list = Result.filter(r => r.score > lowerThreshold).filter(r => r.score < upperThreshold)

assert(list.length, 1)
assert(list[0].score, 15)

lowerThreshold = 7

assert(list.length, 2)
assert(list[0].score, 10)
assert(list[1].score, 15)

upperThreshold = 14

assert(list.length, 1)
assert(list[0].score, 10)
```

Full detail: [README.md §9](README.md#9-functions), [NUC 8](nuc-0008.md).

---

## 10. Transactions

**A statement and everything it triggers are one transaction**

- The rule throws while `a = 6` is propagating, so the assignment is rolled back.
- Afterwards the state is exactly what it was.

```nuc
a = 5

if a > 5:
    throw 'INVALID_VALUE'

try:
    a = 6
catch error:
    assert(error, "INVALID_VALUE")

assert(a, 5)
```

**A caught failure cancels only the propagation it queued**

- The observer of `source` must not run for the aborted update.
- The unrelated dependent of `trigger` was already queued and still runs.

```nuc
trigger = 0
source = 1
observations = 0

if source > 0:
    observations += 1

if trigger > 0:
    try:
        source = 2
        throw "ABORT"
    catch error:
        if error != "ABORT":
            throw error

remaining = trigger + 10
trigger = 1

assert(source, 1)
assert(observations, 1)
assert(remaining, 11)
```

**A deletion permits temporary undefined reads only during its cascade**

- Dependents settle to `null` while the removed name propagates.
- The direct read after the cascade is a reference error, not another undefined read.

```nuc
trigger = 0
source = 1
dependent = source + 1

if trigger > 0:
    delete source

def read_after_update():
    trigger = 1
    try:
        source
    catch error:
        return error
    return "NO_ERROR"

assert(read_after_update(), ReferenceError("source is not defined"))
assert(dependent, null)
```

**An instance rejected at creation is not registered and its name is not bound**

- The rule throws while the instance is being made, so nothing is left behind.

```nuc
class User(first: str, last: str):
    this.first = first
    this.last = last

if $User.first.length < 3:
    throw 'INVALID_USER'

try:
    user1 = User('F', 'L')
catch error:
    assert(error, "INVALID_USER")

assert(User.length, 0)

try:
    user1
catch error:
    assert(error, ReferenceError("user1 is not defined"))
```

**A thrown value may be a number or a string**

- What `catch` binds is the value that was thrown.

```nuc
length = 0.1

try:
    if length < 1:
        throw length
catch error:
    assert(error, 0.1)

try:
    if length < 1.1:
        throw 'length'
catch error:
    assert(error, "length")
```

Full detail: [README.md §10](README.md#10-transactions), [NUC 9](nuc-0009.md).

---

## 11. Built-in Objects

**A list function re-evaluates when the list it reads is mutated**

- `count` is stated once and answers for every later `push` and `pop`.

```nuc
list = []
count = list.filter(n => n % 2)

list.push(1)

assert(count.length, 1)

list.push(2)

assert(count.length, 1)

list.push(3)

assert(count.length, 2)

list.pop()

assert(count.length, 1)
```

**Built-in calls take part in the graph like any other expression**

- `String.fromCharCode` and `Math.floor` are read by a rule, which then holds for the instance that already exists.

```nuc
class Product

product1 = Product()

class Quality

product1.quality = Quality()
product1.quality.score = 15

assert(product1.quality.class, null)

$Quality.class = String.fromCharCode(65 + Math.floor($Quality.score / 10))

assert(product1.quality.class, "B")
```

**A regular expression may guard a type**

- The guard is a standing rule, so it is the assignment that fails, not a later check.

```nuc
class User

if not /.{4,8}/.test($User.password):
    throw 'INVALID_PASSWORD'

user1 = User()

assert(user1.password, null)

try:
    user1.password = 'PAS'
catch error:
    assert(error, "INVALID_PASSWORD")
```

Full detail: [README.md §11](README.md#11-built-in-objects), [NUC 10](nuc-0010.md).

---

## 12. Reasoning Operations

**A live explanation follows rewritten ancestry even when its value stays the same**

- Adding `d` to the relationship changes the explanation before it changes `c`.
- Later changes to `d` update the explanation's values too.

```nuc
a = 1
b = a + 2
c = b * 2
trace = why c
d = 0

b = a + d + 2

assert(trace.length, 4)
assert(trace[0].holds, 6)
assert(trace[1].from, ["a", "d"])

d = 5

assert(trace[0].holds, 16)
```

**An empty impact selection follows new relationships and their removal**

- The source does not have to change for its impact to change.

```nuc
a = 1
exposure = affects a

assert(exposure, [])

b = a + 1
c = b * 2

assert(exposure.map(step => step.node), ["b", "c"])

b = 0

assert(exposure, [])

b = a + 1

assert(exposure.map(step => step.node), ["b", "c"])
```

**An empty model selection follows declarations and deletions but excludes observers**

```nuc
inventory = model |> why

assert(inventory, [])

a = 1
b = a + 1

assert(inventory.map(step => step.node), ["a", "b"])

delete b

assert(inventory.map(step => step.node), ["a"])

b = a + 2
trace = why b

assert(inventory.map(step => step.node), ["a", "b"])
assert(inventory[1].holds, 3)

trace = a + 0

assert(inventory.map(step => step.node), ["a", "b", "trace"])
assert((affects a).map(step => step.node), ["b", "trace"])

trace = why b

assert(inventory.map(step => step.node), ["a", "b"])
assert((affects a).map(step => step.node), ["b"])
```

**An explanation stays subscribed while its property is absent**

```nuc
class Item

item = Item()
trace = why item.amount

assert(trace, [])

item.amount = 3

assert(trace[0].holds, 3)
assert(trace[0].node, "item.amount")

delete item.amount

assert(trace, [])

item.amount = 4

assert(trace[0].holds, 4)
```

**An impact walk stops at reasoning observers**

- A value computed from an explanation is not an indirect model consequence of the explained source.

```nuc
a = 1
b = a + 1
trace = why b
size = trace.length
exposure = affects a

assert(exposure.map(step => step.node), ["b"])

secondTrace = why b
secondSize = secondTrace.length

assert(exposure.map(step => step.node), ["b"])
```

**The value property freezes a selection against structural changes**

```nuc
a = 1
exposure = affects a
frozen = exposure.value

b = a + 1

assert(exposure.map(step => step.node), ["b"])
assert(frozen, [])

a = 2

assert(exposure[0].holds, 3)
assert(frozen, [])
```

**A rejected rewrite restores the explanation and its original subscriptions**

```nuc
a = 1
alternative = 1
b = a + 1
trace = why b
exposure = affects a

try:
    b = alternative + 1
    throw "REJECTED_REWRITE"
catch error:
    assert(error, "REJECTED_REWRITE")

assert(trace[0].from, ["a"])
assert(exposure.map(step => step.node), ["b"])

alternative = 4

assert(trace[0].holds, 2)

a = 3

assert(trace[0].holds, 4)

b = alternative + 1

assert(trace[0].from, ["alternative"])
assert(exposure, [])
```

Full detail: [README.md §12](README.md#12-reasoning-operations), [NUC 11](nuc-0011.md).

---

## 13. Error Reference

**Reading something that was never defined is a reference error**

- The message names what was missing, and is part of the observable behaviour.

```nuc
try:
    t = e + 1
catch error:
    assert(error, ReferenceError("e is not defined"))
```

**An assignment that would close a cycle is refused**

- `number2` already reads `number1`, so `number1` may not be stated in terms of `number2`.

```nuc
number1 = 10
number2 = number1 * 10

assert(number2, 100)

try:
    number1 = number2 * 10
catch error:
    assert(error, TypeError("Circular Dependency"))
```

**value may not be used as a property name**

- The name is reserved for reading a value without depending on it.

```nuc
class Schedule

class Place

value = Schedule()

assert(value, { "id": "value" })

try:
    value.value = Place()
catch error:
    assert(error, TypeError("Cannot use 'value' as a property"))
```

**A class-level rule may not be declared in a non-class block**

```nuc
class Person

person1 = Person()
person1.weight = 90
person1.height = 1.8

try:
    {
        weight = person1.weight
        height = person1.height
        $Person.bmi = weight / (height * height)
    }
catch error:
    assert(error, SyntaxError("Cannot define class declaration in non-class block"))
```

Full detail: [README.md §13](README.md#13-error-reference), [NUC 9](nuc-0009.md).

---

## Single-parameter indented block definitions

**Single parameter block definitions preserve live inputs**

```nuc
fee = 2

def total amount:
    if amount > 0:
        return amount + fee
    return fee

invoice = total(5)

assert(invoice, 7)
assert(total(0), 2)

fee = 3

assert(invoice, 8)
assert(total(0), 3)
```

[NUC 19](nuc-0019.md) specifies this optional spelling for one untyped ordinary
identifier and a real indented colon suite. The same body under
`def total(amount):` remains valid, including both return paths and live
external inputs. Calls and all other accepted signatures retain their syntax
and behavior; inline and brace suites do not gain the bare parameter spelling.

---

## Typed single-parameter named arrows

**Typed single parameter named arrows preserve live inputs**

```nuc
fee = 2

def add_fee amount: int => amount + fee

result = add_fee(5)

assert(result, 7)

fee = 4

assert(result, 9)

def add_fee amount: int => amount * fee

assert(result, 20)
```

[NUC 20](nuc-0020.md) specifies this optional spelling, keeping the existing
`def add_fee(amount: int) => ...` form valid and equivalent. Only immediate
`Identifier`, `Colon`, `Identifier`, `Arrow` tokens after the required name gain
the optional spelling; the annotation is unchanged metadata. Typed bare inline/braced blocks,
other parameter/type forms and logical header breaks remain unsupported.
NUC18/19 spellings, expression bodies, calls and runtime behavior are unchanged.

---

## Reactive numeric clamping

**Numeric clamping bounds live inventory levels**

```nuc
stock = 12
minimum = 0
capacity = 10

usable = Math.clamp(stock, minimum, capacity)

assert(usable, 10)

stock = -3

assert(usable, 0)

minimum = 2

assert(usable, 2)

stock = 7

assert(usable, 7)

capacity = 5

assert(usable, 5)
```

[NUC 21](nuc-0021.md) specifies this static runtime capability; its proposal
header records its status. The result follows ordered min/max coercion,
including numeric `NaN`, equal/reversed bounds, infinities and signed zeros.
Omitted inputs supply numeric `NaN`; extra argument expressions are still
evaluated left to right and included in the existing undefined gate.
See [README.md §11](README.md#11-built-in-objects) for the reference example
and static/member-read limits.

---

## Reactive numeric interpolation

**Numeric interpolation keeps an animated position live**

```nuc
origin = 0
destination = 100
progress = 0.25

coordinate = Math.lerp(origin, destination, progress)

assert(coordinate, 25)

progress = 0.5

assert(coordinate, 50)

destination = 80

assert(coordinate, 40)

origin = 20

assert(coordinate, 50)

progress = 1.5

assert(coordinate, 110)
```

[NUC 22](nuc-0022.md) specifies this static runtime capability; its header
records its status. Intrinsic numeric conversion precedes ordinary subtraction,
multiplication and addition in that order. Weights are not clamped, so this
position extrapolates past the destination. Equal/reversed endpoints, infinities,
overflow, cancellation, numeric `NaN` and signed zeros follow the ordered formula,
not endpoint shortcuts or a fused alternative. Every supplied expression,
including extras, runs once left to right and keeps the existing undefined gate
and live-null/deferred context. See [README.md §11](README.md#11-built-in-objects)
for the reference example, arity and static/member-read limits.

---

## Reactive unit-interval saturation

**Unit-interval saturation bounds live animation opacity**

```nuc
# Unit-interval saturation bounds live animation opacity

class Fade(level):
    this.level = level

$Fade.opacity = Math.saturate($Fade.level)

fade = Fade(0.4)

assert(fade.opacity, 0.4)

fade.level = 2

assert(fade.opacity, 1)

fade.level = -1

assert(fade.opacity, 0)
```

- [NUC 23](nuc-0023.md) specifies static unit-interval saturation.
- The normative [saturation section](../nucleoid.spec.md#reactive-unit-interval-saturation)
  defines ordered `Math.clamp(value, 0, 1)` through the accepted extremum folds.
- Intrinsic conversion, numeric `NaN`, infinities and signed zeros follow that
  composition, not input validation, conversion skips or endpoint shortcuts.
- Every expression, including ignored extras, runs once left to right with
  existing dependencies, undefined/live-null gates and transaction recovery.
- Class-rule property syntax and ordinary lookup are unchanged.
- See [README.md §11](README.md#11-built-in-objects) for the exact executable
  reference example.

---

## Bare typed single-parameter indented block definitions

**Typed block definitions preserve both live return paths**

```nuc
# Typed block definitions preserve both live return paths

fee = 2

def total amount: int:
    if amount > 0:
        return amount + fee
    return fee

invoice = total(5)

assert(invoice, 7)
assert(total(0), 2)

fee = 3

assert(invoice, 8)
assert(total(0), 3)
```

- [NUC 24](nuc-0024.md) describes the optional single typed parameter spelling.
- `def total(amount: int):` with the same suite remains valid and equivalent;
  both return paths are used, and external inputs remain live.
- Require immediate parameter/colon/annotation/colon tokens followed by normal
  body newline layout and a real indent. Blank/comment lines are allowed;
  header-line logical breaks/semicolons, inline/braced bare suites and other
  parameter/type forms are not added.
- Annotations remain identifier metadata, not type constraints or dependencies.
- Calls, NUC18/19/20 forms, all explicit signatures, no implicit block return,
  receivers/init/super, scope limitations, snapshots, rollback and resource
  bounds retain their existing behavior.
- See [README.md §9](README.md#9-functions) for the exact executable reference
  example and the normative
  [typed-block section](../nucleoid.spec.md#bare-typed-single-parameter-indented-block-definitions).

---

## Logical compound assignments

**Logical compound assignments keep their reactive relationships**

```nuc
available = false
fallback = false
available ||= fallback
assert(available, false)
fallback = true
assert(available, true)
allowed = true
allowed &&= available
assert(allowed, true)
available = false
assert(allowed, false)
```

- [Accepted NUC 26](nuc-0026.md) adds optional spellings of the existing expanded
  logical assignments; every paired case passed independent evaluation.
- `available ||= fallback` means `available = available || (fallback)`,
  and `allowed &&= available` means `allowed = allowed && (available)`.
- A standing relationship is always installed, even when its logical evaluation
  skips the right operand. Later dependency updates use the target's current
  value, with no new conditional-write or imperative-update mode.
- Old `=`, `&&`, `||`, `and`, `or`, and arithmetic compounds remain valid.
- The executable reference counterpart is
  `Nucleoid preserves logical compound assignment relationships` in
  `tests/reference.md`; see [README.md §2](README.md#2-variables).

---

## Bare multiple-parameter named arrows

**Multiple parameter named arrows keep live inputs**

```nuc
bias = 1
def total left right => left + right + bias
result = total(2, 3)
assert(result, 6)
bias = 4
assert(result, 9)
```

Under [accepted NUC 27](nuc-0027.md),
`def total(left, right) => left + right + bias` remains valid and exactly
equivalent. Only the complete comma-separated list of at least two untyped
ordinary identifiers immediately followed by `=>` on the same logical line
qualifies. No logical newline, semicolon or comment-induced break is skipped.
Bare inline/braced blocks, defaults, destructuring, rest
parameters, trailing bare commas and return annotations are not added.
Typed/mixed arrow lists have the separate NUC29 spelling below.
Calls and anonymous arrows are unchanged; parenthesized, single-parameter
typed-arrow and typed/untyped single-parameter block alternatives remain valid.

---

## Bare multiple-parameter indented blocks

**Multiple parameter blocks keep live inputs**

```nuc
bias = 1
def total left right =>
    left + right + bias
result = total(2, 3)
assert(result, 6)
bias = 4
assert(result, 9)
```

- [NUC 28](nuc-0028.md) spells the same block as
  `def total(left, right):` with exactly the same body; methods and `init` share
  the same parser. Two or more untyped ordinary identifiers form the complete
  comma-separated header immediately followed by its suite colon.
- Skip no logical newline, semicolon or comment-induced break inside the header.
  The colon must immediately lead to `Newline` or `Indent`, and the first
  non-newline/non-semicolon token from that body break must be a real `Indent`.
  Blank/comment lines and normal suite layout remain valid; `:;` does not qualify.
- NUC30 separately adds typed/mixed multiple indented block headers below;
  NUC28's untyped forms remain unchanged. Inline/braced bare suites, trailing bare
  commas, defaults, destructuring, rest and return annotations remain unsupported.
  Ordinary indented bodies may contain existing branches and inner blocks.
- All parenthesized forms, NUC27 arrows and single typed/untyped guards remain
  valid. Binding, explicit/bare/no returns, noncapturing scope, receivers,
  snapshots, redefinition, graphs and rollback retain the existing block behavior.

---

## Bare typed multiple-parameter named arrows

**Typed multiple parameter named arrows keep live inputs**

```nuc
bias = 1
def total left: int, right => left + right + bias
result = total(2, 3)
assert(result, 6)
bias = 4
assert(result, 9)
```

- [NUC 29](nuc-0029.md) adds an optional spelling of
  `def total(left: int, right) => left + right + bias`, without replacing any
  accepted form.
- Two or more ordinary identifier parameters may mix the existing ordinary
  identifier annotations in any position; at least one annotation distinguishes
  the new forms from NUC27's entirely untyped lists. Metadata and argument
  binding are unchanged, with no runtime type enforcement.
- Require the complete immediate comma-separated sequence followed by `=>`;
  skip no logical newline, semicolon, indentation or comment-induced break.
  NUC29 itself adds no blocks; NUC30's separate typed/mixed indented spelling
  follows below. Bare inline/braced suites, trailing bare commas, new
  parameter/type syntax and call-site omission are not added.
- The named return block, body expressions, methods/init/receivers, external
  dependencies, aliases, snapshots, redefinition and transactions retain their
  parenthesized behavior. NUC28 untyped blocks and every single guard remain
  unchanged; blocks gain no implicit return.
- Full grammar, exact errors and positions are in the authoritative
  [typed multiple-parameter arrow section](../nucleoid.spec.md#bare-typed-multiple-parameter-named-arrows).

---

## Bare typed multiple-parameter indented blocks

**Typed multiple parameter blocks keep live inputs**

```nuc
bias = 1
def total left: int, right =>
    left + right + bias
result = total(2, 3)
assert(result, 6)
bias = 4
assert(result, 9)
```

- [NUC 30](nuc-0030.md) describes the optional spelling of
  `def total(left: int, right):` with exactly the same indented body, including
  methods and `init`.
- The complete immediate list has at least two ordinary identifiers and may
  annotate any position with the existing `Colon Identifier` metadata. At least
  one annotation distinguishes the new forms; all-untyped lists remain NUC28.
  Annotations do not introduce lookups, dependencies or runtime type checks.
- Skip no logical header breaks. Immediately after the suite colon require
  `Newline` or `Indent`, then a real `Indent` after only normal newline/semicolon
  layout; `:;` never qualifies. Blank/comment lines remain suite layout.
- A final untyped parameter's following colon is the suite delimiter. A final
  typed parameter has its annotation colon/name and a separate suite colon.
  The unchanged suite parser owns that final colon and the entire body.
- Keep every accepted parenthesized/single/multiple form, NUC24 typed blocks
  and NUC29 arrows. Inline/braced bare suites, trailing commas, richer
  parameter/type syntax, return annotations and call-site changes are not added.
- Multiple statements, branches, nesting, explicit/bare/no returns, local
  shadowing, noncapturing scope, receivers/init/super, aliases/snapshots, reactive
  inputs, graph dependencies and rollback retain the parenthesized behavior.
- Full grammar and exact diagnostics are in the authoritative
  [typed multiple-parameter block section](../nucleoid.spec.md#bare-typed-multiple-parameter-indented-block-definitions).

---

## Implicit iterator indented loops

**Implicit iterator loops keep one shot updates**

```nuc
class Reading
first = Reading()
first.amount = 2
second = Reading()
second.amount = 5
scale = 2
for Reading:
    it.scaled = it.amount * scale
assert(first.scaled, 4)
scale = 3
assert(first.scaled, 4)
for Reading:
    it.scaled = it.amount * scale
assert(first.scaled, 6)
assert(second.scaled, 15)
```

- Accepted [NUC 31](nuc-0031.md) specifies `for Reading:` as exactly
  `for it of Reading:` with that same genuinely indented body. Existing
  explicit `of`/`in` loops and their arbitrary expressions remain valid.
- The source is exactly one ordinary identifier and is evaluated before local
  `it` is bound. Each loop walks its current snapshot once; later changes to
  `scale` or the class population do not re-run a previous body.
- Lists likewise visit only registered typed instances, preserving order and
  duplicates while skipping primitives, null elements and plain objects.
  Noniterable sources retain their existing errors, without auto-invocation.
- The [NUC 51 candidate](nuc-0051.md) also accepts String snapshots without
  changing List filtering; see [string iteration](#string-iteration).
- Require the immediate colon followed by newline/indent and a real indent
  after only normal newline/semicolon layout; `:;` never qualifies. No
  inline/braced/missing-indent, parenthesized or expression-source shorthand
  is added.
- Nested `it` shadowing/restoration, receivers, return propagation, body
  control flow, graph dependencies, rollback and resource limits are unchanged.
- Full grammar and exact diagnostics are in the authoritative
  [implicit iterator loop section](../nucleoid.spec.md#implicit-iterator-indented-loops).

---

**Indented named arrows keep live inputs**

- The optional NUC32 layout returns exactly one expression through the existing
  named return block; the same colon suite with explicit `return` remains valid.
- External inputs remain live; parameter annotations are unchanged metadata.

```nuc
bias = 1
def total left: int, right =>
    left + right + bias
result = total(2, 3)
assert(result, 6)
bias = 4
assert(result, 9)
```

---

**Smooth cubic transitions preserve clamped endpoints**

```nuc
lower = 0
upper = 8
value = -2
unit = Math.saturate((value - lower) / (upper - lower))
eased = Math.smoothstep(lower, upper, value)
assert(unit, 0)
assert(eased, 0)
value = 2
assert(unit, 0.25)
assert(eased, 0.15625)
value = 6
assert(unit, 0.75)
assert(eased, 0.84375)
value = 10
assert(unit, 1)
assert(eased, 1)
```

- [Accepted NUC 33](nuc-0033.md) defines lower/upper/value argument order,
  independent live normalization controls and the exact ordered cubic.
- Equal or reversed bounds and nonfinite values follow accepted normalization
  and maximum-then-minimum composition, without range validation or shortcuts.

**Indented else suites preserve conditional chains**

```nuc
class Stock
item = Stock()
item.quantity = 0
limit = 5
if item.quantity > limit:
    item.status = "HIGH"
else if item.quantity > 0:
    item.status = "LOW"
else
    item.status = "EMPTY"
assert(item.status, "EMPTY")
item.quantity = 2
assert(item.status, "LOW")
item.quantity = 8
assert(item.status, "HIGH")
limit = 10
assert(item.status, "LOW")
item.quantity = 0
assert(item.status, "EMPTY")
```

- Accepted [NUC 34](nuc-0034.md) makes only a final real-indented `else` colon
  optional; the same source with `else:` remains equivalent.
- This legacy control deliberately retains every condition colon.
  [NUC 49](nuc-0049.md) also permits their omission before genuine indentation,
  with unchanged branch order.

**Smooth quintic transitions preserve clamped endpoints**

```nuc
lower = 0
upper = 8
value = -2
unit = Math.saturate((value - lower) / (upper - lower))
eased = Math.smootherstep(lower, upper, value)
assert(unit, 0)
assert(eased, 0)
value = 2
assert(unit, 0.25)
assert(eased, 0.103515625)
value = 6
assert(unit, 0.75)
assert(eased, 0.896484375)
value = 10
assert(unit, 1)
assert(eased, 1)
```

- Accepted [NUC 35](nuc-0035.md) specifies lower/upper/value normalization and
  the ordered quintic, retaining every independent unit-control assertion.
- The clamped normalization supplies the displayed endpoint results; rounded
  interior polynomial values are not promised to stay in `[0, 1]`.
- Equal/reversed bounds, nonfinite values, coercion and deferred reads follow
  the existing arithmetic composition without validation or shortcuts.

**Floor fractional parts preserve negative and rounded endpoints**

```nuc
value = -2.75
whole = Math.floor(value)
fraction = Math.fract(value)
assert(whole, -3)
assert(fraction, 0.25)
value = 0.5
assert(whole, 0)
assert(fraction, 0.5)
value = 2
assert(whole, 2)
assert(fraction, 0)
value = -0.00000000000000000001
assert(whole, -1)
assert(fraction, 1)
```

- Accepted [NUC 36](nuc-0036.md) adds only static `Math.fract(value)`;
  the existing `value - Math.floor(value)` remains valid and equivalent.
- Convert intrinsically, then compute floor and subtraction in that order.
  The tiny negative endpoint rounds to exactly one without clamping; no strict
  `[0, 1)` guarantee is made. Independent floor controls remain unchanged.
- Nonfinite values, coercion, omitted inputs, ignored-extra reads/effects,
  deferred properties, transactions and recovery retain the existing machinery.

**Comma-free named arrows keep live inputs**

```nuc
bias = 1
def total left right => left + right + bias
result = total(2, 3)
assert(result, 6)
bias = 4
assert(result, 9)
```

- [NUC 37](nuc-0037.md) adds an additional untyped named-arrow
  header guard; the same program with `def total left, right =>` remains
  equivalent. Calls, anonymous arrows, body layout and live dependencies
  retain their existing grammar and behavior.

## Reactive inverse interpolation

**Inverse interpolation handles descending live ranges**

```nuc
start = 10
end = 2
sample = 8
weight = Math.inverseLerp(start, end, sample)
assert(weight, 0.25)
sample = 4
assert(weight, 0.75)
sample = 14
assert(weight, -0.5)
start = 18
assert(weight, 0.25)
```

- Accepted [NUC 38](nuc-0038.md) adds only unshadowed static
  `Math.inverseLerp(start, end, value)`. The accepted arithmetic
  `(value - start) / (end - start)` remains available and equivalent.
- Convert intrinsically, then compute numerator subtraction, span subtraction
  and division in exactly that order. Do not clamp, reorder ranges, shortcut
  endpoints or scale to avoid overflow. Descending ranges are valid.
- Nonfinite results, missing inputs, ignored-extra effects/reads, deferred
  properties and transactional recovery retain the existing machinery.
- The authoritative
  [inverse-interpolation section](../nucleoid.spec.md#reactive-inverse-interpolation)
  and the [reference](README.md#reactive-inverse-interpolation) give the full contract.

## Reactive range remapping

**Range remapping handles descending live source and target ranges**

```nuc
start = 10
end = 2
targetStart = 30
targetEnd = 10
sample = 8
mapped = Math.remap(start, end, targetStart, targetEnd, sample)
assert(mapped, 25)
sample = 4
assert(mapped, 15)
sample = 14
assert(mapped, 40)
start = 18
assert(mapped, 25)
```

- Accepted [NUC 39](nuc-0039.md) adds only unshadowed static
  `Math.remap(start, end, targetStart, targetEnd, value)`. The accepted
  `Math.lerp(targetStart, targetEnd, Math.inverseLerp(start, end, value))`
  remains available for the same numeric composition.
- Convert intrinsically, then subtract, subtract, divide, subtract, multiply
  and add, with separate rounding in exactly that order. Do not clamp, reorder
  ranges, rescale or shortcut endpoints; descending ranges and extrapolation
  are intentional.
- New-call expressions run once in listed order, unlike the nested spelling's
  target-first argument order. The sealed demonstrations use pure reads.
- Deferred inputs, ignored-extra reads/effects, strict failures and rollback
  retain the existing machinery. The authoritative
  [remapping section](../nucleoid.spec.md#reactive-range-remapping) and
  [reference](README.md#reactive-range-remapping) define the full contract.

## Comma-free bare indented blocks

**Comma-free blocks keep live input relationships**

```nuc
bias = 1
def combined left right:
    total = left + right
    return total + bias
left = 2
right = 3
reading = combined(left, right)
assert(reading, 6)
bias = 4
assert(reading, 9)
left = 5
assert(reading, 12)
```

- [NUC 40](nuc-0040.md) adds an optional wholly untyped bare block
  header with at least two names and no parameter commas. The final colon
  and genuine indentation remain mandatory; this is a statement suite,
  not an arrow expression or an implicit-return form.
- `def combined left, right:` and `def combined(left, right):` with this
  exact body remain valid. Body, call, list, object and anonymous-function
  punctuation is unchanged; typed/mixed lists still keep their commas.
- All other grammar, graph relationships and transactional behavior follow
  the [authoritative section](../nucleoid.spec.md#comma-free-bare-indented-blocks).

## Reactive flat mapping

**Flattened batches retain live object identities**

```nuc
class Task(label):
    this.label = label
task1 = Task("alpha")
task2 = Task("beta")
batches = [[task1], [task2, task1]]
flattened = batches.flatMap(batch => batch)
labels = flattened.map(task => task.label)
assert(labels, ["alpha", "beta", "alpha"])
task1.label = "gamma"
assert(labels, ["gamma", "beta", "gamma"])
first = flattened[0]
first.label = "delta"
assert(task1.label, "delta")
assert(labels, ["delta", "beta", "delta"])
task3 = Task("third")
batches.push([task3])
assert(labels, ["delta", "beta", "delta", "third"])
assert(batches.length, 3)
```

- Accepted [NUC 41](nuc-0041.md) adds the same one-level query to lists and
  class populations. It preserves duplicates, order and identities without
  mutating the receiver or recursively flattening deeper lists.
- The callback receives one snapshotted item, just as for `map`. Real reads,
  member updates, supplied-argument effects and errors retain their existing
  reactive and transactional behavior. Returning null alone does not imply
  the null read made by an explicit intermediate reducer.
- Existing map/reduce/concat programs remain valid. See the root
  [contract](../nucleoid.spec.md#reactive-flat-mapping) and the
  [reference](README.md#reactive-flat-mapping) for details.

## Reactive literal string replacement

**Literal replacement preserves frozen text normalizers**

```nuc
text = "archive::notes::today"
def normalize(value) => value.replaceAll("::", "/")
frozen = normalize.value
live = normalize(text)
snapshot = frozen(text)
assert(live, "archive/notes/today")
assert(snapshot, "archive/notes/today")
def normalize(value) => value.replaceAll("::", "-")
assert(live, "archive-notes-today")
assert(snapshot, "archive/notes/today")
text = "archive::notes"
assert(live, "archive-notes")
assert(snapshot, "archive/notes")
```

- Accepted [NUC 42](nuc-0042.md) adds literal all-occurrences string replacement.
  The function definition may be live or frozen without freezing its inputs.
- The method never mutates its receiver. Existing `replace` remains first-only;
  supplied arguments, actual reads and transactions keep their existing rules.
- See the [root contract](../nucleoid.spec.md#reactive-literal-string-replacement)
  and [reference](README.md#reactive-literal-string-replacement) for coercion,
  Unicode boundaries and inherited null/deferred behavior.

## Direct conditional alternatives

**Conditional alternatives keep frozen classifiers and live inputs separate**

```nuc
def classify(state, urgent):
    if state == "blocked"
        return "hold"
    else urgent
        return "fast"
    elif state == "ready"
        return "send"
    else:
        return "wait"
state = "ready"
urgent = false
live = classify(state, urgent)
frozen = classify.value
usingFrozen = frozen(state, urgent)
snapshot = usingFrozen.value
assert(live, "send")
assert(usingFrozen, "send")
urgent = true
assert(live, "fast")
assert(usingFrozen, "fast")
def classify(state, urgent):
    if state == "blocked"
        return "pause"
    else urgent
        return "rush"
    elif state == "ready"
        return "go"
    else:
        return "queue"
assert(live, "rush")
assert(usingFrozen, "fast")
assert(snapshot, "send")
state = "blocked"
assert(live, "pause")
assert(usingFrozen, "hold")
```

- [NUC 43](nuc-0043.md) adds only an optional spelling for genuine
  indented conditional alternatives. `else condition:` and `elif condition:`
  retain the complete AST and behavior of `else if condition:`.
- [NUC 49](nuc-0049.md) permits the additional colon-free spellings shown
  here, including initial and explicit else-if conditions, only at genuine
  indentation. Colon-bearing, mixed and legacy inline/braced suites remain
  valid wherever previously supported.
- Function definitions may be live or frozen without freezing their inputs.
  Ordinary noncapture, explicit returns, receiver scopes and transaction
  failures are unchanged; no receiver-frame repair is included.
- Explicit `else if`, unconditional `else:`/`elif:` and their accepted
  colon-free indented final forms remain supported. See the
  [root contract](../nucleoid.spec.md#colon-free-indented-conditional-suites) and
  [reference](README.md#conditionals) for layout and exact errors.

## Comma-free explicit object properties

**Compact records keep class rules independent**

```nuc
# Compact records keep class rules independent

class Ticket
first = Ticket()
second = Ticket()
first.amount = 2
second.amount = 5
$Ticket.payload = {reading: $Ticket.amount category: "sample" ready: true}
assert(first.payload.reading, 2)
assert(second.payload.reading, 5)
first.amount = 7
assert(first.payload.reading, 7)
assert(second.payload.reading, 5)
assert(first.payload.ready, true)
```

- [NUC 44](nuc-0044.md) uses the same ordered explicit
  properties and per-instance receiver reads as the comma-bearing spelling.
- Only a key/colon pair after a fully parsed value can omit its comma.
  Shorthand qualification, list/call commas, class rules, aliases, `.value`
  snapshots and transaction failures remain unchanged.
- The [root contract](../nucleoid.spec.md#comma-free-explicit-object-properties)
  and [expression guidance](README.md#comma-free-explicit-properties)
  specify the bounded syntax. Comma-bearing forms remain supported.

## Colon-led assertion statements (Accepted NUC 45)

**Compact assertions verify class rules and snapshots**

```nuc
# Compact assertions verify class rules and snapshots

class Sensor
first = Sensor()
second = Sensor()
first.reading = 2
second.reading = 8
$Sensor.active = $Sensor.reading > 5
saved = second.active.value
assert: first.active, false
assert: second.active, true
second.reading = 1
assert: second.active, false
assert: saved, true
saved
# return: true
```

- Accepted [NUC 45](nuc-0045.md) lowers these statement-position checks
  exactly to `assert(actual, expected)`, which remains supported.
- Assertions do not install model rules or observers and do not replace the
  preceding program result. Class rules, `.value` snapshots and ordinary
  expression arguments retain their existing behavior.
- Only lowercase `assert` followed by `:` in statement position qualifies;
  existing typed declarations win before this new spelling is considered.

**Compact exception suites preserve independent class rules**

```nuc
# Compact exception suites preserve independent class rules

class Task(state: str):
    this.state = state
first = Task("open")
second = Task("open")
$Task.closed = $Task.state == "closed"
try
    first.state = "closed"
    throw "CANCEL"
catch issue
    assert(issue, "CANCEL")
assert(first.closed, false)
assert(second.closed, false)
first.state = "closed"
assert(first.closed, true)
assert(second.closed, false)
first.closed
# return: true
```

- Accepted [NUC 46](nuc-0046.md) changes only the optional suite colons.
  The same bodies with `try:` and `catch issue:` retain the complete AST,
  actual checks, explicit return, rollback and independent reactive rules.
- A real indent must qualify each omitted colon separately; inline/braced
  forms keep their existing required colons. The native REPL buffers a complete
  lexical bare `try` header through its catch until blank-line commit or EOF.

**Reasoning state filters separate supporting facts from derived totals**

```nuc
quantity = 3
subtotal = quantity * 10
total = subtotal + 2
facts = total |> why |> stated
conclusions = total |> why |> derived
assert(facts.map(step => step.node), ["quantity"])
assert(conclusions.map(step => step.node), ["total", "subtotal"])
quantity = 4
assert(facts[0].holds, 4)
assert(conclusions[0].holds, 42)
```

- The [NUC 48 candidate](nuc-0048.md) filters the explanation's existing
  step states without changing their order or live subscriptions.
- The accepted `.filter(step => step.state == "stated")` and
  `.filter(step => step.state == "derived")` spellings remain supported.
- Prefix `stated total` filters only `total` itself; it does not include
  supporting ancestors unless a preceding `why` selects them.

**Radian conversion keeps independent headings reactive**

```nuc
heading = 180
target = 90
radians = Math.toRadians(heading)
targetRadians = Math.toRadians(target)
assert(radians, Math.PI)
assert(targetRadians, Math.PI / 2)
heading = 360
assert(radians, Math.PI * 2)
assert(targetRadians, Math.PI / 2)
target = -90
assert(targetRadians, -Math.PI / 2)
```

- The [NUC 50 candidate](nuc-0050.md) intrinsically converts the first input
  and multiplies it by `PI / 180.0`, with division performed first.
  Finite maxima, signed zeros, subnormals and nonfinite inputs retain that
  exact composition; NaNs are classified without a payload guarantee.
- Omission supplies numeric NaN. Supplied Undefined, including an ignored
  extra, retains existing deferral. All expression effects/errors and namespace
  shadowing remain unchanged. The accepted arithmetic remains available;
  the [root specification](../nucleoid.spec.md#reactive-degree-to-radian-conversion)
  defines the complete contract.

## String iteration

**String iteration keeps independent receiver data separate**

```nuc
class Message(text):
    this.text = text
first = Message("ab")
second = Message("CD")
seen = []
for character of first.text:
    seen.push(character)
for character of second.text:
    seen.push(character)
assert(seen, ["a", "b", "C", "D"])
first.text = "xy"
assert(seen, ["a", "b", "C", "D"])
seen = []
for character of first.text:
    seen.push(character)
assert(seen, ["x", "y"])
assert(second.text, "CD")
```

- Each existing explicit `of`/`in` or implicit-`it` For statement evaluates
  its source once and snapshots one String per Unicode scalar.
- Later property updates do not replay the imperative body. Nested iteration
  bindings, enclosing `this`/`self`, early returns and transactional failures
  use the ordinary For machinery, not callbacks or a new iterator protocol.
- Empty Strings run no body. Supplementary characters stay whole, while
  combining marks and joiners remain separate scalars without normalization.
  Class populations and object-only List traversal are unchanged.
- Split/map remains available; its discarded map results are not a For
  statement's return value. The [root specification](../nucleoid.spec.md#string-iteration)
  defines the candidate contract and preserves legacy syntax.

## Read-only state versions

**Reactive property versions remain independent between instances**

```nuc
class Meter(seed):
    this.seed = seed
$Meter.total = $Meter.seed * 2
first = Meter(2)
second = Meter(3)
first.seed = 4
firstOriginal = first.total@first
firstLatest = first.total@last
firstPrevious = first.total@previous
secondOriginal = second.total@first
secondLatest = second.total@last
secondPrevious = second.total@previous
assert(firstOriginal, 4)
assert(firstLatest, 8)
assert(firstPrevious, 4)
assert(secondOriginal, 6)
assert(secondLatest, 6)
assert(secondPrevious, null)
```

- Each actual property slot retains its initial value and subsequent changes.
  Selection tracks the real owner identity; a constructor or class rule does
  not share one textual history between instances.
- A missing previous version defers to Null and stays live for recovery.
  No manual push, mirror property, wrapper, negative selector or deep object
  snapshot is needed. See [NUC 53](nuc-0053.md) and the
  [root contract](../nucleoid.spec.md#read-only-state-versions).

## Literal parameter defaults

**Defaulted calibration remains reactive to supplied arguments**

```nuc
def calibrate(raw, offset = 2) => raw + offset
sensor = 5
one = calibrate(sensor)
two = calibrate(7)
three = calibrate(9)
untouched = calibrate(2, 0)
assert(one, 7)
assert(two, 9)
assert(three, 11)
assert(untouched, 2)
sensor = 6
assert(one, 8)
```

- An omitted slot uses its scalar default; the supplied zero above remains
  zero. Reactive dependencies on supplied arguments still update callers.
- Parenthesized functions, methods and constructors use fresh effective
  parameter values before initializing their local history. No expression
  defaults, capture or bare-default syntax is introduced. See the
  [root contract](../nucleoid.spec.md#literal-parameter-defaults).

## Class-level state version anchors

**Inherited class receivers keep independent fixed history anchors**

```nuc
class Base(seed):
    this.x = seed
    this.orientation = 180
class Child: Base
    def init(seed):
        super(seed)
step = 1
$Base.x = $Base.x@0 - step
if $Base.orientation == 180:
    $Base.tag = "l4214"
    $Base.x = $Base.x@first - step
parent = Base(10)
child = Child(30)
assert([parent.x, child.x], [9, 29])
assert([parent.tag, child.tag], ["l4214", "l4214"])
assert([parent.x@first, child.x@first], [10, 30])
step = 2
assert([parent.x, child.x], [8, 28])
child.orientation = 0
child.orientation = 180
assert(child.x, 28)
assert(child.x@3, null)
```

- This is the appended inherited-anchor specification example. Each concrete
  property retains its first value even when a parent rule applies to a child.
  A guard re-evaluation repeats the anchored result, not another decrement.
- Non-self histories still observe real slots, including reset/recreation.
  Missing fixed self-entries do not seed themselves. Moving `@last`/`@previous`
  conditionals retain accepted imperative effects, while direct moving
  self-rules still raise `TypeError("Circular Dependency")`. See Draft NUC 55
  and the root **Class-level state version anchors** contract in
  `nucleoid.spec.md`.

## Reactive base two exponentiation

**Reactive power of two scaling**

```nuc
power = -1
amount = 80
factor = Math.exp2(power)
scaled = amount * factor
assert(scaled, 40)
amount = 20
assert(scaled, 10)
power = -2
assert(scaled, 5)
power = 2
assert(scaled, 80)
scaled
# return: 80
```

- Accepted [NUC 56](nuc-0056.md) provides the named base-two helper without
  changing accepted `Math.pow(2, power)`. The factor and scaled amount remain
  live; see the [root contract](../nucleoid.spec.md#reactive-base-two-exponentiation)
  for intrinsic conversion, ignored extras, deferral and numeric edge cases.

## Thin arrows and Python-style lambdas

**Thin arrows and lambdas chain live callbacks**

```nuc
threshold = 10
readings = [4, 12, 18]
alerts = readings.filter(reading -> reading > threshold)
labels = alerts.map(lambda reading: `R${reading}`)
assert(labels, ["R12", "R18"])
threshold = 15
assert(labels, ["R18"])
labels
# return: ["R18"]
```

- [NUC 57](nuc-0057.md) spells the same anonymous functions as
  `reading => reading > threshold` and a parenthesized `(reading) =>` arrow
  around the template body; the callbacks, their dependencies and the
  canonical `=>` arrow are unchanged.
  See the [root contract](../nucleoid.spec.md#thin-arrows-and-python-style-lambdas)
  for every accepted position and the places where `lambda` stays a name.

## Conditional expressions

**Conditional expressions grade live class records**

```nuc
class Student(name, score):
    this.name = name
    this.score = score
$Student.grade = $Student.score >= 90 ? "A" : $Student.score >= 75 ? "B" : "C"
ana = Student("Ana", 92)
ben = Student("Ben", 70)
honors = Student.filter(student => student.grade == "A").map(student => student.name)
assert(ben.grade, "C")
assert(honors, ["Ana"])
ben.score = 80
assert(ben.grade, "B")
ben.score = 95
assert(ben.grade, "A")
assert(honors, ["Ana", "Ben"])
honors
# return: ["Ana","Ben"]
```

- One class-level rule chains two conditionals to the right, in place of a
  class-level `if`/`elif`/`else` statement. Each student evaluates only its
  selected branch and is graded again when its score changes; the honors
  query follows the grades.
- See [NUC 59](nuc-0059.md) and the
  [root contract](../nucleoid.spec.md#conditional-expressions).

## Nullish coalescing

**Nullish coalescing fills missing fields of live class records**

```nuc
class Product(name, price):
    this.name = name
    this.price = price
$Product.shown = $Product.price ?? "ask"
$Product.stock = $Product.count ?? 0
tea = Product("Tea", 4)
herb = Product("Herb", null)
labels = Product.map(product => `${product.name}: ${product.shown}`)
assert(herb.shown, "ask")
assert(herb.stock, 0)
assert(labels, ["Tea: 4", "Herb: ask"])
herb.price = 0
assert(herb.shown, 0)
assert(labels, ["Tea: 4", "Herb: 0"])
herb.count = 3
assert(herb.stock, 3)
labels
# return: ["Tea: 4","Herb: 0"]
```

- Two class-level rules default a missing price and stock count. Each product
  evaluates its own operands, `0` is kept as a real price, and the labels
  follow every change; an `||` default would store `null` for the null price.
- See [NUC 60](nuc-0060.md) and the
  [root contract](../nucleoid.spec.md#nullish-coalescing).

## Instance list ranges and list queries

**Live leaderboards read instance lists by range and position**

```nuc
class Runner(name, time):
    this.name = name
    this.time = time
ana = Runner("Ana", 61)
ben = Runner("Ben", 58)
cy = Runner("Cy", 64)
cutoff = 62
latest = Runner.at(-1).name
qualified = Runner.list().filter(runner => runner.time < cutoff).map(runner => runner.name)
lastSlow = Runner.findLastIndex(runner => runner.time >= cutoff)
leaders = Runner[:2].map(runner => runner.name)
assert(latest, "Cy")
assert(qualified, ["Ana", "Ben"])
assert(lastSlow, 2)
assert(leaders, ["Ana", "Ben"])
cy.time = 60
assert(qualified, ["Ana", "Ben", "Cy"])
assert(lastSlow, -1)
dee = Runner("Dee", 63)
assert(latest, "Dee")
assert(lastSlow, 3)
assert(leaders, ["Ana", "Ben"])
qualified
# return: ["Ana","Ben","Cy"]
```

- `Runner.at(-1)` follows the newest runner, `Runner[:2]` keeps the first two
  in population order, and `Runner.findLastIndex(...)` searches from the end.
  Each declaration re-evaluates when a runner is created or a time changes.
- See [NUC 61](nuc-0061.md) and the
  [root contract](../nucleoid.spec.md#instance-list-ranges-and-list-queries).

## Quantifiers

**Quantifiers keep zone health true to every sensor**

```nuc
class Zone(name):
    this.name = name
class Sensor(zone, ok):
    this.zone = zone
    this.ok = ok
$Zone.healthy = all s in Sensor: s.zone != $Zone.name or s.ok
lab = Zone("lab")
dock = Zone("dock")
probe = Sensor("lab", true)
door = Sensor("dock", false)
alerts = Zone.filter(zone => not zone.healthy).map(zone => zone.name)
assert(lab.healthy, true)
assert(dock.healthy, false)
assert(alerts, ["dock"])
door.ok = true
assert(dock.healthy, true)
assert(alerts, [])
Sensor("lab", false)
assert(lab.healthy, false)
alerts
# return: ["lab"]
```

- The class-level rule states one claim per zone: every sensor is either in
  another zone or reports `ok`. Inside the condition, `$Zone.name` is the zone
  the rule is evaluated for, and `s` is the sensor being tested.
- Creating a sensor or changing one re-evaluates every zone's `healthy`, and
  `alerts` follows. The accepted spelling
  `Sensor.every(s => s.zone != $Zone.name or s.ok)` keeps working and gives
  the same results.
- See [NUC 62](nuc-0062.md) and the
  [root contract](../nucleoid.spec.md#quantifiers).

## Exponentiation

**Powers keep a kinetic energy model live**

```nuc
class Body(mass, speed):
    this.mass = mass
    this.speed = speed
$Body.energy = 0.5 * $Body.mass * $Body.speed ** 2
cart = Body(2, 3)
ball = Body(0.5, 4)
total = Body.reduce((sum, body) -> sum + body.energy, 0)
assert(cart.energy, 9)
assert(ball.energy, 4)
assert(total, 13)
cart.speed = 5
assert(cart.energy, 25)
assert(total, 29)
total
# return: 29
```

- The class-level rule states kinetic energy once for every body, and
  `$Body.speed ** 2` squares each body's own speed. Changing a speed
  re-evaluates that body's energy and the total that sums them.
- `$Body.speed ** 2` gives exactly `Math.pow($Body.speed, 2)`. Prefix
  operators apply to the whole power, so `-$Body.speed ** 2` is negative and
  `(-$Body.speed) ** 2` squares the negated speed.
- See [NUC 64](nuc-0064.md) and the
  [root contract](../nucleoid.spec.md#exponentiation).

## Hypotheticals

**Suppose blocks test restocking plans without committing them**

```nuc
class Item(name, stock, demand):
    this.name = name
    this.stock = stock
    this.demand = demand
$Item.short = $Item.stock < $Item.demand
shortages = Item.filter(item -> item.short).map(item -> item.name)
bolts = Item("bolts", 40, 50)
nuts = Item("nuts", 80, 30)
restock = suppose bolts.stock += 20: shortages
surge = suppose nuts.demand = 100: shortages
assert: restock, []
assert: surge, ["bolts", "nuts"]
assert: shortages, ["bolts"]
assert: bolts.stock, 40
nuts.stock = 20
shortages
# return: ["bolts","nuts"]
```

- Each `suppose` block applies one plan to the live model: the class-level
  rule re-derives `short` for every item, and `shortages` follows. The block
  keeps the resulting list as a snapshot and then rolls the plan back, so
  `bolts.stock` is still `40` afterwards.
- The snapshots do not follow later changes: after `nuts.stock = 20`,
  `shortages` lists both items while `restock` stays empty. See
  [NUC 65](nuc-0065.md) and the
  [root contract](../nucleoid.spec.md#hypotheticals).

## Value assignment

**Value assignments lock deposit rates while interest stays live**

```nuc
rate = 2
class Deposit(amount):
    this.amount = amount
    this.rate := rate
$Deposit.interest = $Deposit.amount * $Deposit.rate / 100
total = Deposit.reduce(lambda sum, deposit: sum + deposit.interest, 0)
early = Deposit(100)
rate = 3
late = Deposit(100)
opening := total
early.amount = 300
assert: early.interest, 6
assert: late.interest, 3
assert: total, 9
assert: opening, 5
Deposit.map(deposit -> deposit.rate)
# return: [2,3]
```

- Each deposit locks in the rate that holds when it opens. `this.rate := rate`
  runs once in the constructor and records no dependency on `rate`, so the
  later `rate = 3` reaches only `late`.
- `interest` and `total` stay live declarations, so raising `early.amount`
  updates both, while `opening := total` keeps the total from the moment it
  was taken. See [NUC 66](nuc-0066.md) and the
  [root contract](../nucleoid.spec.md#value-assignment).

## List and population sums

**Sums keep room and building power loads live**

```nuc
class Room(name, capacity):
    this.name = name
    this.capacity = capacity
class Device(room, watts, on):
    this.room = room
    this.watts = watts
    this.on = on
$Room.load = Device.filter(device -> device.room == $Room and device.on).sum(device -> device.watts)
$Room.tripped = $Room.load > $Room.capacity
building = Room.sum(room -> room.load)
kitchen = Room("kitchen", 3000)
office = Room("office", 1000)
kettle = Device(kitchen, 2000, true)
oven = Device(kitchen, 1500, false)
Device(office, 400, true)
assert(kitchen.load, 2000)
assert(building, 2400)
oven.on = true
assert(kitchen.tripped, true)
assert(building, 3900)
kettle.on = false
garage = Room("garage", 2000)
Device(garage, 700, true)
[kitchen.load, kitchen.tripped, building]
# return: [1500,false,2600]
```

- The class-level rule states each room's load once: it filters the devices
  that are in the room and switched on, and sums their watts. `building` sums
  the load of every room, so it follows a device switching on or off and a
  room being added.
- `$Room.tripped` compares each room's load with its capacity. Switching the
  oven on trips the kitchen; switching the kettle off brings it back to
  `1500` watts. The accepted spelling
  `Device.filter(...).reduce((sum, device) -> sum + device.watts, 0)` keeps
  working and gives the same results.
- See [NUC 68](nuc-0068.md) and the
  [root contract](../nucleoid.spec.md#list-and-population-sums).

## List and population extremes

**Extremes keep room temperature ranges live**

```nuc
class Room(name, limit):
    this.name = name
    this.limit = limit
class Sensor(room, temp, on):
    this.room = room
    this.temp = temp
    this.on = on
$Room.high = Sensor.filter(sensor -> sensor.room == $Room and sensor.on).max(sensor -> sensor.temp)
$Room.low = Sensor.filter(sensor -> sensor.room == $Room and sensor.on).min(sensor -> sensor.temp)
$Room.uneven = $Room.high - $Room.low > $Room.limit
hottest = Room.max(room -> room.high)
lab = Room("lab", 4)
den = Room("den", 3)
Sensor(lab, 22, true)
door = Sensor(lab, 19, true)
vent = Sensor(den, 20, false)
assert(den.high, Number.NEGATIVE_INFINITY)
assert(lab.uneven, false)
assert(hottest, 22)
door.temp = 16
assert(lab.uneven, true)
vent.on = true
Sensor(den, 27, true)
assert(den.uneven, true)
assert(hottest, 27)
[lab.low, den.high, hottest]
# return: [16,27,27]
```

- The class-level rules state each room's highest and lowest reading once:
  they filter the sensors that are in the room and switched on, then take
  the `max` and `min` of their temperatures. `uneven` compares the spread
  with the room's limit, and `hottest` takes the highest reading over every
  room.
- Until the vent is switched on, the den has no active sensor, so its `high`
  is `-Infinity`, the value of `Math.max()`. Cooling the door sensor to `16`
  makes the lab uneven; switching the vent on and adding a `27` reading does
  the same for the den and raises `hottest`. The accepted spelling
  `Sensor.filter(...).reduce((high, sensor) -> Math.max(high, sensor.temp), -1 / 0)`
  keeps working and gives the same results.
- See [NUC 69](nuc-0069.md) and the
  [root contract](../nucleoid.spec.md#list-and-population-extremes).

## List and population distinct values

**Distinct values keep the categories stocked in each warehouse live**

```nuc
class Warehouse(name, minimum):
    this.name = name
    this.minimum = minimum
class Item(warehouse, category, qty):
    this.warehouse = warehouse
    this.category = category
    this.qty = qty
$Warehouse.categories = Item.filter(item -> item.warehouse == $Warehouse and item.qty > 0).distinct(item -> item.category)
$Warehouse.narrow = $Warehouse.categories.length < $Warehouse.minimum
stocked = Item.filter(item -> item.qty > 0).distinct(item -> item.category)
north = Warehouse("north", 2)
south = Warehouse("south", 1)
pens = Item(north, "office", 40)
Item(north, "office", 15)
Item(south, "garden", 8)
assert(north.categories, ["office"])
assert(north.narrow, true)
assert(stocked, ["office", "garden"])
Item(north, "garden", 5)
pens.qty = 0
assert(north.categories, ["office", "garden"])
assert(north.narrow, false)
spades = Item(south, "tools", 0)
spades.qty = 3
[north.categories, south.categories, stocked]
# return: [["office","garden"],["garden","tools"],["office","garden","tools"]]
```

- The class-level rule states each warehouse's categories once: it filters
  the items that are in the warehouse and in stock, and lists their
  categories without repeats, in the order of the first item of each.
  `narrow` compares the count with the warehouse's minimum, and `stocked`
  lists the categories in stock anywhere.
- Emptying the pens keeps `office`, because the second office item is still
  in stock, and a garden item makes the north warehouse varied enough;
  restocking the spades adds `tools` to the south warehouse and to
  `stocked`. The accepted spelling
  `Item.filter(...).reduce((kinds, item) -> kinds.includes(item.category) ? kinds : kinds.concat([item.category]), [])`
  keeps working and gives the same results.
- See [NUC 71](nuc-0071.md) and the
  [root contract](../nucleoid.spec.md#list-and-population-distinct-values).

## Object entry loops

**Entry loops stock warehouses from delivery manifests**

```nuc
class Product(site, name, qty):
    this.site = site
    this.name = name
    this.qty = qty
$Product.low = $Product.qty < 10
class Warehouse(site):
    this.site = site
    this.units = 0
    def receive(manifest):
        for name, qty of manifest:
            Product(this.site, name, qty)
            this.units += qty
        return this.units
north = Warehouse("north")
south = Warehouse("south")
assert([north.receive({bolts: 40, nuts: 25, gears: 8}), south.receive({belts: 3, bolts: 12}), south.receive({nuts: 5, gears: 8, springs: 30})], [73, 15, 58])
Product.filter(product -> product.low).map(product -> product.site + ":" + product.name)
# return: ["north:gears","south:belts","south:nuts","south:gears"]
```

- `receive` turns a delivery manifest into products for the warehouse that
  received it and adds each quantity to the warehouse's total, in the order
  the manifest lists its entries. The loop body reads `this` and the entry's
  name and quantity, as a single-name loop body reads its item.
- The three deliveries list different products, and the one method reads
  each without naming them. The class-level rule marks every product under
  ten units as low, so the result lists the low products in the order they
  arrived: the north gears, then the south belts, nuts and gears. Each
  manifest is snapshotted when its loop starts, and the loop runs once; it is
  not a standing rule.
- See [NUC 72](nuc-0072.md) and the
  [root contract](../nucleoid.spec.md#object-entry-loops-with-ordered-rollback).

## While loops

**While loops run packing shifts until every order is done**

```nuc
class Order(qty):
    this.qty = qty
    this.packed = 0
$Order.done = $Order.packed >= $Order.qty
Order(3)
Order(5)
Order(2)
open = Order.filter(o => not o.done).length
shifts = 0
while open > 0:
    for order of Order:
        if not order.done:
            order.packed = order.packed + 2
    shifts = shifts + 1
assert(shifts, 3)
assert(open, 0)
assert(Order.map(o => o.packed), [4, 6, 2])
shifts
# return: 3
```

- The class-level rule keeps `done` true to each order's packed count, and
  `open` counts the orders that are not done. Each shift packs two units for
  every open order, and the loop tests `open` again after every shift, so it
  stops as soon as the last order is done.
- The nested `for` and `if` are part of the one-shot loop body, not standing
  rules. In accepted syntax the shifts need a `for` loop over placeholder
  instances, and their number must be known in advance.
- See [NUC 63](nuc-0063.md) and the
  [root contract](../nucleoid.spec.md#while-loops).

## Destructuring

**Destructuring keeps a rover heading model live**

```nuc
class Rover(name, position, heading):
    this.name = name
    this.position = position
    this.heading = heading
rover = Rover("scout", {x: 2, y: 3}, [1, 0])
{x, y} = rover.position
dx, dy = rover.heading
ahead = {x: x + dx, y: y + dy}
assert: ahead, {x: 3, y: 3}
rover.heading = [0, -1]
assert: ahead, {x: 2, y: 2}
rover.position = {x: 5, y: 5}
assert: [x, y, dx, dy], [5, 5, 0, -1]
ahead
# return: {"x":5,"y":4}
```

- `{x, y} = rover.position` and `dx, dy = rover.heading` declare one live
  name per field and per position, exactly as `x = rover.position.x` and
  `dx = rover.heading[0]` would.
- Turning the rover or moving it updates `x`, `y`, `dx`, `dy` and `ahead`
  together. See [NUC 67](nuc-0067.md) and the
  [root contract](../nucleoid.spec.md#destructuring-declarations).

## Predicate counts

**Predicate counts keep the unfinished tasks of each team live**

```nuc
class Team(name): this.name = name
class Task(team, done):
    this.team = team
    this.done = done
$Team.pending = Task.count(task -> task.team == $Team and not task.done)
north = Team("north")
south = Team("south")
first = Task(north, false)
second = Task(north, true)
Task(south, false)
assert: north.pending, 1
assert: south.pending, 1
second.done = false
assert: north.pending, 2
first.team = south
assert: north.pending, 1
assert: south.pending, 2
```

- The one class-level rule counts each team's unfinished tasks. Completing
  or transferring a task updates every affected team's count.
- `count(predicate)` keeps the invocation, dependency and missing-value
  behavior of `filter(predicate).length` without allocating its result list.
  It does not replace instance methods named `count`.
- See [NUC 73](nuc-0073.md) and the
  [root contract](../nucleoid.spec.md#list-and-population-predicate-counts).

## Final rest parameters

**Variadic totals keep ordinary argument dependencies live**

```nuc
def total(head = 0, ...tail) => head + tail.sum()
reading = 2
amount = total(1, reading, 3)
assert(amount, 6)
assert(total(), 0)
reading = 5
assert(amount, 9)
assert(total(4), 4)
amount
# return: 9
```

- Each invocation starts a fresh tail list and keeps ordinary fixed defaults
  and argument dependencies. Changing the supplied reading re-evaluates the
  standing call; it does not mutate a retained parameter frame.
- Only the final parenthesized formal may use `...`; rest defaults and
  call-site spread are not supported. See [NUC 74](nuc-0074.md) and the
  [root contract](../nucleoid.spec.md#final-rest-parameters).

## List and population modes

**Modes keep the most common defect of each production line live**

```nuc
class Line(name): this.name = name
class Defect(line, kind):
    this.line = line
    this.kind = kind
$Line.common = Defect.filter(defect -> defect.line == $Line).mode(defect -> defect.kind)
worst = Defect.mode(defect -> defect.kind)
press = Line("press")
paint = Line("paint")
Defect(press, "dent")
crack = Defect(press, "crack")
Defect(press, "crack")
Defect(paint, "drip")
assert: [press.common, paint.common, worst], ["crack", "drip", "crack"]
crack.kind = "dent"
assert: [press.common, worst], ["dent", "dent"]
crack.line = paint
assert: [press.common, paint.common], ["dent", "dent"]
Defect(paint, "drip")
assert: [paint.common, worst], ["drip", "dent"]
worst
# return: "dent"
```

- One class-level rule keeps each line's most common defect, and `worst`
  keeps the most common defect of every line. Reclassifying or moving a
  defect updates every affected line.
- On a tie the defect created first wins: once the reclassified dent moves to
  the paint line, that line has one dent and one drip, and the dent, created
  earlier, is its mode until a second drip arrives.
- See [NUC 75](nuc-0075.md) and the
  [root contract](../nucleoid.spec.md#list-and-population-modes).

## Noun list queries

**Noun queries keep the fill color and width of every shape live**

```nuc
class Shape(cells): this.cells = cells
$Shape.fill_color = mode of $Shape.cells.map(cell -> cell.color)
$Shape.width = max of $Shape.cells.map(cell -> cell.x) - min of $Shape.cells.map(cell -> cell.x) + 1
square = Shape([{x: 1, color: 3}, {x: 2, color: 3}, {x: 2, color: 5}])
line = Shape([{x: 4, color: 7}])
widest = max of Shape.map(shape -> shape.width)
assert: [square.fill_color, square.width, line.fill_color, line.width], [3, 2, 7, 1]
assert: widest, 2
square.cells.push({x: 5, color: 5})
square.cells.push({x: 5, color: 5})
assert: [square.fill_color, square.width, widest], [5, 5, 5]
widest
# return: 5
```

- Each shape's fill color is the most common color of its cells, and its
  width spans its smallest and largest column. Both are class-level rules,
  so every shape keeps its own, and `widest` follows every shape's width.
- `NOUN of OPERAND` takes a unary operand, so the width rule subtracts two
  queries and adds `1`. See [NUC 75](nuc-0075.md) and the
  [root contract](../nucleoid.spec.md#noun-list-queries-with-of).

## List and population extreme items

**Extreme items keep the nearest free charger of every robot live**

```nuc
class Charger(name, x, free):
    this.name = name
    this.x = x
    this.free = free
class Robot(name, x, battery):
    this.name = name
    this.x = x
    this.battery = battery
$Robot.dock = Charger.filter(charger -> charger.free).minBy(charger -> Math.abs(charger.x - $Robot.x))
neediest = Robot.minBy(robot -> robot.battery)
north = Charger("north", 0, true)
south = Charger("south", 10, true)
rex = Robot("rex", 2, 80)
ivy = Robot("ivy", 9, 35)
assert: [rex.dock, ivy.dock, neediest], [north, south, ivy]
north.free = false
ivy.battery = 90
assert: [rex.dock, neediest], [south, rex]
Charger("mid", 4, true)
rex.x = 6
assert: [rex.dock.name, ivy.dock.name], ["mid", "south"]
neediest.dock.name
# return: "mid"
```

- A class-level rule keeps each robot's dock, the free charger nearest to it.
  `$Robot` inside the selector is the robot the rule is evaluated for, so
  every robot measures its own distances, and `neediest` keeps the robot with
  the lowest battery.
- Taking `north` out of service moves `rex` to `south`. A new charger at `4`
  becomes the dock of `rex` and stays so when `rex` moves to `6`, while `ivy`
  keeps `south`, and `neediest.dock` follows both rules.
- See [NUC 77](nuc-0077.md) and the
  [root contract](../nucleoid.spec.md#list-and-population-extreme-items).

## Extends clauses

**Extends clauses keep a fleet's inherited insurance rule live**

```nuc
threshold = 1000
class Asset(name, worth):
    this.name = name
    this.worth = worth
$Asset.insured = $Asset.worth > threshold
class Vehicle(name, worth, wheels) extends Asset:
    this.name = name
    this.worth = worth
    this.wheels = wheels
class Drone extends Asset
van = Vehicle("van", 30000, 4)
probe = Drone("probe", 800)
assert([van.insured, probe.insured], [true, false])
threshold = 500
assert(probe.insured, true)
van.worth = 400
assert(van.insured, false)
[Asset.length, Vehicle.length, Drone.length]
# return: [0,1,1]
```

- `Vehicle` names its parent with `extends` and declares its own
  constructor, while the bodyless `class Drone extends Asset` uses the
  constructor of `Asset`. The class-level rule `$Asset.insured` applies to
  both, so lowering `threshold` insures the drone and a lower worth uninsures
  the van.
- The populations stay separate: `Asset` has no instances of its own. See
  [NUC 78](nuc-0078.md) and the
  [root contract](../nucleoid.spec.md#extends-clauses-for-subclasses).

## Property selector arguments

**Selector arguments keep the stock levels of every bin live**

```nuc
class Bin(zone, sku, qty):
    this.zone = zone
    this.sku = sku
    this.qty = qty
$Bin.low = $Bin.qty < 5
stocked = Bin.sum(.qty)
short = Bin.filter(.low).map(.sku)
fullest = Bin.maxBy(.qty)
zones = Bin.distinct(.zone)
north = Bin("north", "bolt", 40)
south = Bin("south", "nut", 3)
Bin("north", "gear", 12)
assert: [stocked, short, fullest, zones], [55, ["nut"], north, ["north", "south"]]
north.qty = 2
south.qty = 30
assert: [stocked, short, fullest.sku], [44, ["bolt"], "nut"]
fullest.zone
# return: "south"
```

- `.qty`, `.low`, `.sku` and `.zone` stand for the functions that read those
  properties from each bin, so `Bin.sum(.qty)` totals the stock and
  `Bin.filter(.low).map(.sku)` lists the parts that run short. `.low` reads
  the class-level rule `$Bin.low`, so it follows every quantity change.
- Changing two quantities re-evaluates every declaration that reads them:
  the bolt bin runs short, and the nut bin is no longer short and becomes
  the fullest. See
  [NUC 79](nuc-0079.md) and the
  [root contract](../nucleoid.spec.md#property-selector-arguments).

## List and population averages

**Averages keep the rating of every product live as reviews arrive**

```nuc
class Product(name): this.name = name
class Review(product, stars):
    this.product = product
    this.stars = stars
$Product.rating = Review.filter(review -> review.product == $Product).average(review -> review.stars)
$Product.featured = $Product.rating >= 4
lamp = Product("lamp")
desk = Product("desk")
Review(lamp, 5)
Review(lamp, 4)
late = Review(desk, 2)
store = Review.average(review -> review.stars)
assert: [lamp.rating, lamp.featured, desk.rating, desk.featured, store], [4.5, true, 2, false, 3.6666666666666665]
late.stars = 5
Review(desk, 4)
assert: [desk.rating, desk.featured, store], [4.5, true, 4.5]
chair = Product("chair")
assert: [Number.isNaN(chair.rating), chair.featured], [true, false]
[lamp.featured, desk.featured, chair.featured]
# return: [true,true,false]
```

- `$Product.rating` is the mean of the stars of each product's reviews, and
  `$Product.featured` follows it. Raising the late review to five stars and
  adding a four-star review lift the desk to `4.5`, so it becomes featured,
  and `store`, the mean of every review, follows both changes.
- The chair has no reviews, so its rating is NaN, the mean of nothing, and
  it is not featured, because NaN compares false. See
  [NUC 80](nuc-0080.md) and the
  [root contract](../nucleoid.spec.md#list-and-population-averages).

## Object spread entries

**Spread entries keep shipping labels in step with their customers**

```nuc
class Customer(name, city):
    this.name = name
    this.city = city
class Order(customer, item, qty):
    this.customer = customer
    this.item = item
    this.qty = qty
$Order.label = {...$Order.customer, item: $Order.item, qty: $Order.qty}
ada = Customer("ada", "Paris")
lamp = Order(ada, "lamp", 1)
desk = Order(ada, "desk", 2)
assert: lamp.label, {id: "ada", name: "ada", city: "Paris", item: "lamp", qty: 1}
ada.city = "Lyon"
ada.zip = "69001"
desk.qty = 3
assert: desk.label, {id: "ada", name: "ada", city: "Lyon", zip: "69001", item: "desk", qty: 3}
[lamp.label.city, lamp.label.zip, desk.label.qty]
# return: ["Lyon","69001",3]
```

- `$Order.label` copies every entry of the order's customer, its `id`
  first, and adds the order's item and quantity. Each order gets its own
  label from the same rule.
- The labels follow their sources: Ada's move to Lyon, her new `zip` entry
  and the desk's new quantity re-evaluate them without naming any of the
  customer's fields. See [NUC 81](nuc-0081.md) and the
  [root contract](../nucleoid.spec.md#object-spread-entries).

## Depth-limited reasoning selections

**Depth keeps the direct effects of a part's cost in view**

```nuc
class Part(cost):
    this.cost = cost
bolt = Part(2)
gear = Part(5)
motor = bolt.cost * 4 + gear.cost
robot = motor * 2 + 10
kit = bolt.cost * 10
direct = bolt.cost |> affects |> depth(1)
assert: direct.map(step -> step.node), ["motor", "kit"]
assert: (affects bolt.cost).length, 3
bolt.cost = 3
[direct.length, direct[0].holds, direct[1].holds]
# return: [2,17,30]
```

- `bolt.cost |> affects |> depth(1)` selects what reads the bolt's cost
  directly, the motor and the kit, and leaves out the robot, which reads it
  only through the motor; `affects bolt.cost` selects all three.
- The selection follows the model: a new cost updates both steps it keeps,
  and a new direct reader of the cost would join it. See
  [NUC 82](nuc-0082.md) and the
  [root contract](../nucleoid.spec.md#depth-limited-reasoning-selections).

## Hypothetical differences

**Diff shows which orders a delayed shipment would make late**

```nuc
class Order(item, days):
    this.item = item
    this.days = days
$Order.late = $Order.days > 3
overdue = Order.filter(order -> order.late).map(order -> order.item)
desk = Order("desk", 2)
lamp = Order("lamp", 5)
def changes() => (model |> diff).map(step -> step.node)
delay = suppose desk.days = 6: changes()
rush = suppose lamp.days = 1: changes()
assert: delay, ["overdue", "desk.days", "desk.late"]
assert: rush, ["overdue", "lamp.days", "lamp.late"]
lamp.days = 2
suppose lamp.days = 3: changes()
# return: ["lamp.days"]
```

- `changes()` evaluates `model |> diff` inside each `suppose` block that
  calls it, so one function reports what each plan would change: a six-day
  delay makes the desk late and adds it to `overdue`, and rushing the lamp
  takes the lamp off it.
- The answer follows the model: once the lamp ships in two days, a third day
  would change only its `days`, because it stays on time. See
  [NUC 83](nuc-0083.md) and the
  [root contract](../nucleoid.spec.md#hypothetical-differences).

## List and population medians

**Medians keep the typical rent of every district live as listings change**

```nuc
class District(name): this.name = name
class Flat(district, rent):
    this.district = district
    this.rent = rent
$District.typical = Flat.filter(flat -> flat.district == $District).median(flat -> flat.rent)
$District.affordable = $District.typical < 1500
north = District("north")
south = District("south")
Flat(north, 1200)
Flat(north, 1400)
penthouse = Flat(north, 4000)
Flat(south, 1600)
Flat(south, 1700)
assert: [north.typical, north.affordable, south.typical, south.affordable], [1400, true, 1650, false]
penthouse.district = south
Flat(north, 950)
assert: [north.typical, south.typical, south.affordable], [1200, 1700, false]
[north.affordable, south.affordable]
# return: [true,false]
```

- `$District.typical` is the median rent of each district's flats, and
  `$District.affordable` follows it. The penthouse leaves the north
  affordable: however high its rent, the north's median is the middle flat's
  1,400, while the mean of the three northern rents is 2,200.
- Moving the penthouse to the south and adding a cheaper northern flat
  re-evaluates both districts: the north's median falls to 1,200 and the
  south's rises to 1,700. See [NUC 84](nuc-0084.md) and the
  [root contract](../nucleoid.spec.md#list-and-population-medians).

## Initializing constructor parameters

**Initializing parameters keep a fleet's service schedule live**

```nuc
class Truck(this.plate, this.km, this.limit = 30000)
$Truck.due = $Truck.km >= $Truck.limit
$Truck.left = $Truck.limit - $Truck.km
north = Truck("NO-1", 29500)
south = Truck("SO-2", 12000, 15000)
due = Truck.filter(truck -> truck.due).map(truck -> truck.plate)
assert([north.left, south.left, due], [500, 3000, []])
north.km = 30100
south.limit = 12000
assert(due, ["NO-1", "SO-2"])
due
# return: ["NO-1","SO-2"]
```

- The header of `Truck` both declares and stores each truck's plate,
  mileage and service limit, so the class-level rules `$Truck.due` and
  `$Truck.left` read them from the first truck on, and `south` keeps its own
  limit instead of the default.
- Driving `north` past its limit and lowering the limit of `south` put both
  trucks on the `due` list. See [NUC 87](nuc-0087.md) and the
  [root contract](../nucleoid.spec.md#initializing-constructor-parameters).

## Derivation path selections

**Paths show which steps a late supplier pushes back**

```nuc
class Supplier(days):
    this.days = days
steel = Supplier(5)
glass = Supplier(3)
frame = steel.days + 2
panes = glass.days + 1
assembly = frame + panes
crating = steel.days + 1
delivery = assembly + 4
invoice = crating * 10
late = steel.days |> affects |> path(delivery)
assert(late.map(step -> step.node), ["frame", "assembly", "delivery"])
assert((affects steel.days).length, 5)
steel.days = 9
late.map(step -> step.holds)
# return: [11,15,19]
```

- `steel.days |> affects |> path(delivery)` selects what the steel lead time
  moves on its way to the delivery date: the frame, the assembly and the
  delivery. It leaves out the crating and the invoice, which also read the lead
  time but do not reach the delivery; `affects steel.days` selects all five.
- The selection follows the model: a longer lead time updates the three steps
  it keeps, and a new step between the steel and the delivery would join it.
  See [NUC 88](nuc-0088.md) and the
  [root contract](../nucleoid.spec.md#derivation-path-selections).

## Named function declarations

**Function declarations keep a class's grading rule live**

```nuc
function grade(score) {
    if score >= 80:
        return "merit"
    if score >= 50:
        return "pass"
    return "fail"
}
class Student(name, score):
    this.name = name
    this.score = score
$Student.result = grade($Student.score)
ana = Student("ana", 72)
bo = Student("bo", 41)
assert([ana.result, bo.result], ["pass", "fail"])
bo.score = 85
assert(bo.result, "merit")
function grade(score) { return score >= 60 }
[ana.result, bo.result]
# return: [true,true]
```

- `grade` is declared with the JavaScript spelling and called by the
  class-level rule `$Student.result`, so each student's result follows its
  own score: raising the score of `bo` moves it to `"merit"`.
- Redeclaring `grade` updates every result that calls it, as redefining a
  `def` function does: both students now reach the new mark of `60`. See
  [NUC 89](nuc-0089.md) and the
  [root contract](../nucleoid.spec.md#named-function-declarations).

## Enum declarations

**Enums keep a ticket's priority rules readable**

```nuc
enum Priority: LOW = 1, HIGH = 3
enum State: OPEN, CLOSED
class Ticket(title, priority):
    this.title = title
    this.priority = priority
    this.state = State.OPEN
$Ticket.urgent = $Ticket.priority == Priority.HIGH && $Ticket.state == State.OPEN
crash = Ticket("crash", Priority.HIGH)
typo = Ticket("typo", Priority.LOW)
assert([crash.urgent, typo.urgent], [true, false])
crash.state = State.CLOSED
[crash.urgent, Ticket.filter(t -> t.priority > Priority.LOW).length]
# return: [false,1]
```

- `Priority` gives its members numbers, so the filter compares
  `t.priority > Priority.LOW`, and `State` names the two states a ticket
  can be in. Members read as ordinary values in constructors and class-level
  rules.
- Closing the crash ticket clears `urgent`, because the rule reads the
  ticket's state. Every write to an enum raises
  `TypeError("Enums are read-only")`, so the categories cannot drift. See
  [NUC 90](nuc-0090.md) and the
  [root contract](../nucleoid.spec.md#enum-declarations).

## List comprehensions

**List comprehensions keep a class's shipping labels live**

```nuc
class Parcel(code, weight):
    this.code = code
    this.weight = weight
limit = 5
heavy = [parcel.code for parcel of Parcel if parcel.weight > limit]
labels = ["#" + code for code in heavy]
Parcel("A1", 2)
b2 = Parcel("B2", 8)
assert(labels, ["#B2"])
b2.weight = 4
assert(labels, [])
limit = 1
[heavy, labels]
# return: [["A1","B2"],["#A1","#B2"]]
```

- `heavy` filters the parcels over the weight limit and keeps their codes,
  and `labels` builds a label from each of those codes. Both are
  comprehensions, exactly the `filter` and `map` calls they spell.
- `labels` follows `heavy`, which follows the parcels, their weights and the
  limit: lightening `B2` empties the labels, and lowering the limit brings
  both parcels back. See [NUC 92](nuc-0092.md) and the
  [root contract](../nucleoid.spec.md#list-comprehensions).

## Reactive cotangent

**Cotangents keep directional ratios live as angles change**

```nuc
class Direction(angle):
    this.angle = angle
$Direction.ratio = Math.cot($Direction.angle)
north = Direction(0)
diagonal = Direction(1)
assert([north.ratio, diagonal.ratio], [Number.POSITIVE_INFINITY, 1 / Math.tan(1)])
diagonal.angle = -1
assert(diagonal.ratio, 1 / Math.tan(-1))
north.angle = 0.5
assert(north.ratio, 1 / Math.tan(0.5))
try:
    diagonal.angle = 0
    throw "restore"
catch error:
    assert(error, "restore")
assert(diagonal.angle, -1)
assert(diagonal.ratio, 1 / Math.tan(-1))
```

- The [NUC 100 candidate](nuc-0100.md) names exactly `1 / Math.tan(angle)`,
  with radians as input and tangent followed by reciprocal in that order.
  Each direction's ratio follows its own angle; a failed transaction restores
  the angle and its derived ratio. The tangent assertions remain independent
  controls. See the [root contract](../nucleoid.spec.md#reactive-cotangent).
- Intrinsic numeric conversion, missing-first NaN, signed-zero reciprocal
  infinities, ignored-extra dependencies, the supplied-Undefined gate and
  inherited live-null/parameter deferral remain as for existing Math calls.

## Implicit receiver properties

**Leading dots keep an order line's rules about its own fields**

```nuc
class Line(item, qty, price):
    this.item = item
    this.qty = qty
    this.price = price
$Line.total = .qty * .price
$Line.size = .qty >= 10 ? "bulk" : "single"
$Line.text = `${.item} x${.qty}: ${.total}`
pens = Line("pen", 12, 2)
ink = Line("ink", 1, 9)
assert([pens.size, ink.text], ["bulk", "ink x1: 9"])
pens.qty = 4
ink.price = 10
[pens.total, pens.size, pens.text, ink.text]
# return: [8,"single","pen x4: 8","ink x1: 10"]
```

- Each rule names `$Line` once, in its target, and reads the line's other
  fields with a leading dot: `.qty * .price` is
  `$Line.qty * $Line.price`, and the template reads `$Line.item`,
  `$Line.qty` and `$Line.total`.
- The rules stay live like their spelled-out forms: dropping the pens to
  four makes them single-quantity and updates their total and text, and a
  new ink price updates the ink's text. See
  [NUC 102](nuc-0102.md) and the
  [root contract](../nucleoid.spec.md#implicit-receiver-properties).

## Any and all class references

**Any and all keep every shelf's reorder rule live**

```nuc
class Shelf(stock, minimum):
    this.stock = stock
    this.minimum = minimum
all Shelf.low = all Shelf.stock < all Shelf.minimum
if any Shelf.low:
    all Shelf.order = all Shelf.minimum * 2 - all Shelf.stock
else:
    all Shelf.order = 0
apples = Shelf(4, 10)
pears = Shelf(12, 10)
assert([apples.order, pears.order], [16, 0])
pears.stock = 3
all Shelf is tracked
[apples.order, pears.order, pears.tracked]
# return: [16,17,true]
```

- `all Shelf` and `any Shelf` stand for `$Shelf`, so each shelf knows
  whether it runs low and orders enough to reach twice its minimum, and a
  shelf that is not low orders nothing.
- Lowering the pears' stock to 3 makes them low, and their order follows.
  The predicate sentence `all Shelf is tracked` states
  `$Shelf.tracked = true` for every shelf. See [NUC 104](nuc-0104.md) and the
  [root contract](../nucleoid.spec.md#any-and-all-class-references).

## List and population sorting by key

**Sorting by key keeps every courier's route in order of distance**

```nuc
class Courier(name, stops):
    this.name = name
    this.stops = stops
$Courier.route = $Courier.stops.sortBy(stop -> stop.km).map(stop -> stop.name)
ana = Courier("ana", [{name: "mill", km: 12}, {name: "dock", km: 3}, {name: "farm", km: 7}])
bo = Courier("bo", [{name: "pier", km: 5}])
assert([ana.route, bo.route], [["dock", "farm", "mill"], ["pier"]])
ana.stops.push({name: "yard", km: 1})
ana.stops[0].km = 2
bo.stops = [{name: "gate", km: 9}, {name: "lane", km: 2}]
assert(ana.stops[0].name, "mill")
[ana.route, bo.route]
# return: [["yard","mill","dock","farm"],["lane","gate"]]
```

- A class-level rule keeps the route of each courier: the names of its stops
  in order of distance. `$Courier.stops.sortBy(stop -> stop.km)` sorts the
  courier's own list, so every courier has its own route.
- A new stop, a changed distance and a new list of stops re-sort the routes,
  while each courier's `stops` keep the order they were listed in. See
  [NUC 105](nuc-0105.md) and the
  [root contract](../nucleoid.spec.md#list-and-population-sorting-by-key).

## Intersect and except reasoning selections

**Except shows what a steel delay moves and a glass delay does not**

```nuc
steel = 5
glass = 3
frame = steel + 2
panes = glass + 1
assembly = frame + panes
crating = steel + 1
delivery = assembly + 4
invoice = crating * 10
pushed = affects steel |> except(affects glass)
shared = affects steel |> intersect(affects glass)
assert(pushed.map(step -> step.node), ["frame", "crating", "invoice"])
assert(shared.map(step -> step.node), ["assembly", "delivery"])
crating = panes + 1
steel = 9
[pushed.map(step -> step.node), shared.map(step -> step.holds)]
# return: [["frame"],[15,19]]
```

- `affects steel |> except(affects glass)` is what a later steel delivery
  moves and a later glass delivery does not: the frame, the crating and the
  invoice. `affects steel |> intersect(affects glass)` is what both delays
  move: the assembly and the delivery.
- Both follow the model: once the crating is built from the panes, a steel
  delay moves only the frame of its own, and the shared steps hold the new
  dates. See [NUC 106](nuc-0106.md) and the
  [root contract](../nucleoid.spec.md#intersect-and-except-reasoning-selections).

## Reactive list flattening

**Flattening keeps the dirty tiles of every room counted**

```nuc
class Room(tiles):
    this.tiles = tiles
$Room.floor = $Room.tiles.flat()
$Room.dirty = $Room.floor.count(tile => tile == "dirt")
kitchen = Room([["dirt", "clean"], ["clean", "dirt"]])
hall = Room([["clean"]])
assert([kitchen.dirty, hall.dirty], [2, 0])
kitchen.tiles.push(["dirt", "dirt"])
hall.tiles = [["dirt"], ["clean"]]
[kitchen.dirty, hall.dirty]
# return: [4,1]
```

- Each room keeps its tiles as rows, and `$Room.floor` flattens them into
  one list per room, exactly as `$Room.tiles.flatMap(v => v)` would, so
  `$Room.dirty` counts the dirty tiles of every room.
- A new row of tiles in the kitchen and a new floor plan in the hall update
  both counts. See [NUC 103](nuc-0103.md) and the
  [root contract](../nucleoid.spec.md#reactive-list-flattening).

## Membership tests

**Membership tests keep an order book's rules live**

```nuc
class Order(status):
    this.status = status
$Order.open = $Order.status in ["new", "paid"]
if $Order.status not in ["new", "paid"]:
    $Order.archived = true
first = Order("new")
second = Order("shipped")
assert([first.open, second.open], [true, false])
assert(second.archived, true)
first.status = "paid"
first.open
# return: true
```

- `$Order.open = $Order.status in ["new", "paid"]` gives every order its
  own `open`, and the class-level `if` archives each order whose status is
  not in that list.
- Changing `first.status` re-evaluates both rules for that order. See
  [NUC 97](nuc-0097.md) and the
  [root contract](../nucleoid.spec.md#membership-tests).

## Every and predicate sentences

**Every keeps a fleet's rules stated as claims about every truck**

```nuc
class Truck(name, load, limit):
    this.name = name
    this.load = load
    this.limit = limit
every Truck is registered
every Truck.overloaded = every Truck.load > every Truck.limit
if any Truck.overloaded:
    all Truck.status = "hold"
else:
    all Truck.status = "go"
north = Truck("north", 8, 10)
south = Truck("south", 12, 10)
assert([north.status, south.status], ["go", "hold"])
assert(every truck in Truck: truck.registered, true)
south.load = 9
safe = every truck in Truck: not truck.overloaded
[south.status, safe]
# return: ["go",true]
```

- `every Truck is registered` states `$Truck.registered = true`, and
  `every Truck` stands for `$Truck` in the overloaded rule. Inside the if
  statement `every` is rejected, so its condition reads `any Truck` and both
  of its clauses `all Truck`, which stand for `$Truck` too. So each truck is
  registered, knows whether it is overloaded and takes its status from that.
- `every truck in Truck: not truck.overloaded` is the quantifier
  `all truck in Truck: not truck.overloaded`. Lightening `south` releases it
  and makes the fleet safe. See [NUC 98](nuc-0098.md) and the
  [root contract](../nucleoid.spec.md#every-quantifiers-class-references-and-predicate-sentences).

## Copyright

Copyright 2020 Nucleoid

This document is licensed under the Apache License, Version 2.0. See
[LICENSE](../LICENSE) for the full text.
