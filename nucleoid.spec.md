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

# Nucleoid returns value of variable

# k is 1
k = 1

k

# return: 1

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

# Nucleoid supports an array with brackets

# states is a list of "NY", "GA", "CT" and "MI"
states = ["NY", "GA", "CT", "MI"]

# The value of states at index 2
states[2]

# return: "CT"

---

# Nucleoid searches a variable in the block scope before the state

# e is 2.71828
e = 2.71828

# a local e shadows the outer e inside the block,
# and number is the local e
{
    e = 3
    number = e
}

assert(number, 3)

---

# Nucleoid throws an error if a variable in an expression is not defined

try:
    # whether e is equal to 2.71828
    e == 2.71828
catch err:
    assert(err, ReferenceError("e is not defined"))

---

# Nucleoid retrieves the value of a variable

# number is -1
number = -1

number

# return: -1

---

# Nucleoid creates a property assignment on a local variable only if the instance is defined

# There is a Ticket type
class Ticket:
    pass

try:
    # while in the block, ticket is a local Ticket,
    # and ticket's event's group is "ENTERTAINMENT"
    {
        ticket = Ticket()
        ticket.event.group = "ENTERTAINMENT"
    }
catch err:
    assert(err, ReferenceError("ticket.event is not defined"))

---

# Nucleoid declares a local variable as undefined

# There is a Device type,
# which has a code as a string
class Device(code: str):
    self.code = code

# device1 is a Device whose code is "A0"
device1 = Device("A0")

# device2 is a Device whose code is "B1"
device2 = Device("B1")

# While in the block, device is a local variable that is the Device whose code is "A0",
# and if there is no device, then throw "INVALID_DEVICE",
# and return device
{
    device = Device.find(d => d.code == "A0")
    if not device:
        raise "INVALID_DEVICE"
    return device
}

# return: { "id": "device1", "code": "A0" }

---

# Nucleoid rejects a local variable declared as undefined

# There is a Device type,
# which has a code as a string
class Device(code: str):
    self.code = code

# device1 is a Device whose code is "A0"
device1 = Device("A0")

# device2 is a Device whose code is "B1"
device2 = Device("B1")

try:
    # While in the block, device is a local variable that is the Device whose code is "A1",
    # and if there is no device, then throw "INVALID_DEVICE"
    {
        device = Device.find(d => d.code == "A1")
        if not device:
            raise "INVALID_DEVICE"
        return device
    }
catch err:
    assert(err, "INVALID_DEVICE")

---

# Nucleoid creates a standard built-in object as a property of a local variable

# There is a Shipment type
class Shipment:
    pass

# While in the block, shipment is a local Shipment whose date is January 3, 2019,
# and shipment1 is shipment
{
    shipment = Shipment()
    shipment.date = Date("2019-1-3")
    shipment1 = shipment
}

assert(shipment1.date.toDateString(), "Thu Jan 03 2019")

---

# Nucleoid creates a property of a local variable in a different scope

# There is a User type
class User:
    pass

# user0 is a User
user0 = User()

# While in the block, user is a local variable that is the User whose id is "user0",
# and if there is a user, then the user's name is "TEST"
{
    user = User["user0"]
    if user:
        user.name = "TEST"
}

assert(user0.name, "TEST")

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

---

# Nucleoid deletes an instance

# There is a Circle type
class Circle:
    pass

# circle1 is a Circle
circle1 = Circle()

# circle1 is deleted
delete circle1

assert(Circle["circle1"], None)
assert(Circle.find(circle => circle.id == "circle1"), None)

try:
    # circle1
    circle1
catch err:
    assert(err, ReferenceError("circle1 is not defined"))

---

# Nucleoid deletes an instance by reference

# There is an Item type
class Item:
    pass

# item1 is an Item
item1 = Item()

# item2 is an Item
item2 = Item()

assert(Item["item1"], { "id": "item1" })

# The Item whose id is "item1" is deleted
delete Item["item1"]

assert(Item["item1"], None)
assert(Item["item2"], { "id": "item2" })

# While in the block, item is a local variable that is "item2",
# and the Item whose id is item is deleted
{
    item = "item2"
    delete Item[item]
}

assert(Item["item2"], None)

---

# Nucleoid returns a boolean when deleting an object

# There is a Location type
class Location:
    pass

# location1 is a Location
location1 = Location()

# Deleting location1 returns true
assert(delete location1, True)

# Deleting location2, which is not defined, returns false
assert(delete location2, False)

---

# Nucleoid rejects deleting an instance if it has any properties

# There is a Channel type
class Channel:
    pass

# channel1 is a Channel
channel1 = Channel()

# channel1's frequency is 440
channel1.frequency = 440

try:
    # channel1 is deleted
    delete channel1
catch err:
    assert(err, ReferenceError("Cannot delete object 'channel1'"))

assert(channel1.frequency, 440)

# channel1's frequency is deleted
delete channel1.frequency

# channel1 is deleted
delete channel1

assert(Channel["channel1"], None)

---

# Nucleoid rejects deleting an instance if it has an object as a property

# There is a Shape type
class Shape:
    pass

# There is a Type type
class Type:
    pass

# shape1 is a Shape
shape1 = Shape()

