# Nucleoid Language Reference - Synthesized Use Cases 02

```

# Nucleoid deletes a class-level property assignment

# There is a Rate type
class Rate:
    pass

# Any rate's percent is the rate's basis divided by 100
$Rate.percent = $Rate.basis / 100

# rate1 is a Rate whose basis is 250
rate1 = Rate()
rate1.basis = 250

assert(rate1.percent, 2.5)

# Any rate's percent is deleted
delete $Rate.percent

# rate1's basis is 500
rate1.basis = 500

assert(rate1.percent, null)

# rate2 is a Rate whose basis is 100
rate2 = Rate()
rate2.basis = 100

assert(rate2.percent, null)

---

# Nucleoid detects a circular dependency across three variables

# first is 1
first = 1

# second is first plus 1
second = first + 1

# third is second plus 1
third = second + 1

assert(third, 3)

try:
    # first is third plus 1
    first = third + 1
catch error:
    assert(error, TypeError("Circular Dependency"))

assert(first, 1)
assert(third, 3)

---

# Nucleoid detects a circular dependency between class-level properties

# There is a Box type
class Box:
    pass

# Any box's area is the box's width times the box's height
$Box.area = $Box.width * $Box.height

try:
    # any box's width is the box's area divided by the box's height
    $Box.width = $Box.area / $Box.height
catch error:
    assert(error, TypeError("Circular Dependency"))

# box1 is a Box whose width is 3 and whose height is 4
box1 = Box()
box1.width = 3
box1.height = 4

assert(box1.area, 12)

---

# Nucleoid updates dependents when a function is redefined

# fee returns amount times 0.1
def fee(amount):
    return amount * 0.1

# base is 200
base = 200

# charge is the result of the fee function call with base
charge = fee(base)

assert(charge, 20)

# fee returns amount times 0.2
def fee(amount):
    return amount * 0.2

assert(charge, 40)

# base is 300
base = 300

assert(charge, 60)

---

# Nucleoid supports a recursive function

# factorial returns 1 if n is less than or equal to 1,
# else n times the result of the factorial function call with n minus 1
def factorial(n):
    if n <= 1:
        return 1
    return n * factorial(n - 1)

# number is 5
number = 5

# result is the result of the factorial function call with number
result = factorial(number)

assert(result, 120)

# number is 6
number = 6

assert(result, 720)

---

# Nucleoid supports multiple return statements in a function

# sign returns "POSITIVE" if n is greater than 0,
# "NEGATIVE" if n is less than 0,
# else "ZERO"
def sign(n):
    if n > 0:
        return "POSITIVE"
    else if n < 0:
        return "NEGATIVE"
    return "ZERO"

# value is -4
value = -4

# label is the result of the sign function call with value
label = sign(value)

assert(label, "NEGATIVE")

# value is 0
value = 0

assert(label, "ZERO")

# value is 7
value = 7

assert(label, "POSITIVE")

---

# Nucleoid uses the remainder operator in a dependency

# minutes is 135
minutes = 135

# hours is the floor of minutes divided by 60
hours = Math.floor(minutes / 60)

# rest is the remainder of minutes divided by 60
rest = minutes % 60

# duration is hours plus "h" plus rest plus "m"
duration = hours + "h" + rest + "m"

assert(duration, "2h15m")

# minutes is 200
minutes = 200

assert(duration, "3h20m")

---

# Nucleoid maps a list into another list as a dependency

# prices is a list of 10, 20 and 30
prices = [10, 20, 30]

# rate is 2
rate = 2

# doubled is each price of prices times rate
doubled = prices.map(p => p * rate)

assert(doubled.length, 3)
assert(doubled[2], 60)

# rate is 3
rate = 3

assert(doubled[2], 90)

# Add 40 to prices
prices.push(40)

assert(doubled.length, 4)
assert(doubled[3], 120)

---

# Nucleoid reduces a list into a value as a dependency

# amounts is a list of 5, 10 and 15
amounts = [5, 10, 15]

# total is the sum of amounts
total = amounts.reduce((sum, amount) => sum + amount, 0)

assert(total, 30)

# Add 20 to amounts
amounts.push(20)

assert(total, 50)

# Remove the last item from amounts
amounts.pop()

assert(total, 30)

---

# Nucleoid aggregates a class list in a variable

# There is an Order type
class Order:
    pass

# order1 is an Order whose amount is 100
order1 = Order()
order1.amount = 100

# order2 is an Order whose amount is 250
order2 = Order()
order2.amount = 250

# revenue is the sum of the amounts of Orders
revenue = Order.reduce((sum, order) => sum + order.amount, 0)

assert(revenue, 350)

# order3 is an Order whose amount is 50
order3 = Order()
order3.amount = 50

assert(revenue, 400)

# order1's amount is 150
order1.amount = 150

assert(revenue, 450)

---

# Nucleoid checks whether every instance satisfies a condition

# There is a Check type
class Check:
    pass

# check1 is a Check which passed
check1 = Check()
check1.passed = true

# check2 is a Check which passed
check2 = Check()
check2.passed = true

# ready is whether every check passed
ready = Check.every(c => c.passed)

assert(ready, true)

# check2 did not pass
check2.passed = false

assert(ready, false)

---

# Nucleoid checks whether any element satisfies a condition

# levels is a list of 1, 2 and 3
levels = [1, 2, 3]

# limit is 5
limit = 5

# exceeded is whether any level is greater than limit
exceeded = levels.some(l => l > limit)

assert(exceeded, false)

# Add 9 to levels
levels.push(9)

assert(exceeded, true)

# limit is 10
limit = 10

assert(exceeded, false)

---

# Nucleoid sorts a copy of a list as a dependency

# scores is a list of 30, 10 and 20
scores = [30, 10, 20]

# ranked is a copy of scores sorted in ascending order
ranked = scores.slice().sort((a, b) => a - b)

assert(ranked[0], 10)
assert(ranked[2], 30)

# Add 5 to scores
scores.push(5)

assert(ranked[0], 5)
assert(ranked.length, 4)

---

# Nucleoid joins a list into a string as a dependency

# tags is a list of "logic" and "state"
tags = ["logic", "state"]

# separator is ","
separator = ","

# label is tags joined with separator
label = tags.join(separator)

assert(label, "logic,state")

# Add "graph" to tags
tags.push("graph")

assert(label, "logic,state,graph")

# separator is " | "
separator = " | "

assert(label, "logic | state | graph")

---

# Nucleoid splits a string into a list as a dependency

# csv is "NY,GA,CT"
csv = "NY,GA,CT"

# states is csv split by ","
states = csv.split(",")

assert(states.length, 3)
assert(states[1], "GA")

# csv is "NY,GA"
csv = "NY,GA"

assert(states.length, 2)
assert(states[1], "GA")

---

# Nucleoid uses string functions in a class-level property

# There is an Account type
class Account:
    pass

# Any account's slug is the account's name lowercased with spaces replaced by "-"
$Account.slug = $Account.name.lower().replace(" ", "-")

# account1 is an Account whose name is "New York"
account1 = Account()
account1.name = "New York"

assert(account1.slug, "new-york")

# account1's name is "Los Angeles"
account1.name = "Los Angeles"

assert(account1.slug, "los-angeles")

---

# Nucleoid uses a standard built-in function with multiple dependencies

# a is 5
a = 5

# b is 9
b = 9

# peak is the maximum of a and b
peak = Math.max(a, b)

# floor is the minimum of a and b
floor = Math.min(a, b)

assert(peak, 9)
assert(floor, 5)

# b is 3
b = 3

assert(peak, 5)
assert(floor, 3)

---

# Nucleoid rounds a class-level derived property

# There is a Rating type
class Rating:
    pass

# Any rating's stars is the round of the rating's score divided by 20
$Rating.stars = Math.round($Rating.score / 20)

# rating1 is a Rating whose score is 85
rating1 = Rating()
rating1.score = 85

assert(rating1.stars, 4)

# rating1's score is 95
rating1.score = 95

assert(rating1.stars, 5)

---

# Nucleoid creates instances in a nested for of statement

# There is a Row type
class Row:
    pass

# There is a Column type
class Column:
    pass

# There is a Cell type,
# which has a row as a Row
# and a column as a Column
class Cell(row, column):
    this.row = row
    this.column = column

# row1 is a Row, and row2 is a Row
row1 = Row(); row2 = Row()

# column1 is a Column, and column2 is a Column
column1 = Column(); column2 = Column()

# For each row of Row, for each column of Column,
# there is a Cell whose row is the row and whose column is the column
for row of Row:
    for column of Column:
        Cell(row, column)

assert(Cell.length, 4)
assert(Cell[0].row.id, "row1")
assert(Cell[0].column.id, "column1")
assert(Cell[3].row.id, "row2")
assert(Cell[3].column.id, "column2")

---

# Nucleoid rolls back instances created in a for of statement if an exception is thrown

# There is an Item type,
# which has a sku as a string
class Item(sku: str):
    this.sku = sku

# item1 is an Item whose sku is "A1"
item1 = Item("A1")

# item2 is an Item whose sku is "BAD"
item2 = Item("BAD")

# There is a Label type,
# which has an item as an Item
class Label(item):
    this.item = item

# If any label's item's sku is "BAD", then throw "INVALID_ITEM"
if $Label.item.sku == "BAD":
    throw "INVALID_ITEM"

try:
    # For each item of Item, there is a Label whose item is the item
    for item of Item:
        Label(item)
catch error:
    assert(error, "INVALID_ITEM")

assert(Label.length, 0)

---

# Nucleoid rolls back a class-level assignment if an existing instance throws

# There is a Reading type
class Reading:
    pass

# reading1 is a Reading whose celsius is 200
reading1 = Reading()
reading1.celsius = 200

# If any reading's fahrenheit is greater than 300, then throw "OUT_OF_RANGE"
if $Reading.fahrenheit > 300:
    throw "OUT_OF_RANGE"

try:
    # any reading's fahrenheit is the reading's celsius times 9 divided by 5, plus 32
    $Reading.fahrenheit = $Reading.celsius * 9 / 5 + 32
catch error:
    assert(error, "OUT_OF_RANGE")

assert(reading1.fahrenheit, null)

# reading1's celsius is 100
reading1.celsius = 100

# any reading's fahrenheit is the reading's celsius times 9 divided by 5, plus 32
$Reading.fahrenheit = $Reading.celsius * 9 / 5 + 32

assert(reading1.fahrenheit, 212)

---

# Nucleoid runs class-level statements in order as received

# There is an Alert type
class Alert:
    pass

# If any alert's level is greater than 1, then the alert's tone is "LOW"
if $Alert.level > 1:
    $Alert.tone = "LOW"

# If any alert's level is greater than 2, then the alert's tone is "HIGH"
if $Alert.level > 2:
    $Alert.tone = "HIGH"

# If any alert's level is greater than 1, then the alert's tone is "FINAL"
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

# Nucleoid applies a class-level property to a subclass instance

# There is an Animal type,
# which has a name as a string
class Animal(name: str):
    this.name = name

# Any animal's kingdom is "ANIMALIA"
$Animal.kingdom = "ANIMALIA"

# Any animal's label is "A-" plus the animal's name
$Animal.label = "A-" + $Animal.name

# There is a Dog type,
# which is a subtype of Animal
# and has a breed as a string
class Dog: Animal
    def init(name, breed):
        super(name)
        this.breed = breed

# dog1 is a Dog whose name is "Rex" and whose breed is "Husky"
dog1 = Dog("Rex", "Husky")

assert(dog1.kingdom, "ANIMALIA")
assert(dog1.label, "A-Rex")
assert(Dog.length, 1)

# dog1's name is "Max"
dog1.name = "Max"

assert(dog1.label, "A-Max")

---

# Nucleoid overrides a class-level property in a subclass

# There is a Payment type
class Payment:
    pass

# Any payment's method is "CARD"
$Payment.method = "CARD"

# There is a Wire type,
# which is a subtype of Payment
class Wire: Payment
    pass

# Any wire's method is "TRANSFER"
$Wire.method = "TRANSFER"

# payment1 is a Payment
payment1 = Payment()

# wire1 is a Wire
wire1 = Wire()

assert(payment1.method, "CARD")
assert(wire1.method, "TRANSFER")

---

# Nucleoid creates a class-level condition with an and operator

# There is a Loan type
class Loan:
    pass

# If any loan's score is greater than 700 and the loan's income is greater than 50000,
# then the loan is approved
if $Loan.score > 700 and $Loan.income > 50000:
    $Loan.approved = true

# loan1 is a Loan whose score is 720 and whose income is 40000
loan1 = Loan()
loan1.score = 720
loan1.income = 40000

assert(loan1.approved, null)

# loan1's income is 60000
loan1.income = 60000

assert(loan1.approved, true)

---

# Nucleoid creates a class-level condition with an or operator

# There is an Access type
class Access:
    pass

# If any access's role is "ADMIN" or the access is owner, then the access is granted
if $Access.role == "ADMIN" or $Access.owner == true:
    $Access.granted = true

# access1 is an Access whose role is "GUEST"
access1 = Access()
access1.role = "GUEST"

assert(access1.granted, null)

# access1 is owner
access1.owner = true

assert(access1.granted, true)

# access2 is an Access whose role is "ADMIN"
access2 = Access()
access2.role = "ADMIN"

assert(access2.granted, true)

---

# Nucleoid reads a class-level property through a chain of three classes

# There is a Country type
class Country:
    pass

# There is a City type
class City:
    pass

# There is a Store type
class Store:
    pass

# Any store's label is the store's city's country's code plus "-" plus the store's city's name
$Store.label = $Store.city.country.code + "-" + $Store.city.name

# country1 is a Country whose code is "US"
country1 = Country()
country1.code = "US"

# city1 is a City whose country is country1 and whose name is "NY"
city1 = City()
city1.country = country1
city1.name = "NY"

# store1 is a Store whose city is city1
store1 = Store()
store1.city = city1

assert(store1.label, "US-NY")

# country1's code is "CA"
country1.code = "CA"

assert(store1.label, "CA-NY")

---

# Nucleoid assigns null when an intermediate reference is deleted

# There is a Team type
class Team:
    pass

# There is a Player type
class Player:
    pass

# team1 is a Team whose city is "Boston"
team1 = Team()
team1.city = "Boston"

# player1 is a Player whose team is team1
player1 = Player()
player1.team = team1

# player1's origin is player1's team's city
player1.origin = player1.team.city

assert(player1.origin, "Boston")

# player1's team is deleted
delete player1.team

assert(player1.origin, null)

# player1's team is team1
player1.team = team1

assert(player1.origin, "Boston")

---

# Nucleoid uses value property as a function argument

# tag returns prefix plus count
def tag(prefix, count):
    return prefix + count

# seed is 1
seed = 1

# label is the result of the tag function call with "N" and seed's value
label = tag("N", seed.value)

assert(label, "N1")

# seed is 2
seed = 2

assert(label, "N1")

---

# Nucleoid uses value property in a for of statement

# There is a Slot type
class Slot:
    pass

# counter is 0
counter = 0

# slot1 is a Slot, and slot2 is a Slot
slot1 = Slot(); slot2 = Slot()

# For each slot of Slot, the slot's index is counter's value,
# and counter is counter plus 1
for slot of Slot:
    slot.index = counter.value
    counter = counter + 1

assert(slot1.index, 0)
assert(slot2.index, 1)
assert(counter, 2)

# counter is 10
counter = 10

assert(slot1.index, 0)
assert(slot2.index, 1)

---

# Nucleoid rejects an unknown function of a standard built-in string object

try:
    # text is the wrong string function
    text = String.wrong("abc")
catch error:
    assert(error, TypeError("String.wrong is not a function"))

---

# Nucleoid rejects an unknown function of a list

# list is a list of 1 and 2
list = [1, 2]

try:
    # result is the wrong list function
    result = list.wrong()
catch error:
    assert(error, TypeError("list.wrong is not a function"))

---

# Nucleoid throws a reference error when a deleted variable is used in a new assignment

# x is 1
x = 1

# x is deleted
delete x

try:
    # y is x plus 1
    y = x + 1
catch error:
    assert(error, ReferenceError("x is not defined"))

---

# Nucleoid propagates null when a variable with dependents is deleted

# base is 10
base = 10

# tax is base times 0.1
tax = base * 0.1

# total is base plus tax
total = base + tax

assert(tax, 1)
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

# Nucleoid rejects value as a class-level property name

# There is a Meter type
class Meter:
    pass

try:
    # any meter's value is 100
    $Meter.value = 100
catch error:
    assert(error, TypeError("Cannot use 'value' as a property"))

---

# Nucleoid updates a variable dependent on a filtered class list

# There is a Task type
class Task:
    pass

# task1 is a Task which is done
task1 = Task()
task1.done = true

# task2 is a Task which is not done
task2 = Task()
task2.done = false

# open is the number of Tasks that are not done
open = Task.filter(t => not t.done).length

assert(open, 1)

# task2 is done
task2.done = true

assert(open, 0)

# task3 is a Task
task3 = Task()

assert(open, 1)

---

# Nucleoid updates a variable dependent on the length of a class list

# There is a Guest type
class Guest:
    pass

# capacity is 2
capacity = 2

# guest1 is a Guest
guest1 = Guest()

# full is whether the number of Guests is greater than or equal to capacity
full = Guest.length >= capacity

assert(full, false)

# guest2 is a Guest
guest2 = Guest()

assert(full, true)

# guest2 is deleted
delete guest2

assert(full, false)

# capacity is 1
capacity = 1

assert(full, true)

---

# Nucleoid runs a block statement that assigns both a variable and a property

# There is a Cart type
class Cart:
    pass

# cart1 is a Cart whose subtotal is 200
cart1 = Cart()
cart1.subtotal = 200

# shipping is 15
shipping = 15

# while in the block, freight is a local variable that is shipping times 2,
# and cart1's total is cart1's subtotal plus freight,
# and summary is "TOTAL:" plus cart1's total
{
    freight = shipping * 2
    cart1.total = cart1.subtotal + freight
    summary = "TOTAL:" + cart1.total
}

assert(cart1.total, 230)
assert(summary, "TOTAL:230")

# shipping is 20
shipping = 20

assert(cart1.total, 240)
assert(summary, "TOTAL:240")

---

# Nucleoid rolls back a variable if an exception is thrown in a nested block

# threshold is 50
threshold = 50

# while in the block, in a nested block,
# if threshold is greater than 100, then throw "TOO_HIGH"
{
    {
        if threshold > 100:
            throw "TOO_HIGH"
    }
}

try:
    # threshold is 150
    threshold = 150
catch error:
    assert(error, "TOO_HIGH")

assert(threshold, 50)

# threshold is 90
threshold = 90

assert(threshold, 90)

---

# Nucleoid compares a class-level property with a date variable

# There is a Task type
class Task:
    pass

# deadline is June 1, 2020
deadline = Date("2020-6-1")

# If any task's due is after deadline, then the task is late
if $Task.due > deadline:
    $Task.late = true

# task1 is a Task whose due is July 1, 2020
task1 = Task()
task1.due = Date("2020-7-1")

assert(task1.late, true)

# task2 is a Task whose due is May 1, 2020
task2 = Task()
task2.due = Date("2020-5-1")

assert(task2.late, null)

---

# Nucleoid updates a property when a referenced instance is replaced

# There is a Plan type
class Plan:
    pass

# There is a User type
class User:
    pass

# basic is a Plan whose rate is 10
basic = Plan()
basic.rate = 10

# pro is a Plan whose rate is 30
pro = Plan()
pro.rate = 30

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

# Nucleoid creates a dependency on a list index

# options is a list of "A", "B" and "C"
options = ["A", "B", "C"]

# index is 1
index = 1

# choice is the value of options at index
choice = options[index]

assert(choice, "B")

# index is 2
index = 2

assert(choice, "C")

# options is a list of "A", "B" and "D"
options = ["A", "B", "D"]

assert(choice, "D")

---

# Nucleoid creates a dependency on a list held as a property

# There is a Playlist type
class Playlist:
    pass

# playlist1 is a Playlist whose tracks is an empty list
playlist1 = Playlist()
playlist1.tracks = []

# playlist1's count is the length of playlist1's tracks
playlist1.count = playlist1.tracks.length

assert(playlist1.count, 0)

# Add "T1" to playlist1's tracks
playlist1.tracks.push("T1")

assert(playlist1.count, 1)

# Add "T2" to playlist1's tracks
playlist1.tracks.push("T2")

assert(playlist1.count, 2)

# Remove the last track from playlist1's tracks
playlist1.tracks.pop()

assert(playlist1.count, 1)

---

# Nucleoid aggregates another class list in a class-level property

# There is a Category type
class Category:
    pass

# There is a Product type
class Product:
    pass

# Any category's revenue is the sum of the prices of the Products whose category is the category
$Category.revenue = Product.filter(p => p.category == $Category).reduce((sum, p) => sum + p.price, 0)

# category1 is a Category
category1 = Category()

assert(category1.revenue, 0)

# product1 is a Product whose category is category1 and whose price is 100
product1 = Product()
product1.category = category1
product1.price = 100

# product2 is a Product whose category is category1 and whose price is 50
product2 = Product()
product2.category = category1
product2.price = 50

assert(category1.revenue, 150)

# product1's price is 200
product1.price = 200

assert(category1.revenue, 250)

---

# Nucleoid rejects a class-level assignment if the class does not exist

try:
    # any ghost's name is "X"
    $Ghost.name = "X"
catch error:
    assert(error, ReferenceError("Ghost is not defined"))

# There is a Ghost type
class Ghost:
    pass

# Any ghost's name is "X"
$Ghost.name = "X"

# ghost1 is a Ghost
ghost1 = Ghost()

assert(ghost1.name, "X")

---

# Nucleoid rolls back a variable if a function throws

# validate returns n, or throws "NEGATIVE" if n is less than 0
def validate(n):
    if n < 0:
        throw "NEGATIVE"
    return n

# value is 5
value = 5

# checked is the result of the validate function call with value
checked = validate(value)

assert(checked, 5)

try:
    # value is -1
    value = -1
catch error:
    assert(error, "NEGATIVE")

assert(value, 5)
assert(checked, 5)

# value is 8
value = 8

assert(checked, 8)
```
