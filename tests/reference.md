# Nucleoid Language Reference - Documented Behaviours

Executable form of the behaviours stated in `docs/README.md`, in the same
case format as `nucleoid.spec.md`. Where this file and the specification
disagree, the specification wins.

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
class User:
    pass

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
    def init(make, payload):
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

# Nucleoid nests for of statements as a cross product

# There is a Size type
class Size:
    pass

# There is a Color type
class Color:
    pass

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
class Reading:
    pass

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
class Node:
    pass

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
class Grade:
    pass

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
else:
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
class Category:
    pass

# There is a Product type
class Product:
    pass

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
class Toy:
    pass

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
class Report:
    pass

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
class Tag:
    pass

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
class Alert:
    pass

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
class Node:
    pass

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
class Payment:
    pass

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
class Country:
    pass

# There is a City type
class City:
    pass

# There is a Store type
class Store:
    pass

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
    def init(make, payload):
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
assert(Math.max(1, 5, 3), 5)
assert(Math.min(1, 5, 3), 1)

# Number
assert(Number.MAX_INTEGER, 9007199254740991)

# String, as a namespace and on an instance
assert(String.fromCharCode(65), "A")
assert("ABC".lower(), "abc")
assert("abc".charAt(1), "b")
assert("abc".length, 3)
assert("abc".replace("b", "x"), "axc")
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
```
