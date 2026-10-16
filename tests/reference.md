# Nucleoid Language Reference - Documented Behaviours

Executable form of the behaviours stated in `docs/README.md`, in the same
case format as `nucleoid.spec.md`. Where this file and the specification
disagree, the specification wins.

Named arrows may optionally omit parentheses around one untyped ordinary
identifier, one ordinary identifier with its existing single-identifier
annotation, or NUC27's complete comma-separated list of two or more untyped
ordinary identifiers immediately followed by `=>` on the same logical line.
Parenthesized alternatives, class/anonymous parameter lists and typed/untyped
single-parameter indented blocks stay valid. Multiple-parameter bare blocks and
typed/mixed multiple-parameter bare arrows are not added. Annotations remain
metadata with the existing binding and runtime behavior.

```

# Nucleoid tracks dependencies through a template literal

# host is "localhost"
host = "localhost"

# port is 8080
port = 8080

# url is the host and port in a template
url = `http://${host}:${port}`

assert(url, "http://localhost:8080")

# port is 9090
port = 9090

assert(url, "http://localhost:9090")

---

# Nucleoid clears transitive dependents when a variable is deleted

# base is 10
base = 10

# tax is base times 0.1
tax = base * 0.1

# total is base plus tax
total = base + tax

assert(total, 11)

# base is deleted
delete base

assert(tax, null)
assert(total, null)

try:
    # base
    base
catch error:
    assert(error, ReferenceError("base is not defined"))

---

# Nucleoid clears a chain of dependents when the head is deleted

# first is 2
first = 2

# second is first times 2
second = first * 2

# third is second times 2
third = second * 2

assert(third, 8)

# first is deleted
delete first

assert(second, null)
assert(third, null)

---

# Nucleoid re-points a dependency when the reference it reads through changes

# There is a Plan type, which has a rate as a number
class Plan(rate: int):
    this.rate = rate

# There is a User type
class User

# basic is a Plan whose rate is 10
basic = Plan(10)

# pro is a Plan whose rate is 30
pro = Plan(30)

# user1 is a User whose plan is basic
user1 = User()
user1.plan = basic

# user1's bill is user1's plan's rate times 12
user1.bill = user1.plan.rate * 12

assert(user1.bill, 120)

# user1's plan is pro
user1.plan = pro

assert(user1.bill, 360)

# pro's rate is 40
pro.rate = 40

assert(user1.bill, 480)

---

# Nucleoid applies a supertype's class-level rule to a subtype instance

# There is a Vehicle type, which has a make as a string
class Vehicle(make: str):
    this.make = make

# Every vehicle is registered
$Vehicle.registered = true

# There is a Truck type, which is a subtype of Vehicle
class Truck: Vehicle
    def init make payload:
        super(make)
        this.payload = payload

# truck1 is a Truck whose make is "Ford" and whose payload is 2000
truck1 = Truck("Ford", 2000)

assert(truck1.make, "Ford")
assert(truck1.payload, 2000)
assert(truck1.registered, true)

---

# Nucleoid runs a recursive function with several return statements

# factorial returns n times the factorial of n minus 1
def factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)

# result is the factorial of 5
result = factorial(5)

assert(result, 120)

---

# Nucleoid keeps named callbacks and aliases live while freezing function values

# def name(params) => expression is a named block containing return expression
# Named arrow braces are object literals; blocks without return still yield null
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

---

# Nucleoid restores callable properties after null replacement and deletion

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

---

# Nucleoid shadows outer functions with null and non callable parameters

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

---

# Nucleoid nests for of statements as a cross product

# There is a Size type
class Size

# There is a Color type
class Color

# There is a Variant type, which has a size and a color
class Variant(size, color):
    this.size = size
    this.color = color

# There are two Sizes and two Colors
Size(); Size(); Color(); Color()

# For each size and each color, there is a Variant
for size of Size:
    for color of Color:
        Variant(size, color)

assert(Variant.length, 4)

---

# Nucleoid updates a mapped list when its source or a variable changes

# prices is a list of 10, 20 and 30
prices = [10, 20, 30]

# rate is 2
rate = 2

# doubled is each price times rate
doubled = prices.map(p => p * rate)