# shape1's type is a Type
shape1.type = Type()

try:
    # shape1 is deleted
    delete shape1
catch err:
    assert(err, ReferenceError("Cannot delete object 'shape1'"))

# shape1's type is deleted
delete shape1.type

# shape1 is deleted
delete shape1

assert(Shape["shape1"], None)

---

# Nucleoid deletes a property assignment

# There is an Agent type
class Agent:
    pass

# agent is an Agent
agent = Agent()

# agent's time is 52926163455
agent.time = 52926163455

# agent's location is "CITY"
agent.location = "CITY"

# agent's report is agent's time plus "@" plus agent's location
agent.report = agent.time + "@" + agent.location

assert(agent.report, "52926163455@CITY")

# agent's time is deleted
delete agent.time

assert(agent.report, None)

# agent's report is deleted
delete agent.report

assert(agent.report, None)

---

# Nucleoid runs a block statement of property

# There is an Item type
class Item:
    pass

# item1 is an Item
item1 = Item()

# item1's sku is "0000001"
item1.sku = "0000001"

# while in the block, custom is a local variable that is "US" plus item1's sku,
# and item1's custom is custom
{
    custom = "US" + item1.sku
    item1.custom = custom
}

assert(item1.custom, "US0000001")

# item1's sku is "0000002"
item1.sku = "0000002"

assert(item1.custom, "US0000002")

# Nucleoid runs a nested block statement of property

# There is a Figure type
class Figure:
    pass

# figure1 is a Figure
figure1 = Figure()

# figure1's width is 9
figure1.width = 9

# figure1's height is 10
figure1.height = 10

# while in the block, base is a local variable that is figure1's width squared,
# and in a nested block, figure1's volume is base times figure1's height
{
    base = Math.pow(figure1.width, 2)
    {
        figure1.volume = base * figure1.height
    }
}

assert(figure1.volume, 810)

# figure1's height is 9
figure1.height = 9

assert(figure1.volume, 729)

---

# Nucleoid runs a nested if statement of property

# There is a Sale type
class Sale:
    pass

# sale1 is a Sale
sale1 = Sale()

# sale1's price is 50
sale1.price = 50

# sale1's quantity is 2
sale1.quantity = 2

# while in the block, amount is a local variable that is sale1's price times sale1's quantity,
# and if amount is greater than 100, then sale1's tax is amount times 10 divided by 100
{
    amount = sale1.price * sale1.quantity
    if amount > 100:
        sale1.tax = amount * 10 / 100
}

assert(sale1.tax, None)

# sale1's quantity is 3
sale1.quantity = 3

assert(sale1.tax, 15)

---

# Nucleoid creates a nested else statement of property

# There is a Chart type
class Chart:
    pass

# chart1 is a Chart
chart1 = Chart()

# chart1's percentage is 1
chart1.percentage = 1

# invalid is "INVALID"
invalid = "INVALID"

# valid is "VALID"
valid = "VALID"

# while in the block, ratio is a local variable that is chart1's percentage divided by 100,
# and if ratio is greater than 1, then chart1's status is invalid,
# else chart1's status is valid
{
    ratio = chart1.percentage / 100
    if ratio > 1:
        chart1.status = invalid
    else:
        chart1.status = valid
}

assert(chart1.status, "VALID")

# valid is "V"
valid = "V"

assert(chart1.status, "V")

---

# Nucleoid creates a property assignment with multiple properties

# There is a Person type
class Person:
    pass

# person1 is a Person
person1 = Person()

# There is an Address type
class Address:
    pass

# address1 is an Address
address1 = Address()

# Any address's print is the address's city plus ", " plus the address's state
$Address.print = $Address.city + ", " + $Address.state

# person1's address is an Address
person1.address = Address()

# person1's address's city is "Syracuse"
person1.address.city = "Syracuse"

# person1's address's state is "NY"
person1.address.state = "NY"

assert(person1.address.print, "Syracuse, NY")

---

# Nucleoid creates a property assignment with multiple properties as part of a declaration

# There is a Server type
class Server:
    pass

# server1 is a Server
server1 = Server()

# server1's name is "HOST1"
server1.name = "HOST1"

# There is an IP type
class IP:
    pass

# ip1 is an IP
ip1 = IP()

# server1's ip is ip1
server1.ip = ip1

# ip1's address is "10.0.0.1"
ip1.address = "10.0.0.1"

# server1's summary is server1's name plus "@" plus server1's ip's address
server1.summary = server1.name + "@" + server1.ip.address

assert(server1.summary, "HOST1@10.0.0.1")

# ip1's address is "10.0.0.2"
ip1.address = "10.0.0.2"

assert(server1.summary, "HOST1@10.0.0.2")

---

# Nucleoid creates a dependency on behalf if a property has a reference

# There is a Schedule type
class Schedule:
    pass

# schedule1 is a Schedule
schedule1 = Schedule()

# There is a Template type
class Template:
    pass

# template1 is a Template
template1 = Template()

# template1's type is "W"
template1.type = "W"

# schedule1's template is template1
schedule1.template = template1

# schedule1's template's name is schedule1's template's type plus "-0001"
schedule1.template.name = schedule1.template.type + "-0001"

