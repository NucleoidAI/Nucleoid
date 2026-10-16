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
  optional; the same source with `else:` remains equivalent. Every condition
  header retains its colon and existing branch order.

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
    if state == "blocked":
        return "hold"
    else urgent:
        return "fast"
    elif state == "ready":
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
    if state == "blocked":
        return "pause"
    else urgent:
        return "rush"
    elif state == "ready":
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
- Function definitions may be live or frozen without freezing their inputs.
  Ordinary noncapture, explicit returns, receiver scopes and transaction
  failures are unchanged; no receiver-frame repair is included.
- Explicit `else if`, unconditional `else:`/`elif:` and their accepted
  colon-free indented final forms remain supported. See the
  [root contract](../nucleoid.spec.md#direct-conditional-alternatives) and
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

## Copyright

Copyright 2020 Nucleoid

This document is licensed under the Apache License, Version 2.0. See
[LICENSE](../LICENSE) for the full text.