assert(doubled[2], 60)

# rate is 3
rate = 3

assert(doubled[2], 90)

# Add 40 to prices
prices.push(40)

assert(doubled[3], 120)

---

# Nucleoid rolls back instances created earlier in a failing loop

# There is a Source type, which has a code as a string
class Source(code: str):
    this.code = code

# There is an Item type, which has a code as a string
class Item(code: str):
    this.code = code

# There are Sources whose codes are "A", "B" and "BAD"
Source("A"); Source("B"); Source("BAD")

# If any item's code is "BAD", then throw "INVALID_CODE"
if $Item.code == "BAD":
    throw "INVALID_CODE"

try:
    # For each source, there is an Item whose code is the source's code
    for source of Source:
        Item(source.code)
catch error:
    assert(error, "INVALID_CODE")

assert(Item.length, 0)

---

# Nucleoid does not install a class-level rule that throws for an existing instance

# There is a Reading type
class Reading

# reading1 is a Reading whose celsius is 200
reading1 = Reading()
reading1.celsius = 200

try:
    # If any reading's celsius is greater than 100, then throw "OUT_OF_RANGE"
    if $Reading.celsius > 100:
        throw "OUT_OF_RANGE"
catch error:
    assert(error, "OUT_OF_RANGE")

# reading1's celsius is 150
reading1.celsius = 150

assert(reading1.celsius, 150)

---

# Nucleoid reuses a variable name after its instance is deleted

# There is a Node type
class Node

# node1 is a Node
node1 = Node()

# node1 is deleted
delete node1

# node1 is a Node
node1 = Node()

assert(Node.length, 1)
assert(node1.id, "node1")

---

# Nucleoid assigns a comparison directly

# reading is 5
reading = 5

# threshold is 10
threshold = 10

# alarm is whether reading is greater than threshold
alarm = reading > threshold

assert(alarm, false)

# reading is 15
reading = 15

assert(alarm, true)

---

# Nucleoid supports the list functions of the reference

# values is a list of 3, 1 and 2
values = [3, 1, 2]

assert(values.join("-"), "3-1-2")
assert(values.some(v => v > 2), true)
assert(values.every(v => v > 0), true)
assert(values.slice(1).length, 2)
assert(values.reduce((sum, v) => sum + v, 0), 6)

---

# Nucleoid evaluates a class-level conditional chain per instance

# There is a Grade type
class Grade

# grade1 is a Grade whose score is 95
grade1 = Grade()
grade1.score = 95

# grade2 is a Grade whose score is 72
grade2 = Grade()
grade2.score = 72

# If any grade's score is greater than 89, then the grade's letter is "A",
# else if the score is greater than 79, then "B", else "C"
if $Grade.score > 89:
    $Grade.letter = "A"
else if $Grade.score > 79:
    $Grade.letter = "B"
else
    $Grade.letter = "C"

assert(grade1.letter, "A")
assert(grade2.letter, "C")

# grade1's score is 85
grade1.score = 85

assert(grade1.letter, "B")
assert(grade2.letter, "C")

---

# Nucleoid aggregates another type's list in a class-level rule

# There is a Category type
class Category

# There is a Product type
class Product

# category1 is a Category
category1 = Category()

# Any category's revenue is the total price of its products
$Category.revenue = Product.filter(p => p.category == $Category).reduce((sum, p) => sum + p.price, 0)

assert(category1.revenue, 0)

# product1 is a Product in category1 whose price is 30
product1 = Product()
product1.category = category1
product1.price = 30

assert(category1.revenue, 30)

# product2 is a Product in category1 whose price is 12
product2 = Product()
product2.category = category1
product2.price = 12

assert(category1.revenue, 42)

---

# Nucleoid keeps a false condition from clearing its target

# There is a Toy type
class Toy

# toy1 is a Toy whose shape is "SQUARE"
toy1 = Toy()
toy1.shape = "SQUARE"

# color is "BLUE"
color = "BLUE"

# if color is "RED", then toy1's shape is "CIRCLE"
if color == "RED":
    toy1.shape = "CIRCLE"

assert(toy1.shape, "SQUARE")

# color is "RED"
color = "RED"

assert(toy1.shape, "CIRCLE")