assert(template1.name, "W-0001")
assert(schedule1.template.name, "W-0001")

# template1's type is "D"
template1.type = "D"

assert(template1.name, "D-0001")

# template1's shape is template1's type plus "-Form"
template1.shape = template1.type + "-Form"

assert(template1.shape, "D-Form")
assert(schedule1.template.shape, "D-Form")

# template1's type is "C"
template1.type = "C"

assert(template1.shape, "C-Form")
assert(schedule1.template.shape, "C-Form")

---

# Nucleoid creates a dependency on behalf if a local variable has a reference

# There is a Vote type
class Vote:
    pass

# vote1 is a Vote whose rate is 4
vote1 = Vote()
vote1.rate = 4

# There is a Question type
class Question:
    pass

# Any question's rate is 0
$Question.rate = 0

# Any question's count is 0
$Question.count = 0

# question1 is a Question
question1 = Question()

# vote1's question is question1
vote1.question = question1

# While in the block, question is vote1's question,
# and question's rate is question's rate times question's count plus vote1's rate,
# divided by question's count plus 1,
# and question's count is question's count plus 1
{
    question = vote1.question
    question.rate = (question.rate * question.count + vote1.rate) / (question.count + 1)
    question.count = question.count + 1
}

assert(question1.rate, 4)
assert(question1.count, 1)

# vote1's rate is 5
vote1.rate = 5

assert(question1.rate, 4.5)

---

# Nucleoid runs an expression statement of class

# There is an Element type
class Element:
    pass

# alkalis is an empty list
alkalis = []

# element1 is an Element whose number is 3
element1 = Element()
element1.number = 3

# While in the block, number is any element's number,
# and if number is 3, then the element is added to alkalis
{
    number = $Element.number
    if number == 3:
        alkalis.push($Element)
}

assert(alkalis.pop(), element1)

---

# Nucleoid rejects a variable declaration without definition

try:
    # a is declared but not defined
    a
catch err:
    assert(err, SyntaxError("Missing definition"))

---

# Nucleoid creates a dependency based on the length of an identifier

# str1 is "ABC"
str1 = "ABC"

# i1 is str1's length plus 1
i1 = str1.length + 1

assert(i1, 4)

# str1 is "ABCD"
str1 = "ABCD"

assert(i1, 5)

# if str1's length is greater than 5, then i2 is i1
if str1.length > 5:
    i2 = i1

# str1 is "ABCDEF"
str1 = "ABCDEF"

assert(i2, 7)

---

# Nucleoid adds a created class to the class list

assert(Class.length, 0)

# There is a Student type
class Student:
    pass

assert(Class.length, 1)

# There is a User type
class User:
    pass

assert(Class.length, 2)

---

# Nucleoid updates a class definition

# There is a Message type
class Message:
    pass

# No message is read
$Message.read = False

# message1 is a Message
message1 = Message()

# There is a Message type,
# which has a payload as a string
class Message(payload: str):
    self.payload = payload

assert(message1.read, False)
assert(message1.payload, None)

# message2 is a Message whose payload is "MESSAGE"
message2 = Message("MESSAGE")

assert(message2.read, False)
assert(message2.payload, "MESSAGE")

---

# Nucleoid supports a string in an expression

assert('New String', "New String")
assert("New String", "New String")
assert(`New String`, "New String")

# a is 123
a = 123

assert(`New ${a} String`, "New 123 String")

---

# Nucleoid supports logical operators

# condition is false
condition = False

assert(condition or True, True)
assert(condition || True, True)

assert(not condition and True, True)
assert(!condition && True, True)

---

# Nucleoid supports standard built-in objects

# max is the maximum integer
max = Number.MAX_INTEGER

assert(max, 9007199254740991)

# now is the current time
now = Date.now()

assert(now > 0, True)

---

# Nucleoid supports creating standard built-in objects

# date is July 24, 2019
date = Date("2019-7-24")

assert(date.getYear(), 119)

---

# Nucleoid supports built-in objects

# date1 is the current date
date1 = Date()

# date2 is a date whose time is date1's time
date2 = Date(date1.getTime())

assert(date1.getTime() == date2.getTime(), True)

# date3 is the parsed date of "04 Dec 1995 00:12:00 GMT"
date3 = Date.parse("04 Dec 1995 00:12:00 GMT")

assert(date3, 818035920000)

try:
    # date4 is the wrong date
    date4 = Date.wrong()
catch err:
    assert(err, TypeError("Date.wrong is not a function"))

---

# Nucleoid calls a function with no return

# a is 1
a = 1

# copy assigns val to b
def copy(val):
    b = val

# Call copy with a
copy(a)

# return: None

---

# Nucleoid calls a function with a return value

# a is 1
a = 1

# copy assigns val to b and returns val
def copy(val):
    b = val
    return val

# Call copy with a
copy(a)

# return: 1

---

# Nucleoid supports a function in an expression

# list is a list of 1, 2 and 3
list = [1, 2, 3]

assert(list.find(function(element) { return element == 3 }), 3)
assert(list.find(element => { return element == 2 }), 2)
assert(list.find(element => element == 1), 1)
assert(list.find(element => (element == 1)), 1)

---

# Nucleoid supports a function with a parameter in an expression

