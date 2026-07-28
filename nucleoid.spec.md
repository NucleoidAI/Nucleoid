```
Project: Nucleoid is a Logic Programming Language for LLMs
Syntax: Minimum tokenized syntax with flexible grammar, which is a superset of Python, JavaScript/TypeScript, Kotlin, Go, Rust, Java, C# and C/C++.

# Nucleoid runs a statement in the state

# i is 1
i = 1

assert(i == 1, True)

---

# Nucleoid runs a expression statement

# j is 1
j = 1

assert(j + 2, 3)

---

# Nucleoid throws an error if variable is not defined

try:
    # t is e plus 1
    t = e + 1
catch err:
    assert(err, ReferenceError("e is not defined"))

---

# Nucleoid throws an error inside a block

# k is 99
k = 99

try:
    # if k is greater than or equal to 99, then throw "INVALID"
    if k >= 99:
        raise "INVALID"
catch err:
    assert(err, "INVALID")

---

# Nucleoid creates a class with constructor

# There is a Shape type,
# which has a type as a string
class Shape(self, type: str):
    self.type = type

# shape1 is a Person whose type is "Square"
shape1 = Shape("Square")

assert(shape1, { "id": "shape1", "type": "Square" })

---

# Nucleoid creates a class with a constructor and a typed attribute

# There is a Shape type,
# which has a type as a string
class Shape:
    type: str

    def init(self, type: str):
        self.type = type

# shape1 is a Shape whose type is "Rectangle"
shape1 = Shape("Rectangle")

assert(shape1, { "id": "shape1", "type": "Rectangle" })

---

# Nucleoid adds an object to the class's object list

# There is a Student type
class Student:
    pass

# user0 is a Student
user0 = Student()

assert(Student.find(student => student.id == "user0"), { "id": "user0" })
assert(Student["user0"], { "id": "user0" })

---

# Nucleoid places an instance in the list of the class when created

# There is a Student type
class Student:
    pass

assert(isinstance(Student, list), True)

# student1 is a Student
student1 = Student()

assert(Student.length, 1)

---

# Nucleoid creates a class and a subclass

# There is a Person type,
# which has a name as a string
class Person(name: str):
    self.name = name

# There is a Student type,
# which is a subtype of Person.
# and has a school as a string
class Student: Person
    def init(self, name, school):
		    super(name)
        self.name = name
        self.school = school

# student1 is a Student,
# whose name is "Joe"
# and whose school is "Riverside High".
student1 = Student("Emma", "Riverside High")

assert(person1, { "id": "student1", "name": "Emma", "school": "Riverside High" })

---

# Nucleoid runs a class-level property assignment

# There is a Human type,
# which has a name as a string
class Human(name: str):
    self.name = name

# All humans are mortal
$Human.mortal = True

# human1 is a Human whose name is "Socrates"
human1 = Human("Socrates")

assert(human1.mortal, True)

---

# Nucleoid runs a class-level conditional

# There is a Device type,
# which has a profile as a string
class Device(profile: str):
    self.profile = profile

# Any device that has a profile is active
if $Device.profile:
    $Device.active = True

# device1 has no profile
device1 = Device()

# device2 has profile "PROFILE-1"
device2 = Device("PROFILE-1")

assert(device1.active, None)
assert(device2.active, True)

---

# creates an instance in a block and assigns it to a property

# There is a Room type
class Room:
    pass

# There is a Meeting type
class Meeting:
    pass

# room1 is a Room
room1 = Room()

# Any meeting's time is now plus " @ " plus the meeting's date as a date string
$Meeting.time = Date.now() + " @ " + $Meeting.date.toDateString()

# meeting is a Meeting whose date is January 1, 2020,
# and room1's meeting is meeting
{
    meeting = Meeting()
    meeting.date = Date("2020-1-1")
    room1.meeting = meeting
}

assert(room1.meeting.date.toDateString(), "Wed Jan 01 2020")
assert(room1.meeting.time[-17:], "@ Wed Jan 01 2020")

---

# creates nested instances in a block and assigns them to a property

# There is a Timesheet type
class Timesheet:
    pass

# There is a Task type
class Task:
    pass

# There is a Project type
class Project:
    pass

# Any project's code is "N-" plus the project's number
$Project.code = "N-" + $Project.number

# timesheet1 is a Timesheet
timesheet1 = Timesheet()

# task is a Task whose project is a Project whose number is 3668347,
# and timesheet1's task is task
{
    task = Task()
    task.project = Project()
    task.project.number = 3668347
    timesheet1.task = task
}

assert(timesheet1.task.project.number, 3668347)
assert(timesheet1.task.project.code, "N-3668347")

---

# Nucleoid creates a local variable in a block and uses in assignment

# integer is 30
integer = 30
equivalency = None

# while in the block, division is a local variable that is integer divided by 10,
# and equivalency is division times 10
{
    division = integer / 10
    equivalency = division * 10
}

assert(equivalency, 30)

# integer is 40
integer = 40

assert(equivalency, 40)

---

# Nucleoid creates and assigns an instance to a local variable inside a block

# There is a Device type
class Device:
    pass

# Any device's renewal is the device's creation time plus 604800000
$Device.renew = $Device.created + 604800000

# device is a Device whose creation time is now
{
    device = Device()
    device.created = Date.now()
}

assert(Device[0].renew - Device[0].created, 604800000)

---

# Nucleoid creates and assigns an instance with a constructor to a local variable inside a block

# There is a Member type,
# which has a first as a string
# and a last as a string
class Member(first: str, last: str):
    self.first = first
    self.last = last

# Any member's display is the member's last plus ", " plus the member's first
$Member.display = $Member.last + ", " + $Member.first

# member is a Member whose first is "First" and whose last is "Last"
{
    member = Member("First", "Last")
}

assert(Member[0].display, "Last, First")

---

# Nucleoid creates an object in a block and assigns it to a class-level property before instantiation

# There is a Member type
class Member:
    pass

# registration is an Object whose date is January 2, 2019,
# and any member's registration is registration
{
    registration = Object()
    registration.date = Date("2019-1-2")
    $Member.registration = registration
}

# member1 is a Member
member1 = Member()

assert(member1.registration.date.toDateString(), "Wed Jan 02 2019")
assert(member1.registration.age, None)

---

# Nucleoid creates an object in a block and assigns it to a class-level property after instantiation

# There is a Distance type
class Distance:
    pass

# distance1 is a Distance
distance1 = Distance()

# location is an Object whose coordinates is "40.6976701,-74.2598779",
# and any distance's starting point is location
{
    location = Object()
    location.coordinates = "40.6976701,-74.2598779"
    $Distance.startingPoint = location
}

assert(distance1.startingPoint.coordinates, "40.6976701,-74.2598779")
assert(distance1.startingPoint.print, None)

---

# Nucleoid calls function in an assignment

# multiply returns the product of two factors
def multiply(first_factor, second_factor)
    product = first_factor * second_factor
    return product

# x is 1
x = 1

# y is 2
y = 2

# z is the result of the multiply function call with x and y, plus 1
z = multiply(x, y) + 1

assert(z, 3)

---

# Nucleoid assigns a block in a function as a dependency

# There is a Student type
class Student:
    pass

# student1 is a Student whose age is 7
student1 = Student()
student1.age = 7

# student2 is a Student whose age is 8
student2 = Student()
student2.age = 8

# student3 is a Student whose age is 9
student3 = Student()
student3.age = 9

# age is 8
age = 8

# student is the Student whose age is age
student = Student.find(s => s.age == age)

assert(student, student2)
assert(student, { "id": "student2", "age": 8 })

# age is 9
age = 9

assert(student, student3)
assert(student, { "id": "student3", "age": 9 })

---

# Nucleoid supports chained functions with a parameter in an expression

# There is a Result type,
# which has a score as a number
class Result(score: int):
    self.score = score

# There are Results whose scores are 10, 15 and 20
Result(10); Result(15); Result(20)

# upperThreshold is 18
upperThreshold = 18

# lowerThreshold is 12
lowerThreshold = 12

# list is Results whose score is greater than lowerThreshold,
# filtered to those whose score is less than upperThreshold
list = Result.filter(r => r.score > lowerThreshold).filter(r => r.score < upperThreshold)

assert(list.length, 1)
assert(list[0].score, 15)

# lowerThreshold is 7
lowerThreshold = 7

assert(list.length, 2)
assert(list[0].score, 10)
assert(list[1].score, 15)

# upperThreshold is 14
upperThreshold = 14

assert(list.length, 1)
assert(list[0].score, 10)

---

# Nucleoid assigns a variable declaratively

# a is 1
a = 1

# b is 2
b = 2

# c is sum of a and b
c = a + b

assert(c, 3)

# a is 2
a = 2

assert(c, 4)

---

# Nucleoid creates if statement of variable

# m is false
m = False

# n is false
n = False

# if m is true, then n is m and true
if m == True:
    n = m and True

assert(n, False)

# m is true
m = True

assert(n, True)

---

# Nucleoid updates if block of variable

# p is 0.01
p = 0.01

# s is 0.02
s = 0.02

# if p is less than 1, then r is p times 10
if p < 1:
    r = p * 10

# if p is less than 1, then r is s times 10
if p < 1:
    r = s * 10

assert(r, 0.2)

# s is 0.03
s = 0.03

assert(r, 0.3)

---

# Nucleoid creates else if statement of variable

# g is 11
g = 11

# earth is 9.8
earth = 9.8

# mars is 3.71
mars = 3.71

# mass is 10
mass = 10

# if g is greater than 9, then weight is earth times mass,
# else if g is greater than 3, then weight is mars times mass
if g > 9:
    weight = earth * mass
elif g > 3:
    weight = mars * mass

# g is 5
g = 5

assert(weight, 37.1)

# mars is 3.72
mars = 3.72

assert(weight, 37.2)

---

# Nucleoid creates multiple else if statement of variable

# fraction is -0.1
fraction = -0.1

# point is 1
point = 1

# if fraction is greater than 1, then score is fraction times point times 3,
# else if fraction is greater than 0, then score is fraction times point times 2,
# else score is fraction times point
if fraction > 1:
    score = fraction * point * 3
elif fraction > 0:
    score = fraction * point * 2
else:
    score = fraction * point

assert(score, -0.1)

# point is 2
point = 2

assert(score, -0.2)

---

# Nucleoid runs dependencies in order as received

# any is 0
any = 0

# if any is bigger than 1, then result is 1
if any > 1:
    result = 1

# if any is bigger than 2, then result is 2
if any > 2:
    result = 2

# if any is bigger than 3, then result is 3
if any > 3:
    result = 3

# if any is bigger than 2, then result is 4
if any > 2:
    result = 4

# if any is bigger than 1, then result is 5
if any > 1:
    result = 5

# any is 4
any = 4

assert(result, 5)

---

# Nucleoid uses local variable at lowest scope as priority

# pi is 3.14
pi = 3.14

# number is pi
number = pi

# a local pi shadows the outer pi inside the block
{
    pi = 3.141
    number = pi
}

assert(number, 3.141)

---

# Nucleoid assigns undefined if any dependency in expression is undefined

# There is a Person type
class Person:
    pass

# person1 is a Person
person1 = Person()

# person1's last name is "Brown"
person1.lastName = "Brown"

# person1's full name is first and last name
person1.fullName = person1.firstName + " " + person1.lastName

assert(person1.fullName, None)

---

# Nucleoid assigns null if there is null pointer in expression

# There is a Product type
class Product:
    pass

# product1 is a Product
product1 = Product()

# score is product1's quality score
score = product1.quality.score

assert(score, None)

---

# Nucleoid creates a function in state

# generate returns number times 10
def generate(number):
    return number * 10

# random is 10
random = 10

# number is the result of the generate function call with random
number = generate(random)

assert(number, 100)

# random is 20
random = 20

assert(number, 200)

---

# Nucleoid assigns a function as a dependency

# list is an empty list
list = []

# count is the list filtered to odd numbers
count = list.filter(n => n % 2)

# Add 1 to the list
list.push(1)

assert(count.length, 1)

# Add 2 to the list
list.push(2)

assert(count.length, 1)

# Add 3 to the list
list.push(3)

assert(count.length, 2)

# Remove the last item from the list
list.pop()

assert(count.length, 1)

---

# Nucleoid assigns a parameter in a function as a dependency

# str1 is "ABC"
str1 = "ABC"

# str2 is str1 lowercased plus "d"
str2 = str1.lower() + "d"

# str3 is str2 concatenated with str1
str3 = str2 + str1

assert(str2, "abcd")
assert(str3, "abcdABC")

# str1 is "AAA"
str1 = "AAA"

assert(str2, "aaad")
assert(str3, "aaadAAA")

---

# Nucleoid uses value property to indicate using only value of variable

# goldenRatio is 1.618
goldenRatio = 1.618

# altitude is 10
altitude = 10

# width is goldenRatio's value times altitude
width = goldenRatio.value * altitude

# depth is goldenRatio's value times altitude
depth = goldenRatio.value * altitude

assert(width, 16.18)
assert(depth, 16.18)

# goldenRatio is 1.62
goldenRatio = 1.62

assert(width, 16.18)
assert(depth, 16.18)

# altitude is 100
altitude = 100

assert(width, 161.8)
assert(depth, 161.8)

---

# Nucleoid creates a nested object in a block and assigns it to a class-level property before instantiation

# There is an Account type
class Account:
    pass

# There is a Balance type
class Balance:
    pass

# There is a Currency type
class Currency:
    pass

# Any currency's description is "Code:" plus the currency's code
$Currency.description = "Code:" + $Currency.code

# balance is an Object whose currency is an Object whose code is "USD",
# and any account's balance is balance
{
    balance = Object()
    balance.currency = Object()
    balance.currency.code = "USD"
    $Account.balance = balance
}

# account1 is an Account
account1 = Account()

assert(account1.balance.currency.code, "USD")
assert(account1.balance.currency.description, None)

---

# creates a nested object in a block and assigns it to a class-level property after instantiation

# There is a Warehouse type
class Warehouse:
    pass

# warehouse1 is a Warehouse
warehouse1 = Warehouse()

# inventory is an Object whose item is an Object whose sku is "699546085767",
# and any warehouse's inventory is inventory
{
    inventory = Object()
    inventory.item = Object()
    inventory.item.sku = "699546085767"
    $Warehouse.inventory = inventory
}

assert(warehouse1.inventory.item.sku, "699546085767")
assert(warehouse1.inventory.item.description, None)

---

# Nucleoid creates an instance inside a block

# There is a Device type,
# which has a name as a string
class Device(name: str):
    self.name = name

# No device is deleted
$Device.deleted = False

# Any device's key is "X-" plus the device's name
$Device.key = "X-" + $Device.name

# name is "Hall",
# and device1 is a Device whose name is name
{
    name = "Hall"
    device1 = Device(name)
}

assert(device1.name, "Hall")
assert(device1.key, "X-Hall")
assert(device1.deleted, False)

---

# Nucleoid creates an instance inside a block without a variable name

# There is a Summary type,
# which has a rate as a number
class Summary(rate: int):
    self.rate = rate

# Any summary's score is the summary's rate times 100
$Summary.score = $Summary.rate * 100

# rate is 4,
# and there is a Summary whose rate is rate
{
    rate = 4
    Summary(rate)
}

assert(Summary[0].rate, 4)
assert(Summary[0].score, 400)

---

# Nucleoid creates a local variable inside a block

# a is 5
a = 5

# b is 10
b = 10

# if a is greater than 9, then while in the block, c is a plus b,
# and d is c times 10
if a > 9:
    c = a + b
    d = c * 10

# a is 10
a = 10

assert(d, 200)

# a is 15
a = 15

assert(d, 250)

# b is 20
b = 20

assert(d, 350)

---

# Nucleoid runs a local variable as an object before declaration

# There is a Plane type
class Plane:
    pass

# There is a Trip type
class Trip:
    pass

# plane1 is a Plane whose speed is 903
plane1 = Plane()
plane1.speed = 903

# trip1 is a Trip whose distance is 5540
trip1 = Trip()
trip1.distance = 5540

# while in the block trip is any plane's trip,
# and any plane's time is trip's distance divided by the plane's speed
{
    trip = $Plane.trip
    $Plane.time = trip.distance / $Plane.speed
}

# plane1's trip is trip1
plane1.trip = trip1

assert(plane1.time, 6.135105204872647)

---

# Nucleoid runs a local variable as an object after declaration

# There is a Seller type
class Seller:
    pass

# There is a Commission type
class Commission:
    pass

# seller1 is a Seller whose sales is 1000000
seller1 = Seller()
seller1.sales = 1000000

# comm1 is a Commission whose rate is 0.05
comm1 = Commission()
comm1.rate = 0.05

# seller1's commission is comm1
seller1.commission = comm1

# While in the block, commission is any seller's commission,
# and any seller's pay is the seller's sales times commission's rate
{
    commission = $Seller.commission
    $Seller.pay = $Seller.sales * commission.rate
}

assert(seller1.pay, 50000)

---

# Nucleoid assigns a property on a local variable after initialization

# There is a Stock type
class Stock:
    pass

# There is a Trade type
class Trade:
    pass

# stock1 is a Stock whose price is 100
stock1 = Stock()
stock1.price = 100

# trade1 is a Trade whose quantity is 1
trade1 = Trade()
trade1.quantity = 1

# stock1's trade is trade1
stock1.trade = trade1

# While in the block, trade is any stock's trade,
# and trade's worth is the stock's price times trade's quantity
{
    trade = $Stock.trade
    trade.worth = $Stock.price * trade.quantity
}

assert(trade1.worth, 100)

---

# Nucleoid reassigns a shadowing local variable in a nested block

# barcode is "barcode"
barcode = "barcode"

# a local barcode shadows the outer barcode inside the block,
# and a nested block reassigns the local barcode to "barcode2"
{
    barcode = "barcode"
    {
        barcode = "barcode2"
        {
            assert(barcode, "barcode2")
        }
    }
}

assert(barcode, "barcode")

---

---

# Nucleoid holds the result of a function in a local variable

# bugs is an empty list
bugs = []

# ticket is 1
ticket = 1

# There is a Bug type
class Bug:
    pass

# bug1 is a Bug whose ticket is 1 and whose priority is "LOW",
# and bug1 is added to bugs
bug1 = Bug()
bug1.ticket = 1
bug1.priority = "LOW"
bugs.push(bug1)

# bug2 is a Bug whose ticket is 2 and whose priority is "MEDIUM",
# and bug2 is added to bugs
bug2 = Bug()
bug2.ticket = 2
bug2.priority = "MEDIUM"
bugs.push(bug2)

# While in the block, bug is the bug in bugs whose ticket is ticket,
# and bug is selected
{
    bug = bugs.find(b => b.ticket == ticket)
    bug.selected = True
}

assert(bug1.selected, True)
assert(bug2.selected, None)

# ticket is 2
ticket = 2

assert(bug2.selected, True)

---

# Nucleoid runs a block statement of variable

# h is 1
h = 1

# while in the block, value is a local variable that is h times 2,
# and j is value times 2
{
    value = h * 2
    j = value * 2
}

assert(j, 4)

# h is 2
h = 2

assert(j, 8)

---

# Nucleoid runs a nested block statement of variable

# radius is 10
radius = 10

# while in the block, area is a local variable that is radius squared times 3.14,
# and in a nested block, volume is area times 5
{
    area = Math.pow(radius, 2) * 3.14
    {
        volume = area * 5
    }
}

assert(volume, 1570)

---

# Nucleoid runs a nested if statement of variable

# gravity is 9.8
gravity = 9.8

# time is 10
time = 10

# distance is 480
distance = 480

# target is true
target = True

# while in the block, dist is a local variable that is one half times gravity times time times time,
# and if dist is greater than distance, then hit is target
{
    dist = 1 / 2 * gravity * time * time
    if dist > distance:
        hit = target
}

assert(hit, True)

# target is false
target = False

assert(hit, False)

---

# Nucleoid runs a nested else statement of variable

# percentage is 28
percentage = 28

# density is 0.899
density = 0.899

# substance is "NH3"
substance = "NH3"

# molarConcentration is null
molarConcentration = None

# fallback is 0
fallback = 0

# while in the block, concentration is a local variable that is percentage times density divided by 100 times 1000,
# and if substance is "NH3", then molarConcentration is concentration divided by 17.04,
# else molarConcentration is fallback
{
    concentration = percentage * density / 100 * 1000
    if substance == "NH3":
        molarConcentration = concentration / 17.04
    else:
        molarConcentration = fallback
}

# substance is "NH16"
substance = "NH16"

# fallback is 1
fallback = 1

assert(molarConcentration, 1)

---

# Nucleoid assigns a variable to a reference

# a is 1
a = 1

# b is a
b = a

assert(b, 1)

# a is 2
a = 2

assert(b, 2)

---

# Nucleoid assigns an object to a variable

# There is a Model type
class Model:
    pass

# model1 is a Model
model1 = Model()

assert(isinstance(model1, object), True)

---

# Nucleoid rejects creating an instance if the class does not exist

try:
    # chart1 is a Chart
    chart1 = Chart()
catch err:
    assert(err, ReferenceError("Chart is not defined"))

# There is a Chart type
class Chart:
    pass

# chart1 is a Chart
chart1 = Chart()

try:
    # chart1's plot is a Plot
    chart1.plot = Plot()
catch err:
    assert(err, ReferenceError("Plot is not defined"))

try:
    # any chart's plot is a Plot
    $Chart.plot = Plot()
catch err:
    assert(err, ReferenceError("Plot is not defined"))
    
---

# Nucleoid creates a property assignment before declaration

# There is an Order type
class Order:
    pass

# order1 is an Order
order1 = Order()

# order1's upc is "04061" plus order1's barcode
order1.upc = "04061" + order1.barcode

assert(order1.upc, None)

# order1's barcode is "94067"
order1.barcode = "94067"

assert(order1.upc, "0406194067")

---

# Nucleoid creates a property assignment after declaration

# There is a User type
class User:
    pass

# user1 is a User
user1 = User()

# user1's name is "sample"
user1.name = "sample"

# user1's email is user1's name plus "@example.com"
user1.email = user1.name + "@example.com"

assert(user1.email, "sample@example.com")

# user1's name is "samplex"
user1.name = "samplex"

assert(user1.email, "samplex@example.com")

---

# Nucleoid creates a property assignment only if the instance is defined

# There is a Channel type
class Channel:
    pass

# channel1 is a Channel
channel1 = Channel()

try:
    # channel1's frequency's type is "ANGULAR"
    channel1.frequency.type = "ANGULAR"
catch err:
    assert(err, ReferenceError("channel1.frequency is not defined"))

---

# Nucleoid creates an object and assigns it to a variable

# There is an Item type,
# which has a name as a string
class Item(name: str):
    self.name = name

# item1 is an Item whose name is "NAME-1"
item1 = Item("NAME-1")

assert(item1, { "id": "item1", "name": "NAME-1" })

# item2 is an Item with no name
item2 = Item()

assert(item2, { "id": "item2", "name": None })

---

# Nucleoid creates an object assignment as a property only if the instance is defined

# There is a Worker type
class Worker:
    pass

# There is a Schedule type
class Schedule:
    pass

# worker1 is a Worker
worker1 = Worker()

try:
    # worker1's duty's schedule is a Schedule
    worker1.duty.schedule = Schedule()
catch err:
    assert(err, ReferenceError("worker1.duty is not defined"))

---

# Nucleoid uses only the value when a property references itself

# There is a Construction type
class Construction:
    pass

# construction1 is a Construction
construction1 = Construction()

# construction1's timeline is 120
construction1.timeline = 120

# construction1's timeline is 2 times construction1's timeline
construction1.timeline = 2 * construction1.timeline

assert(construction1.timeline, 240)

---

# Nucleoid assigns an object to a property before initialization

# There is an Agent type
class Agent:
    pass

# There is a Distance type
class Distance:
    pass

# Any distance's total is the square root of the distance's x squared plus the distance's y squared
$Distance.total = Math.sqrt($Distance.x * $Distance.x + $Distance.y * $Distance.y)

# agent1 is an Agent
agent1 = Agent()

# agent1's distance is a Distance
agent1.distance = Distance()

assert(agent1.distance.total, None)

# agent1's distance's x is 3 and whose y is 4
agent1.distance.x = 3
agent1.distance.y = 4

assert(agent1.distance.total, 5)

---

---

# Nucleoid assigns an object to a property after initialization

# There is a Product type
class Product:
    pass

# product1 is a Product
product1 = Product()

# There is a Quality type
class Quality:
    pass

# product1's quality is a Quality whose score is 15
product1.quality = Quality()
product1.quality.score = 15

assert(product1.quality.class, None)

# Any quality's class is the character with the code 65 plus the floor of the quality's score divided by 10
$Quality.class = String.fromCharCode(65 + Math.floor($Quality.score / 10))

assert(product1.quality.class, "B")

---

# Nucleoid rejects value as a property name

# There is a Schedule type
class Schedule:
    pass

# There is a Place type
class Place:
    pass

# value is a Schedule
value = Schedule()

assert(value, { "id": "value" })

try:
    # value's value is a Place
    value.value = Place()
catch err:
    assert(err, TypeError("Cannot use 'value' as a property"))

---

# Nucleoid rejects value as a property name

# There is a Schedule type
class Schedule:
    pass

# There is a Place type
class Place:
    pass

# value is a Schedule
value = Schedule()

assert(value, { "id": "value" })

try:
    # value's value is a Place
    value.value = Place()
catch err:
    assert(err, TypeError("Cannot use 'value' as a property"))

---

# Nucleoid rejects value as a property name in a value assignment

# There is a Value type
class Value:
    pass

# value is a Value
value = Value()

assert(value, { "id": "value" })

try:
    # value's value is 2147483647
    value.value = 2147483647
catch err:
    assert(err, TypeError("Cannot use 'value' as a name"))

---

# Nucleoid uses value property to indicate using only value of property

# There is a Weight type
class Weight:
    pass

# weight1 is a Weight whose gravity is 1.352 and whose mass is 1000
weight1 = Weight()
weight1.gravity = 1.352
weight1.mass = 1000

# weight1's force is weight1's gravity times weight1's mass's value
weight1.force = weight1.gravity * weight1.mass.value

assert(weight1.force, 1352)

# weight1's mass is 2000
weight1.mass = 2000

assert(weight1.force, 1352)

# weight1's gravity is 2
weight1.gravity = 2

assert(weight1.force, 2000)

---

# Nucleoid uses value property in an if condition to indicate using only value of property

# There is a Question type
class Question:
    pass

# question1 is a Question whose text is "How was the service?"
question1 = Question()
question1.text = "How was the service?"

# if question1's text is not question1's text's value, then throw "QUESTION_ARCHIVED"
if question1.text != question1.text.value:
    raise "QUESTION_ARCHIVED"

assert(question1.text, "How was the service?")

try:
    # question1's text is "How would you rate us?"
    question1.text = "How would you rate us?"
catch err:
    assert(err, "QUESTION_ARCHIVED")

---

# Nucleoid rejects value of a property if the property is not defined

# There is a Travel type
class Travel:
    pass

# travel1 is a Travel whose speed is 65
travel1 = Travel()
travel1.speed = 65

# travel1's duration is travel1's distance divided by travel1's speed
travel1.duration = travel1.distance / travel1.speed

assert(travel1.duration, None)

try:
    # travel1's time is travel1's distance's value divided by travel1's speed
    travel1.time = travel1.distance.value / travel1.speed
catch err:
    assert(err, ReferenceError("travel1.distance is not defined"))

---

# Nucleoid uses the value of a null property as zero

# There is an Interest type
class Interest:
    pass

# interest1 is an Interest whose rate is 3 and whose amount is null
interest1 = Interest()
interest1.rate = 3
interest1.amount = None

# interest1's annual is interest1's rate times interest1's amount's value divided by 100
interest1.annual = interest1.rate * interest1.amount.value / 100

assert(interest1.annual, 0)

# interest1's amount is 10000
interest1.amount = 10000

assert(interest1.annual, 0)

---

# Nucleoid rejects value as a name in a block

# There is an Alarm type
class Alarm:
    pass

try:
    # while in the block, value is a local Alarm,
    # and value's value is "22:00"
    {
        value = Alarm()
        value.value = "22:00"
    }
catch err:
    assert(err, TypeError("Cannot use 'value' in local"))

---

# Nucleoid uses value property in a class-level assignment

# count is 0
count = 0

# There is a Device type
class Device:
    pass

# device1 is a Device
device1 = Device()

# while in the block, any device's code is "A" plus count's value,
# and count is count plus 1
{
    $Device.code = "A" + count.value
    count = count + 1
}

assert(device1.code, "A0")

---

# Nucleoid uses value property on a class-level property chain

# There is a Summary type,
# which has a question as a Question
class Summary(question):
    self.question = question

# There is a Question type
class Question:
    pass

# Any summary's count is the value of the summary's question's count
$Summary.count = $Summary.question.count.value

# question1 is a Question whose count is 10
question1 = Question()
question1.count = 10

# summary1 is a Summary whose question is question1
summary1 = Summary(question1)

assert(summary1.count, 10)

# question1's count is 11
question1.count = 11

assert(question1.count, 11)
assert(summary1.count, 10)

---

# Nucleoid updates if block of property

# There is an Account type
class Account:
    pass

# account1 is an Account whose balance is 1000
account1 = Account()
account1.balance = 1000

# if account1's balance is less than 1500, then account1's status is "OK"
if account1.balance < 1500:
    account1.status = "OK"

assert(account1.status, "OK")

# if account1's balance is less than 1500, then account1's status is "LOW"
if account1.balance < 1500:
    account1.status = "LOW"

assert(account1.status, "LOW")

---

# Nucleoid creates if statement of property

# There is a Toy type
class Toy:
    pass

# toy1 is a Toy whose color is "BLUE"
toy1 = Toy()
toy1.color = "BLUE"

# if toy1's color is "RED", then toy1's shape is "CIRCLE"
if toy1.color == "RED":
    toy1.shape = "CIRCLE"

assert(toy1.shape, None)

# toy1's color is "RED"
toy1.color = "RED"

assert(toy1.shape, "CIRCLE")

---

# Nucleoid creates else statement of property

# There is an Engine type
class Engine:
    pass

# engine1 is an Engine whose type is "V8"
engine1 = Engine()
engine1.type = "V8"

# mpl is "MPL"
mpl = "MPL"

# bsd is "BSD"
bsd = "BSD"

# if engine1's type is "Gecko", then engine1's license is mpl,
# else engine1's license is bsd
if engine1.type == "Gecko":
    engine1.license = mpl
else:
    engine1.license = bsd

assert(engine1.license, "BSD")

# bsd is "Berkeley Software Distribution"
bsd = "Berkeley Software Distribution"

assert(engine1.license, "Berkeley Software Distribution")

# engine1's type is "Gecko"
engine1.type = "Gecko"

assert(engine1.license, "MPL")

---

# Nucleoid creates else statement of property with property dependencies

# There is a Contact type
class Contact:
    pass

# contact1 is a Contact whose type is "PERSON",
# whose first is "First" and whose last is "Last"
contact1 = Contact()
contact1.type = "PERSON"
contact1.first = "First"
contact1.last = "Last"

# if contact1's type is "BUSINESS", then contact1's full is "B" plus contact1's first,
# else contact1's full is contact1's first plus " " plus contact1's last
if contact1.type == "BUSINESS":
    contact1.full = "B" + contact1.first
else:
    contact1.full = contact1.first + " " + contact1.last

assert(contact1.full, "First Last")

# contact1's first is "F" and contact1's last is "L"
contact1.first = "F"
contact1.last = "L"

assert(contact1.full, "F L")

# contact1's type is "BUSINESS"
contact1.type = "BUSINESS"

assert(contact1.full, "BF")

---

# Nucleoid creates multiple else if statement of property

# There is a Taxpayer type
class Taxpayer:
    pass

# taxpayer1 is a Taxpayer whose income is 60000 and whose member is 1
taxpayer1 = Taxpayer()
taxpayer1.income = 60000
taxpayer1.member = 1

# rate is 22
rate = 22

# if taxpayer1's member is greater than 4, then taxpayer1's tax is taxpayer1's income times rate divided by 100 minus 2000,
# else if taxpayer1's member is greater than 2, then taxpayer1's tax is taxpayer1's income times rate divided by 100 minus 1000,
# else taxpayer1's tax is taxpayer1's income times rate divided by 100
if taxpayer1.member > 4:
    taxpayer1.tax = taxpayer1.income * rate / 100 - 2000
elif taxpayer1.member > 2:
    taxpayer1.tax = taxpayer1.income * rate / 100 - 1000
else:
    taxpayer1.tax = taxpayer1.income * rate / 100

assert(taxpayer1.tax, 13200)

# rate is 23
rate = 23

assert(taxpayer1.tax, 13800)

# taxpayer1's member is 3
taxpayer1.member = 3

assert(taxpayer1.tax, 12800)

# taxpayer1's member is 5
taxpayer1.member = 5

assert(taxpayer1.tax, 11800)

---

# Nucleoid updates property assignment

# There is a Matter type
class Matter:
    pass

# matter1 is a Matter whose mass is 10
matter1 = Matter()
matter1.mass = 10

# matter1's weight is matter1's mass times 9.8
matter1.weight = matter1.mass * 9.8

assert(matter1.weight, 98)

# matter1's weight is matter1's mass times 3.7
matter1.weight = matter1.mass * 3.7

assert(matter1.weight, 37)

# matter1's mass is 20
matter1.mass = 20

assert(matter1.weight, 74)
```