---

# Nucleoid reports an unknown function of a built-in

try:
    # date is the wrong date
    date = Date.wrong()
catch error:
    assert(error, TypeError("Date.wrong is not a function"))

# list is a list of 1 and 2
list = [1, 2]

try:
    # result is the wrong list function
    result = list.wrong()
catch error:
    assert(error, TypeError("list.wrong is not a function"))

---

# Nucleoid discards the previous dependencies when a variable is reassigned

# a is 1, b is 2
a = 1
b = 2

# c is a plus 3
c = a + 3

assert(c, 4)

# c is b plus 3
c = b + 3

assert(c, 5)

# b is 4
b = 4

assert(c, 7)

# a is 100, which c no longer reads
a = 100

assert(c, 7)

---

# Nucleoid clears only the dependents of a deleted property

# There is a Report type
class Report

# report1 is a Report of April 2019
report1 = Report()
report1.year = 2019
report1.month = 4

# report1's period is its year and month, and its label is its year
report1.period = report1.year + "-" + report1.month
report1.label = "P" + report1.year

assert(report1.period, "2019-4")
assert(report1.label, "P2019")

# report1's month is deleted
assert(delete report1.month, true)

assert(report1.period, null)
assert(report1.label, "P2019")

# Deleting it again removes nothing
assert(delete report1.month, false)

---

# Nucleoid does not apply a class-level rule to a plain object

# There is a Tag type
class Tag

# Any tag's kind is "TAG"
$Tag.kind = "TAG"

# tag1 is a Tag
tag1 = Tag()

# plain is an Object, which belongs to no type
plain = Object()

assert(tag1.kind, "TAG")
assert(plain.kind, null)

---

# Nucleoid applies class-level rules in the order received

# There is an Alert type
class Alert

# If any alert's level is greater than 1, then its tone is "LOW"
if $Alert.level > 1:
    $Alert.tone = "LOW"

# If any alert's level is greater than 2, then its tone is "HIGH"
if $Alert.level > 2:
    $Alert.tone = "HIGH"

# If any alert's level is greater than 1, then its tone is "FINAL"
if $Alert.level > 1:
    $Alert.tone = "FINAL"

# alert1 is an Alert whose level is 5
alert1 = Alert()
alert1.level = 5

assert(alert1.tone, "FINAL")

# alert2 is an Alert whose level is 0
alert2 = Alert()
alert2.level = 0

assert(alert2.tone, null)

---

# Nucleoid lets an instance assignment override the rule for that instance only

# There is a Node type
class Node

# Any node's status is "IDLE"
$Node.status = "IDLE"

# node1 and node2 are Nodes
node1 = Node()
node2 = Node()

assert(node1.status, "IDLE")
assert(node2.status, "IDLE")

# node1's status is "ACTIVE"
node1.status = "ACTIVE"

assert(node1.status, "ACTIVE")
assert(node2.status, "IDLE")

---

# Nucleoid lets a subtype rule override the supertype rule

# There is a Payment type
class Payment

# Any payment's method is "CARD"
$Payment.method = "CARD"

# There is a Wire type, which is a subtype of Payment
class Wire: Payment
    pass

# Any wire's method is "TRANSFER"
$Wire.method = "TRANSFER"

# payment1 is a Payment and wire1 is a Wire
payment1 = Payment()
wire1 = Wire()

assert(payment1.method, "CARD")
assert(wire1.method, "TRANSFER")

---

# Nucleoid follows a class-level rule through a reference into another type

# There is a Country type
class Country

# There is a City type
class City

# There is a Store type
class Store

# country1 is a Country whose code is "US"
country1 = Country()
country1.code = "US"

# city1 is a City named "NY" in country1
city1 = City()
city1.name = "NY"
city1.country = country1

# Any store's label is its city's country code and its city's name
$Store.label = $Store.city.country.code + "-" + $Store.city.name

# store1 is a Store in city1
store1 = Store()
store1.city = city1

assert(store1.label, "US-NY")

# country1's code is "CA"
country1.code = "CA"

assert(store1.label, "CA-NY")

---

# Nucleoid reads zero as false in a condition

# count is 0
count = 0

# flag is "NO"
flag = "NO"