# samples is a list of 38.2, 39.1, 38.8 and 39
samples = [38.2, 39.1, 38.8, 39]

# ratio is 2.1
ratio = 2.1

# element is 38.5
element = 38.5

# The element parameter shadows the outer element in each function
assert(samples.find(function(element) { result = element * ratio; return result == 81.48 }), 38.8)
assert(samples.find(element => { result = element * ratio; return result == 81.48 }), 38.8)
assert(samples.find(element => element == 38.8), 38.8)
assert(samples.find(element => (element == 38.8)), 38.8)

---

# Nucleoid creates a variable statement with JSON

# While in the block, payload is a local variable whose data is "TEST"
# and whose nested data is "NESTED_TEST"
{
    payload = { "data": "TEST", "nested": { "data": "NESTED_TEST" } }
    assert(payload.data, "TEST")
    assert(payload.nested.data, "NESTED_TEST")
}

# message is an object whose pid is 1200
message = { "pid": 1200 }

assert(message.pid, 1200)

# While in the block, scope is a local variable whose query is "test",
# and i is a local variable whose test is scope's query
{
    scope = { "query": "test" }
    i = { "test": scope.query }
    assert(i.test, "test")
}

---

# Nucleoid returns an inline JSON object

# While in the block, return an object whose number is 123,
# whose string is "ABC" and whose bool is true
{
    return { "number": 123, "string": "ABC", "bool": True }
}

# return: { "number": 123, "string": "ABC", "bool": True }

---

# Nucleoid returns an inline JSON array

# While in the block, return a list of an object whose number is 123,
# whose string is "ABC" and whose bool is true
{
    return [{ "number": 123, "string": "ABC", "bool": True }]
}

# return: [{ "number": 123, "string": "ABC", "bool": True }]

---

# Nucleoid returns an inline object

# While in the block, return an object whose number is 123,
# whose string is "ABC" and whose bool is true
{
    return { number: 123, string: "ABC", bool: True }
}

# return: { "number": 123, "string": "ABC", "bool": True }

---

# Nucleoid returns an inline array

# While in the block, return a list of an object whose number is 123,
# whose string is "ABC" and whose bool is true
{
    return [{ number: 123, string: "ABC", bool: True }]
}

# return: [{ "number": 123, "string": "ABC", "bool": True }]

---

# Nucleoid supports nested functions as a parameter in an expression

# name is "AbCDE"
name = "AbCDE"

# pointer is 0
pointer = 0

# if the character of name at pointer is not an uppercase letter,
# then throw "INVALID_FIRST_CHARACTER"
if not /[A-Z]/.test(name.charAt(pointer)):
    raise "INVALID_FIRST_CHARACTER"

try:
    # name is "bbCDE"
    name = "bbCDE"
catch err:
    assert(err, "INVALID_FIRST_CHARACTER")

# name is "CbCDE"
name = "CbCDE"

try:
    # pointer is 1
    pointer = 1
catch err:
    assert(err, "INVALID_FIRST_CHARACTER")

---

# Nucleoid supports a property of chained functions in an expression

# There is a User type
class User:
    pass

# There is a Registration type
class Registration:
    pass

# user1 is a User
user1 = User()

# registration1 is a Registration whose user is user1
registration1 = Registration()
registration1.user = user1

# registration2 is a Registration whose user is user1
registration2 = Registration()
registration2.user = user1

try:
    # if the registrations whose user is any user number more than one,
    # then throw "USER_ALREADY_REGISTERED"
    if Registration.filter(r => r.user == $User).length > 1:
        raise "USER_ALREADY_REGISTERED"
catch err:
    assert(err, "USER_ALREADY_REGISTERED")
 
---

# Nucleoid supports a property of chained functions in an expression

# There is a User type
class User:
    pass

# There is a Registration type
class Registration:
    pass

# user1 is a User
user1 = User()

# registration1 is a Registration whose user is user1
registration1 = Registration()
registration1.user = user1

# registration2 is a Registration whose user is user1
registration2 = Registration()
registration2.user = user1

try:
    # if the registrations whose user is any user number more than one,
    # then throw "USER_ALREADY_REGISTERED"
    if Registration.filter(r => r.user == $User).length > 1:
        raise "USER_ALREADY_REGISTERED"
catch err:
    assert(err, "USER_ALREADY_REGISTERED")

---

# Nucleoid supports an array with brackets

# states is a list of "NY", "GA", "CT" and "MI"
states = ["NY", "GA", "CT", "MI"]

# The value of states at index 2
states[2]

# return: "CT"

---

# Nucleoid creates a class assignment before initialization

# There is a Review type
class Review:
    pass

# Any review's rate is the review's sum divided by 10
$Review.rate = $Review.sum / 10

# review1 is a Review
review1 = Review()

assert(review1.rate, None)

# review1's sum is 42
review1.sum = 42

assert(review1.rate, 4.2)

---

# Nucleoid creates a class assignment after initialization

# There is a Shape type
class Shape:
    pass

# shape1 is a Shape whose edge is 3
shape1 = Shape()
shape1.edge = 3

# shape2 is a Shape whose edge is 3
shape2 = Shape()
shape2.edge = 3

