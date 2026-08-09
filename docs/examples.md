# Nucleoid Examples

Worked examples of the language, each one a complete program that runs as written.

The sections follow [README.md](README.md) one for one, so an example can
be read next to the behaviour it demonstrates. Every example is taken from
`nucleoid.spec.md`, which is normative; where this document and the
specification disagree, the specification wins. See [NUC 0](nuc-0000.md) for the
index of NUC documents.

The examples are executable. `cargo test --test examples` runs every block on
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
12. [Error Reference](#12-error-reference)

[Syntax Summary](README.md#13-syntax-summary) is a table of forms rather than
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

**A variable that reads itself reads only its value**

- The occurrence on the right is the current value, not a dependency.
- This is why it is not a cycle.

```nuc
radius = 10
radius = radius + 10

assert(radius, 20)
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

Full detail: [README.md §3](README.md#3-expressions), [NUC 10](nuc-0010.md).

---

## 4. Types and Instances

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

**A subtype calls its supertype's constructor**

- The supertype is named after the colon, and `super(...)` invokes it.

```nuc
class Person(name: str):
    this.name = name

class Student: Person
    def init(name, school):
        super(name)
        this.name = name
        this.school = school

student1 = Student("Emma", "Riverside High")

assert(student1, { "id": "student1", "name": "Emma", "school": "Riverside High" })
```

**Re-declaring a type keeps its instances and its rules**

- The instances made before the new declaration are still there, and the rule still holds for them.
- An attribute the earlier instances never had is `null` on them.

```nuc
class Message:
    pass

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

Full detail: [README.md §4](README.md#4-types-and-instances), [NUC 3](nuc-0003.md).

---

## 5. Properties

**A property may be assigned from one that does not exist yet**

- Reading something undefined yields `null` and re-evaluates when it arrives.

```nuc
class Order:
    pass

order1 = Order()
order1.upc = "04061" + order1.barcode

assert(order1.upc, null)

order1.barcode = "94067"

assert(order1.upc, "0406194067")
```

**A property reads through a reference, and the dependency follows it**

- `server1.summary` reads `ip1` through `server1.ip`, so a change to `ip1` reaches it.

```nuc
class Server:
    pass

class IP:
    pass

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
class Agent:
    pass

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
class Employee:
    pass

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
class Storage:
    pass

normal = "NORMAL"; low = "LOW"; empty = "EMPTY"

if $Storage.capacity > 25:
    $Storage.status = normal
else if $Storage.capacity > 0:
    $Storage.status = low
else:
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
class User:
    pass

class Registration:
    pass

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
class Toy:
    pass

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

**A for of statement runs its body once, over what is there at the time**

- The loop is carried out immediately; it does not re-run when `Question` grows.
- What it creates is declarative all the same, so the rule on `Summary` still reaches the instances it made.

```nuc
class Question:
    pass

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
def generate(number):
    return number * 10

random = 10
number = generate(random)

assert(number, 100)

random = 20

assert(number, 200)
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
class Product:
    pass

product1 = Product()

class Quality:
    pass

product1.quality = Quality()
product1.quality.score = 15

assert(product1.quality.class, null)

$Quality.class = String.fromCharCode(65 + Math.floor($Quality.score / 10))

assert(product1.quality.class, "B")
```

**A regular expression may guard a type**

- The guard is a standing rule, so it is the assignment that fails, not a later check.

```nuc
class User:
    pass

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

## 12. Error Reference

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
class Schedule:
    pass

class Place:
    pass

value = Schedule()

assert(value, { "id": "value" })

try:
    value.value = Place()
catch error:
    assert(error, TypeError("Cannot use 'value' as a property"))
```

**A class-level rule may not be declared in a non-class block**

```nuc
class Person:
    pass

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

Full detail: [README.md §12](README.md#12-error-reference), [NUC 9](nuc-0009.md).

---

## Copyright

Copyright 2020 Nucleoid

This document is licensed under the Apache License, Version 2.0. See
[LICENSE](../LICENSE) for the full text.