# if there is a count, then flag is "YES"
if count:
    flag = "YES"

assert(flag, "NO")

# count is 3
count = 3

assert(flag, "YES")

---

# Nucleoid registers a subtype instance in the subtype list

# There is a Vehicle type, which has a make as a string
class Vehicle(make: str):
    this.make = make

# There is a Truck type, which is a subtype of Vehicle
class Truck: Vehicle
    def init make payload:
        super(make)
        this.payload = payload

# truck1 is a Truck whose make is "Volvo" and whose payload is 12000
truck1 = Truck("Volvo", 12000)

assert(truck1, { "id": "truck1", "make": "Volvo", "payload": 12000 })
assert(Truck.length, 1)

---

# Nucleoid provides the built-in members the reference lists

# Math
assert(Math.pow(2, 10), 1024)
assert(Math.sqrt(144), 12)
assert(Math.floor(2.9), 2)
assert(Math.round(2.5), 3)
assert(Math.round(0.49999999999999994), 0)
assert(Math.round(-0.49999999999999994), 0)
assert(Math.round(-0.5000000000000001), -1)
assert(Math.round(1.4999999999999998), 1)
assert(Math.round(1.5), 2)
assert(Math.round(-1.5), -1)
assert(1 / Math.round(-0.49999999999999994), Number.NEGATIVE_INFINITY)
assert(Math.round(4503599627370495.5), 4503599627370496)
assert(Math.round(4503599627370497), 4503599627370497)
assert(Math.max(1, 5, 3), 5)
assert(Math.min(1, 5, 3), 1)
assert(String(Math.max(1, Number.NaN)), "NaN")
assert(String(Math.min(1, "invalid")), "NaN")

# Number
assert(Number.MAX_INTEGER, 9007199254740991)

# String, as a namespace and on an instance
assert(String.fromCharCode(65), "A")
assert("ABC".lower(), "abc")
assert("abc".charAt(1), "b")
assert("abc".length, 3)
assert("abc".replace("b", "x"), "axc")
assert("ab".repeat(3), "ababab")
assert("a,b,c".split(",").length, 3)

# Date
assert(Date.parse("04 Dec 1995 00:12:00 GMT"), 818035920000)
assert(Date("2019-7-24").getYear(), 119)
assert(Date("2020-1-1").toDateString(), "Wed Jan 01 2020")
assert(Date(818035920000).getTime(), 818035920000)

# Boolean and Object
assert(Boolean(0), false)
assert(Object(), {})

# List
assert([1, 2, 3].length, 3)
assert([3, 1, 2].sort()[0], 1)
assert([1, 2, 3].slice(1).length, 2)
assert([1, 2, 3].join("-"), "1-2-3")
assert([1, 2, 3].map(n => n * 2)[2], 6)
assert([1, 2, 3].filter(n => n > 1).length, 2)
assert([1, 2, 3].find(n => n == 2), 2)
assert([1, 2, 3].reduce((sum, n) => sum + n, 0), 6)
assert([1, 2, 3].some(n => n > 2), true)
assert([1, 2, 3].every(n => n > 0), true)

# Class
assert(Class.length, 0)

count = Class.length

class Entity

assert(count, 1)

---

# Nucleoid keeps compound snapshots fixed in the reference

seed = 1
live = 10
values = [seed.value, live]
label = `${seed.value}-${live}`
record = { "seed": seed.value, live }
start = 1
end = 3
part = "abcd"[start.value:end]

seed = 2
live = 20
start = 2
end = 4

assert(values, [1, 20])
assert(label, "1-20")
assert(record, { "seed": 1, "live": 20 })
assert(part, "bcd")

---

# Nucleoid updates inherited rules and respects subtype precedence in the reference

class Parent(amount):
    this.amount = amount

class Child: Parent
    def init amount:
        super(amount + 1)

class Leaf: Child
    def init amount:
        super(amount * 2)

leaf = Leaf(3)

assert(leaf.amount, 7)

$Parent.flag = 1

assert(leaf.flag, 1)

$Leaf.flag = 7
$Parent.flag = 2

assert(leaf.flag, 7)

---

# Nucleoid validates inherited classes and indexed reads in the reference

class Parent

class Child: Parent
    pass