# Any shape's angle is the shape's edge minus 2, times 180
$Shape.angle = ($Shape.edge - 2) * 180

assert(shape1.angle, 180)
assert(shape2.angle, 180)

# shape1's edge is 4
shape1.edge = 4

assert(shape1.angle, 360)
assert(shape2.angle, 180)

---

# Nucleoid updates a class assignment

# There is an Employee type
class Employee:
    pass

# employee1 is an Employee whose id is 1
employee1 = Employee()
employee1.id = 1

# Any employee's username is "E" plus the employee's id
$Employee.username = "E" + $Employee.id

assert(employee1.username, "E1")

# Any employee's username is "F" plus the employee's id
$Employee.username = "F" + $Employee.id

assert(employee1.username, "F1")

# employee1's id is 2
employee1.id = 2

assert(employee1.username, "F2")

---

# Nucleoid creates an if statement of class before initialization

# There is a Ticket type
class Ticket:
    pass

# Any ticket whose date is after January 1, 1993 is expired
if $Ticket.date > Date("1993-1-1"):
    $Ticket.status = "EXPIRED"

# ticket1 is a Ticket
ticket1 = Ticket()

assert(ticket1.status, None)

# ticket1's date is February 1, 1993
ticket1.date = Date("1993-2-1")

assert(ticket1.status, "EXPIRED")

# ticket2 is a Ticket
ticket2 = Ticket()

assert(ticket2.status, None)

---

# Nucleoid creates an if statement of class after initialization

# There is a Student type
class Student:
    pass

# student1 is a Student whose age is 2 and whose class is "Daycare"
student1 = Student()
student1.age = 2
student1.class = "Daycare"

# student2 is a Student whose age is 2 and whose class is "Daycare"
student2 = Student()
student2.age = 2
student2.class = "Daycare"

# Any student whose age is 3 is in Preschool
if $Student.age == 3:
    $Student.class = "Preschool"

assert(student1.class, "Daycare")
assert(student2.class, "Daycare")

# student1's age is 3
student1.age = 3

assert(student1.class, "Preschool")
assert(student2.class, "Daycare")

---

# Nucleoid updates an if block of class

# There is an Inventory type
class Inventory:
    pass

# inventory1 is an Inventory whose quantity is 0
inventory1 = Inventory()
inventory1.quantity = 0

# inventory2 is an Inventory whose quantity is 1000
inventory2 = Inventory()
inventory2.quantity = 1000

# Any inventory whose quantity is 0 needs replenishment
if $Inventory.quantity == 0:
    $Inventory.replenishment = True

assert(inventory1.replenishment, True)
assert(inventory2.replenishment, None)

# Any inventory whose quantity is 0 does not need replenishment
if $Inventory.quantity == 0:
    $Inventory.replenishment = False

assert(inventory1.replenishment, False)
assert(inventory2.replenishment, None)

---

# Nucleoid creates an else statement of class before initialization

# There is a Count type
class Count:
    pass

# If any count's max is greater than 1000, then the count's reset is urgent,
# else the count's reset is regular
if $Count.max > 1000:
    $Count.reset = urgent
else:
    $Count.reset = regular

# urgent is "URGENT"
urgent = "URGENT"

# regular is "REGULAR"
regular = "REGULAR"

# count1 is a Count
count1 = Count()

# count1's max is 850
count1.max = 850

assert(count1.reset, "REGULAR")

# regular is "R"
regular = "R"

assert(count1.reset, "R")

---

# Nucleoid creates an else statement of class after initialization

# There is a Concentration type
class Concentration:
    pass

# serialDilution is "(c1V1+c2V2)/(V1+V2)"
serialDilution = "(c1V1+c2V2)/(V1+V2)"

# directDilution is "c1/V1"
directDilution = "c1/V1"

# concentration1 is a Concentration whose substances is 2
concentration1 = Concentration()
concentration1.substances = 2

# If any concentration's substances is 1, then the concentration's formula is directDilution,
# else the concentration's formula is serialDilution
if $Concentration.substances == 1:
    $Concentration.formula = directDilution
else:
    $Concentration.formula = serialDilution

assert(concentration1.formula, "(c1V1+c2V2)/(V1+V2)")

# serialDilution is "(c1V1+c2V2+c3V3)/(V1+V2+V3)"
serialDilution = "(c1V1+c2V2+c3V3)/(V1+V2+V3)"

assert(concentration1.formula, "(c1V1+c2V2+c3V3)/(V1+V2+V3)")

---

# Nucleoid creates an else if statement of class before initialization

# There is a Storage type
class Storage:
    pass

# normal is "NORMAL", and low is "LOW"
normal = "NORMAL"; low = "LOW"

# If any storage's capacity is greater than 25, then the storage's status is normal,
# else the storage's status is low
if $Storage.capacity > 25:
    $Storage.status = normal
else:
    $Storage.status = low

# storage1 is a Storage
storage1 = Storage()

# storage1's capacity is 23
storage1.capacity = 23

assert(storage1.status, "LOW")

# low is "L"
low = "L"

assert(storage1.status, "L")

---

# Nucleoid creates an else if statement of class after initialization

# There is a Registration type
class Registration:
    pass

# yes is "YES", and no is "NO"
yes = "YES"; no = "NO"

# registration1 is a Registration whose available is 0
registration1 = Registration()
registration1.available = 0

# If any registration's available is greater than 0, then the registration's accepted is yes,
# else the registration's accepted is no
if $Registration.available > 0:
    $Registration.accepted = yes
else:
    $Registration.accepted = no

assert(registration1.accepted, "NO")

# yes is true, and no is false
yes = True; no = False

assert(registration1.accepted, False)

---

# Nucleoid creates multiple else if statement of class before initialization

# There is a Capacity type
class Capacity:
    pass

# If any capacity's spare divided by the capacity's available is greater than 0.5,
# then the capacity's total is the capacity's available plus the capacity's spare,
# else if any capacity's spare divided by the capacity's available is greater than 0.1,
# then the capacity's total is the capacity's available plus the capacity's spare times 2,
# else the capacity's total is the capacity's available plus the capacity's spare times 3
if $Capacity.spare / $Capacity.available > 0.5:
    $Capacity.total = $Capacity.available + $Capacity.spare
elif $Capacity.spare / $Capacity.available > 0.1:
    $Capacity.total = $Capacity.available + $Capacity.spare * 2
else:
    $Capacity.total = $Capacity.available + $Capacity.spare * 3

# capacity1 is a Capacity
capacity1 = Capacity()

# capacity1's available is 100
capacity1.available = 100

# capacity1's spare is 5
capacity1.spare = 5

assert(capacity1.total, 115)

# capacity1's spare is 1
capacity1.spare = 1

assert(capacity1.total, 103)

---

# Nucleoid creates multiple else if statement of class after initialization

# There is a Shape type
class Shape:
    pass

# shape1 is a Shape whose type is "RECTANGLE",
# whose x is 5 and whose y is 6
shape1 = Shape()
shape1.type = "RECTANGLE"
shape1.x = 5
shape1.y = 6

# If any shape's type is "SQUARE", then the shape's area is the shape's x squared,
# else if any shape's type is "TRIANGLE", then the shape's area is the shape's x times the shape's y divided by 2,
# else the shape's area is the shape's x times the shape's y
if $Shape.type == "SQUARE":
    $Shape.area = Math.pow($Shape.x, 2)
elif $Shape.type == "TRIANGLE":
    $Shape.area = $Shape.x * $Shape.y / 2
else:
    $Shape.area = $Shape.x * $Shape.y

assert(shape1.area, 30)

# shape1's x is 7
shape1.x = 7

assert(shape1.area, 42)

---

# Nucleoid runs a block statement of class before initialization

# There is a Stock type
class Stock:
    pass

# while in the block, change is a local variable that is any stock's before times 4 divided by 100,
# and the stock's after is the stock's before plus change
{
    change = $Stock.before * 4 / 100
    $Stock.after = $Stock.before + change
}

# stock1 is a Stock
stock1 = Stock()

assert(stock1.after, None)

# stock1's before is 57.25
stock1.before = 57.25

assert(stock1.after, 59.54)

# stock1's before is 59.5
stock1.before = 59.5

assert(stock1.after, 61.88)

---

# Nucleoid runs a block statement of class after initialization

# There is a Purchase type
class Purchase:
    pass

# purchase1 is a Purchase whose price is 99
purchase1 = Purchase()
purchase1.price = 99

# while in the block, retail is a local variable that is any purchase's price times 1.15,
# and the purchase's retail price is retail
{
    retail = $Purchase.price * 1.15
    $Purchase.retailPrice = retail
}

assert(purchase1.retailPrice, 113.85)

# purchase1's price is 199
purchase1.price = 199

assert(purchase1.retailPrice, 228.85)

---

# Nucleoid runs a nested block statement of class before initialization

# There is a Compound type
class Compound:
    pass

# while in the block, mol is a local variable that is 69.94 divided by any compound's substance,
# and in a nested block, the compound's sample is the floor of mol times the compound's mol
{
    mol = 69.94 / $Compound.substance
    {
        $Compound.sample = Math.floor(mol * $Compound.mol)
    }
}

# compound1 is a Compound
compound1 = Compound()

# compound1's substance is 55.85
compound1.substance = 55.85

# compound1's mol is 1000
compound1.mol = 1000

assert(compound1.sample, 1252)

---

# Nucleoid runs a nested block statement of class after initialization

# There is a Bug type
class Bug:
    pass

# bug1 is a Bug whose initial score is 1000
# and whose aging is 24
bug1 = Bug()
bug1.initialScore = 1000
bug1.aging = 24

# while in the block, score is a local variable that is any bug's aging times 10,
# and in a nested block, the bug's priority score is score plus the bug's initial score
{
    score = $Bug.aging * 10
    {
        $Bug.priorityScore = score + $Bug.initialScore
    }
}

assert(bug1.priorityScore, 1240)

---

# Nucleoid runs a nested if statement of class before initialization

# There is a Mortgage type
class Mortgage:
    pass

# rate1 is "EXCEPTIONAL"
rate1 = "EXCEPTIONAL"

# while in the block, interest is a local variable that is any mortgage's annual divided by 12,
# and if interest is less than 4, then the mortgage's rate is rate1
{
    interest = $Mortgage.annual / 12
    if interest < 4:
        $Mortgage.rate = rate1
}