try:
    class Parent: Child
        pass
catch error:
    assert(error, TypeError("Circular Inheritance"))

try:
    $Child.part = "abcd"[:missing]
catch error:
    assert(error, ReferenceError("missing is not defined"))

$Child.list = [$Child.amount]

try:
    $Child.amount = $Child.list[0]
catch error:
    assert(error, TypeError("Circular Dependency"))

child = Child()
child.amount = null
total = child["amount"] + 1

assert(total, null)

child.amount = 4

assert(total, 5)

---

# Nucleoid settles the whole queued chain before its dependent in the reference

source = 1
middle = source + 1
last = middle + 1
total = source + last

middle = source + 2
source = 2

assert(middle, 4)
assert(last, 5)
assert(total, 7)

---

# Nucleoid preserves conditional priority after selective updates in the reference

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

---

# Nucleoid revisits later changes during propagation in the reference

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

---

# Nucleoid keeps conditional loop bodies one-shot in the reference

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

---

# Nucleoid cancels only an aborted body's queued work in the reference

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

---

# Nucleoid finishes temporary deletion reads before continuing in the reference

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

---

# Nucleoid keeps live reasoning impact selections current

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

---

# Nucleoid keeps live reasoning current through equal valued reference changes

class Plan(rate):
    this.rate = rate

class User

basic = Plan(10)
pro = Plan(10)
user = User()
user.plan = basic
user.bill = user.plan.rate * 2
trace = why user.bill

user.plan = pro

assert(trace[0].holds, 20)
assert(trace[0].from, ["user", "user.plan", "pro.rate"])
assert(trace.some(step => step.node == "pro.rate"), true)
assert(trace.some(step => step.node == "basic.rate"), false)

pro.rate = 15

assert(trace[0].holds, 30)

---

# Nucleoid keeps live reasoning model selections current

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

---

# Nucleoid keeps live reasoning subscribed to an absent property

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

---

# Nucleoid keeps live reasoning observers outside impact walks

a = 1
b = a + 1
trace = why b
size = trace.length
exposure = affects a

assert(exposure.map(step => step.node), ["b"])

secondTrace = why b
secondSize = secondTrace.length

assert(exposure.map(step => step.node), ["b"])

---

# Nucleoid keeps live reasoning snapshots fixed

a = 1
exposure = affects a
frozen = exposure.value

b = a + 1

assert(exposure.map(step => step.node), ["b"])
assert(frozen, [])

a = 2

assert(exposure[0].holds, 3)
assert(frozen, [])

---

# Nucleoid restores live reasoning subscriptions after a caught failure

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

---

# Nucleoid keeps live reasoning current through function calls

a = 1

def explain() => affects a

exposure = explain()
b = a + 1

assert(exposure.map(step => step.node), ["b"])
assert((affects a).map(step => step.node), ["b"])

b = 0

assert(exposure, [])

---

# Nucleoid enforces live reasoning guards on structural changes

source = 1
limit = 1

if (affects source).length > limit:
    throw "TOO_MANY_DEPENDENTS"

first = source + 1

try:
    second = source + 2
catch error:
    assert(error, "TOO_MANY_DEPENDENTS")

assert((affects source).map(step => step.node), ["first"])

limit = 2
second = source + 2

assert((affects source).map(step => step.node), ["first", "second"])

---

# Nucleoid preserves queued live reasoning across a caught structural failure

trigger = 0
inventory = model |> why

if trigger > 0:
    try:
        class Temporary
        throw "ABORT"
    catch error:
        if error != "ABORT":
            throw error

remaining = trigger + 10
trigger = 1

assert(inventory.some(step => step.node == "$Temporary"), false)
assert(inventory.find(step => step.node == "trigger").holds, 1)
assert(inventory.find(step => step.node == "remaining").holds, 11)

---

# Nucleoid uses this and self for the same instance

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

---

# Nucleoid preserves compound assignment relationships
amount = 10
step = 2
amount += step
assert(amount, 12)
step = 3
assert(amount, 15)
amount += step.value
assert(amount, 18)
step = 5
assert(amount, 18)

---