# mortgage1 is a Mortgage
mortgage1 = Mortgage()

# mortgage1's annual is 46
mortgage1.annual = 46

assert(mortgage1.rate, "EXCEPTIONAL")

# rate1 is "E"
rate1 = "E"

assert(mortgage1.rate, "E")

---

# Nucleoid runs a nested if statement of class after initialization

# There is a Building type
class Building:
    pass

# buildingType1 is "SKYSCRAPER"
buildingType1 = "SKYSCRAPER"

# building1 is a Building whose floors is 20
building1 = Building()
building1.floors = 20

# while in the block, height is a local variable that is any building's floors times 14,
# and if height is greater than 330, then the building's type is buildingType1
{
    height = $Building.floors * 14
    if height > 330:
        $Building.type = buildingType1
}

assert(building1.type, None)

# building1's floors is 25
building1.floors = 25

assert(building1.type, "SKYSCRAPER")

# buildingType1 is "S"
buildingType1 = "S"

assert(building1.type, "S")

---

# Nucleoid creates a nested else statement of class before initialization

# There is an Account type
class Account:
    pass

# noAlert is "NO_ALERT"
noAlert = "NO_ALERT"

# lowAlert is "LOW_ALERT"
lowAlert = "LOW_ALERT"

# while in the block, balance is a local variable that is any account's balance,
# and if balance is greater than 1000, then the account's alert is noAlert,
# else the account's alert is lowAlert
{
    balance = $Account.balance
    if balance > 1000:
        $Account.alert = noAlert
    else:
        $Account.alert = lowAlert
}

# account1 is an Account
account1 = Account()

# account1's balance is 950
account1.balance = 950

assert(account1.alert, "LOW_ALERT")

# lowAlert is "L"
lowAlert = "L"

assert(account1.alert, "L")

---

# Nucleoid creates a nested else statement of class after initialization

# There is a Question type
class Question:
    pass

# high is "HIGH"
high = "HIGH"

# low is "LOW"
low = "LOW"

# question1 is a Question whose count is 1
question1 = Question()
question1.count = 1

# while in the block, score is a local variable that is any question's count times 10,
# and if score is greater than 100, then the question's type is high,
# else the question's type is low
{
    score = $Question.count * 10
    if score > 100:
        $Question.type = high
    else:
        $Question.type = low
}

assert(question1.type, "LOW")

# low is "L"
low = "L"

assert(question1.type, "L")

# question1's count is 11
question1.count = 11

assert(question1.type, "HIGH")

---

# Nucleoid creates a class assignment with multiple properties before declaration

# There is a Room type
class Room:
    pass

# Any room's level is the room's number divided by 10
$Room.level = $Room.number / 10

# There is a Guest type
class Guest:
    pass

# Any guest's room is a Room
$Guest.room = Room()

# guest1 is a Guest
guest1 = Guest()

# guest1's room's number is 30
guest1.room.number = 30

assert(guest1.room.level, 3)

# guest2 is a Guest
guest2 = Guest()

assert(guest2.room.number, 30)
assert(guest2.room.level, 3)

---

# Nucleoid creates a class assignment with multiple properties after declaration

# There is a Channel type
class Channel:
    pass

# There is a Frequency type
class Frequency:
    pass

# channel1 is a Channel
channel1 = Channel()

# Any channel's frequency is a Frequency
$Channel.frequency = Frequency()

# Any frequency's hertz is 1 divided by the frequency's period
$Frequency.hertz = 1 / $Frequency.period

assert(channel1.frequency.hertz, None)

# channel1's frequency's period is 0.0025
channel1.frequency.period = 0.0025

assert(channel1.frequency.hertz, 400)

# channel2 is a Channel
channel2 = Channel()

assert(channel2.frequency.period, 0.0025)
assert(channel2.frequency.hertz, 400)

---

# Nucleoid creates a class assignment as multiple properties as part of a declaration before initialization

# There is a Hospital type
class Hospital:
    pass

# There is a Clinic type
class Clinic:
    pass

# Any hospital's clinic is a Clinic
$Hospital.clinic = Clinic()

# Any hospital's patients is the hospital's clinic's beds times 746
$Hospital.patients = $Hospital.clinic.beds * 746

# hospital1 is a Hospital
hospital1 = Hospital()

assert(hospital1.patients, None)

# hospital1's clinic's beds is 2678
hospital1.clinic.beds = 2678

assert(hospital1.patients, 1997788)

# hospital1's clinic's beds is 3000
hospital1.clinic.beds = 3000

assert(hospital1.patients, 2238000)

---

# Nucleoid creates a class assignment as multiple properties as part of a declaration after initialization

# There is a Server type
class Server:
    pass

# There is an OS type
class OS:
    pass

# Any server's os is an OS
$Server.os = OS()

# server1 is a Server
server1 = Server()

# server1's os's version is 14
server1.os.version = 14

# Any server's build is the server's os's version plus ".526291"
$Server.build = $Server.os.version + ".526291"

assert(server1.build, "14.526291")

# server1's os's version is 15
server1.os.version = 15

assert(server1.build, "15.526291")

---

# Nucleoid creates a class assignment only if the instance is defined

# There is a Phone type
class Phone:
    pass

try:
    # any phone's line's wired is true
    $Phone.line.wired = True
catch err:
    assert(err, ReferenceError("Phone.line is not defined"))

---

# Nucleoid creates a for of statement

# There is a Question type,
# which has a rate as a number
class Question(rate: int):
    self.rate = rate

# question1 is a Question whose rate is 4
question1 = Question(4)

# question2 is a Question whose rate is 5
question2 = Question(5)

# There is a Summary type,
# which has a question as a Question
class Summary(question):
    self.question = question

# Any summary's rate is the value of the summary's question's rate
$Summary.rate = $Summary.question.rate.value

# For each question of Question, there is a Summary whose question is the question
for question of Question:
    Summary(question)

assert(Summary[0].rate, 4)
assert(Summary[1].rate, 5)

---

# Nucleoid creates a block of for statement without dependencies

# There is an Item type
class Item:
    pass

# item1 is an Item
item1 = Item()

# item2 is an Item
item2 = Item()

# VALUE is 10
VALUE = 10

# For each item of Item, while in the block, i is a local variable that is 10 times VALUE,
# and the item's score is i
for item of Item:
    i = 10 * VALUE
    item.score = i

# VALUE is 20
VALUE = 20

assert(item1.score, 100)
assert(item2.score, 100)

# For each item of Item, while in the block, i is a local variable that is 10 times VALUE,
# and the item's score is i
for item of Item:
    i = 10 * VALUE
    item.score = i

assert(item1.score, 200)
assert(item2.score, 200)

# item3 is an Item
item3 = Item()

assert(item3.score, None)

---

# Nucleoid loops through only defined objects in a for of statement

# array is an empty list
array = []

# There is an Item type
class Item:
    pass

# item1 is an Object, and item1 is added to array
item1 = Object()
array.push(item1)

# item2 is an object whose id is "item3", and item2 is added to array
item2 = { "id": "item3" }
array.push(item2)

# item4 is an Item, and item4 is added to array
item4 = Item()
array.push(item4)

# item5 is an object whose id is "item4", and item5 is added to array
item5 = { "id": "item4" }
array.push(item5)

# count is 0
count = 0

# items is an empty list
items = []

# For each item of array, count is count plus 1,
# and the item is added to items
for item of array:
    count = count + 1
    items.push(item)

assert(count, 1)
assert(items.length, 1)
assert(items[0], item4)

---

# Nucleoid supports an if statement in a for of statement

# There is a Question type
class Question:
    pass

# question1 is a Question
question1 = Question()

# question2 is a Question, which is archived
question2 = Question()
question2.archived = True

# question3 is a Question
question3 = Question()

# There is a Summary type,
# which has a question as a Question
class Summary(question):
    self.question = question

# Any summary's type is "DAILY"
$Summary.type = "DAILY"

# For each question of Question, if the question is not archived,
# then there is a Summary whose question is the question
for question of Question:
    if not question.archived:
        Summary(question)

assert(Summary.length, 2)
assert(Summary[0].question.id, "question1")
assert(Summary[1].question.id, "question3")
assert(Summary[0].type, "DAILY")
assert(Summary[1].type, "DAILY")

# Any summary's type is "WEEKLY"
$Summary.type = "WEEKLY"

assert(Summary[0].type, "WEEKLY")
assert(Summary[1].type, "WEEKLY")

---

# Nucleoid returns an integer in variable assignment

# test assigns 2 to a and returns the value of the assignment
def test(a):
    return a = 2

# b is 1
b = 1

# Call test with b
test(b)

# return: 2

---

# Nucleoid returns the reference of a function call

# a is an Object
a = Object()

# c is 1
c = 1

# test assigns a to b and returns the value of the assignment
def test(b):
    return b = a

assert(test(c), {})
assert(c, 1)

---

# Nucleoid returns a string value of a function call

# test assigns "abc" to a and returns the value of the assignment
def test(a):
    return a = "abc"

# b is 1
b = 1

# Call test with b
test(b)

# return: "abc"

---

# Nucleoid returns an object value of a function call

# test assigns an Object to a and returns the value of the assignment
def test(a):
    return a = Object()

# b is 1
b = 1

# Call test with b
test(b)

# return: {}

---

# Nucleoid runs a function with a variable

# test returns a plus 23
def test(a):
    return a + 23

# data is "UUID-1"
data = "UUID-1"

# Call test with data
test(data)

# return: "UUID-123"

---

# Nucleoid returns the first return statement in a block

# While in the block, return 123, and return "abc"
{
    return 123
    return "abc"
}

# return: 123

---

# Nucleoid returns the instance itself in instance creation

# There is a Test type,
# which has a prop as a number
class Test(prop: int):
    self.prop = prop

# There is a Test whose prop is 123
Test(123)

assert(Test[0].prop, 123)
assert(Test[0].id != None, True)

---

# Nucleoid returns the instance itself in instance creation

# There is a Test type,
# which has a prop as a number
class Test(prop: int):
    self.prop = prop

# There is a Test whose prop is 123
Test(123)

# return: { "id": ..., "prop": 123 }
```