# Nucleoid keeps bodyless types in the reference
class Gate
gate1 = Gate()
$Gate.ready = true
assert(gate1.ready, true)
assert(Class.length, 1)
assert(Gate.length, 1)

---

# Bare zero argument super calls

class Parent:
    def init():
        this.ready = true

class Child: Parent
    def init():
        super
        this.code = 7

child = Child()

assert(child.ready, true)
assert(child.code, 7)

---

# Object property shorthand

amount = 4
unit = "C"
reading = { amount, unit }

assert(reading.amount, 4)
assert(reading.unit, "C")

amount = 6

assert(reading.amount, 6)

---

# Bare single parameter named arrows

def double amount => amount * 2

assert(double(3), 6)
assert(double(5), 10)

---

# Bare single parameter indented block definitions

def double amount:
    offset = 1
    return amount * 2 + offset

assert(double(3), 7)
assert(double(5), 11)

---

# Bare typed single parameter named arrows

def increment amount: int => amount + 1
def text value: str => value

assert(increment(4), 5)
assert(text("ok"), "ok")

---

# Numeric clamping keeps bounds live

reading = -3
floor = 0
ceiling = 10

bounded = Math.clamp(reading, floor, ceiling)

assert(bounded, 0)

reading = 12

assert(bounded, 10)

ceiling = 8

assert(bounded, 8)

---

# Numeric interpolation keeps endpoints and weights live

start = 0
end = 20
weight = 0.25

position = Math.lerp(start, end, weight)

assert(position, 5)

end = 40

assert(position, 10)

weight = 0.5

assert(position, 20)

---

# Unit-interval saturation keeps live fractions bounded

reading = -0.2
fraction = Math.saturate(reading)

assert(fraction, 0)

reading = 0.6

assert(fraction, 0.6)

reading = 1.4

assert(fraction, 1)

---

# Bare typed single parameter indented block definitions

def double amount: int:
    offset = 1
    return amount * 2 + offset

assert(double(3), 7)
assert(double(5), 11)

---

# Nucleoid preserves logical compound assignment relationships

available = false
fallback = false
available ||= fallback
fallback = true
assert(available, true)
allowed = true
allowed &&= available
assert(allowed, true)
available = false
assert(allowed, false)

---

# Nucleoid preserves bare multiple parameter arrow relationships

scale = 1
def area width height => width * height * scale
width = 3
height = 4
result = area(width, height)
assert(result, 12)
scale = 2
assert(result, 24)
width = 5
assert(result, 40)

---

# Nucleoid preserves bare multiple parameter block relationships

scale = 2
def area width height =>
    width * height * scale
result = area(3, 4)
assert(result, 24)
scale = 3
assert(result, 36)

---

# Nucleoid preserves bare typed multiple parameter arrow relationships

scale = 2
def area width: number, height: number => width * height * scale
result = area(3, 4)
assert(result, 24)
scale = 3
assert(result, 36)

---

# Nucleoid preserves bare typed multiple parameter block relationships

scale = 2
def area width: number, height: number =>
    width * height * scale
result = area(3, 4)
assert(result, 24)
scale = 3
assert(result, 36)

---

# Nucleoid preserves implicit iterator loop class populations

class Task
first = Task()
first.done = false
second = Task()
second.done = false
for Task:
    it.done = true
assert(first.done, true)
assert(second.done, true)

---

# Nucleoid preserves indented named arrow relationships

scale = 2
def area width: number, height: number =>
    width * height * scale
result = area(3, 4)
assert(result, 24)
scale = 3
assert(result, 36)

---

# Nucleoid preserves cubic smoothstep relationships

lower = 0
upper = 8
value = 2
unit = Math.saturate((value - lower) / (upper - lower))
eased = Math.smoothstep(lower, upper, value)
assert(unit, 0.25)
assert(eased, 0.15625)
value = 6
assert(eased, 0.84375)
upper = 16
assert(eased, 0.31640625)

---

# Nucleoid preserves live property relationships in indented else suites

class Signal
signal = Signal()
signal.enabled = false
base = 4
if signal.enabled:
    signal.output = base
else
    signal.output = -base
assert(signal.output, -4)
base = 6
assert(signal.output, -6)
signal.enabled = true
assert(signal.output, 6)
base = 8
assert(signal.output, 8)

---

# Nucleoid preserves quintic smootherstep relationships

lower = 0
upper = 8
value = 2
unit = Math.saturate((value - lower) / (upper - lower))
eased = Math.smootherstep(lower, upper, value)
assert(unit, 0.25)
assert(eased, 0.103515625)
value = 6
assert(eased, 0.896484375)
upper = 16
assert(eased, 0.27520751953125)

---

# Nucleoid preserves floor fractional part relationships

class Signal
signal = Signal()
signal.sample = 1.75
whole = Math.floor(signal.sample)
fraction = Math.fract(signal.sample)
assert(whole, 1)
assert(fraction, 0.75)
signal.sample = -1.5
assert(whole, -2)
assert(fraction, 0.5)
signal.sample = 4.125
assert(fraction, 0.125)

---

# Nucleoid preserves comma-free named arrow relationships

scale = 1
def area width height => width * height * scale
width = 3
height = 4
result = area(width, height)
assert(result, 12)
scale = 2
assert(result, 24)
width = 5
assert(result, 40)
---

# Nucleoid preserves inverse interpolation relationships

start = 2
end = 10
sample = 4
weight = Math.inverseLerp(start, end, sample)
assert(weight, 0.25)
sample = 6
assert(weight, 0.5)
start = 0
assert(weight, 0.6)
end = 12
assert(weight, 0.5)
sample = 18
assert(weight, 1.5)

---

# Nucleoid preserves range remapping relationships

start = 2
end = 10
targetStart = 10
targetEnd = 30
sample = 4
mapped = Math.remap(start, end, targetStart, targetEnd, sample)
assert(mapped, 15)
sample = 6
assert(mapped, 20)
targetStart = 0
assert(mapped, 15)
targetEnd = 40
assert(mapped, 20)
sample = 18
assert(mapped, 80)
---
# Nucleoid preserves comma-free block relationships

scale = 1
def area width height:
    return width * height * scale
width = 3
height = 4
result = area(width, height)
assert(result, 12)
scale = 2
assert(result, 24)
width = 5
assert(result, 40)

---
# Nucleoid flat maps live related collections

class Team(tags):
    this.tags = tags
team1 = Team(["red", "blue"])
team2 = Team(["green"])
tags = Team.flatMap(team => team.tags)
assert(tags, ["red", "blue", "green"])
team1.tags = ["gold"]
assert(tags, ["gold", "green"])
team2.tags.push("silver")
assert(tags, ["gold", "green", "silver"])
---
# Nucleoid normalizes live text records

labels = ["first name", "last name", "first name"]
normalized = labels.map(label => label.replaceAll(" ", "_"))
assert(normalized, ["first_name", "last_name", "first_name"])
labels.push("display name")
assert(normalized, ["first_name", "last_name", "first_name", "display_name"])
labels = ["given name", "last name"]
assert(normalized, ["given_name", "last_name"])
---
# Nucleoid routes records through direct alternatives

class Job
urgent = Job()
urgent.state = "ready"
urgent.force = true
normal = Job()
normal.state = "queued"
normal.force = false
if $Job.state == "blocked":
    $Job.lane = "hold"
else $Job.force:
    $Job.lane = "fast"
elif $Job.state == "ready":
    $Job.lane = "send"
else:
    $Job.lane = "wait"
assert(urgent.lane, "fast")
assert(normal.lane, "wait")
normal.state = "ready"
assert(normal.lane, "send")
urgent.state = "blocked"
assert(urgent.lane, "hold")
normal.force = true
assert(normal.lane, "fast")
---
# Nucleoid builds compact routing records

def lane(urgent):
    if urgent: return "fast"
    return "normal"
kind = "invoice"
urgent = false
summary = {kind: kind lane: lane(urgent) active: true}
assert(summary.kind, "invoice")
assert(summary.lane, "normal")
urgent = true
assert(summary.lane, "fast")
kind = "alert"
assert(summary.kind, "alert")
---
# Nucleoid preserves assert names and declarations beside compact checks

assert = 12
def identity(assert) => assert
result = identity(4)
record = {assert: "field", reading: 4}
assert: result, 4
assert: record.assert, "field"
assert: record.reading, 4
```
