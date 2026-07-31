# Nucleoid Language Reference - Synthesized Use Cases 01

Project: Nucleoid is a Logic Programming Language for LLMs
Syntax: Minimum tokenized syntax with flexible grammar, which is a superset of Python, JavaScript/TypeScript, Kotlin, Go, Rust, Java, C# and C/C++.
Source: Synthesized from nucleoid.spec.md, covering behavior combinations not present in the base specification.

```

# Nucleoid updates a chain of dependent variables

# meters is 1000
meters = 1000

# kilometers is meters divided by 1000
kilometers = meters / 1000

# miles is kilometers times 0.621371
miles = kilometers * 0.621371

assert(kilometers, 1)
assert(miles, 0.621371)

# meters is 2000
meters = 2000

assert(kilometers, 2)
assert(miles, 1.242742)

---

# Nucleoid assigns a comparison result to a variable

# threshold is 100
threshold = 100

# reading is 120
reading = 120

# alarm is whether reading is greater than threshold
alarm = reading > threshold

assert(alarm, true)

# reading is 80
reading = 80

assert(alarm, false)

---

# Nucleoid updates a template literal when its dependency changes

# host is "localhost"
host = "localhost"

# port is 8080
port = 8080

# url is "http://" plus host plus ":" plus port
url = `http://${host}:${port}`

assert(url, "http://localhost:8080")

# port is 9090
port = 9090

assert(url, "http://localhost:9090")

---

# Nucleoid evaluates a logical expression with mixed operands

# online is true
online = true

# retries is 0
retries = 0

# healthy is online and whether retries is 0
healthy = online and retries == 0

assert(healthy, true)

# retries is 3
retries = 3

assert(healthy, false)

---

# Nucleoid respects parentheses in an expression

# a is 2
a = 2

# b is 3
b = 3

# c is 4
c = 4

assert(a + b * c, 14)
assert((a + b) * c, 20)

---

# Nucleoid creates a dependency on a string function

# name is "nucleoid"
name = "nucleoid"

# initial is the character of name at 0
initial = name.charAt(0)

# label is initial plus "-" plus name's length
label = initial + "-" + name.length

assert(label, "n-8")

# name is "logic"
name = "logic"

assert(label, "l-5")

---

# Nucleoid creates a dependency on a property of an inline object

# config is an object whose retries is 3
config = { "retries": 3 }

# budget is config's retries times 100
budget = config.retries * 100

assert(budget, 300)

# config is an object whose retries is 5
config = { "retries": 5 }

assert(budget, 500)

---

# Nucleoid rolls back dependent variables if an exception is thrown

# limit is 10
limit = 10

# double is limit times 2
double = limit * 2

assert(double, 20)

# if double is greater than 40, then throw "OVER_LIMIT"
if double > 40:
    throw "OVER_LIMIT"

try:
    # limit is 25
    limit = 25
catch error:
    assert(error, "OVER_LIMIT")

assert(limit, 10)
assert(double, 20)

---

# Nucleoid detects a circular dependency between properties

# There is a Box type
class Box:
    pass

# box1 is a Box whose width is 10
box1 = Box()
box1.width = 10

# box1's height is box1's width times 2
box1.height = box1.width * 2

assert(box1.height, 20)

try:
    # box1's width is box1's height divided by 2
    box1.width = box1.height / 2
catch error:
    assert(error, ReferenceError("Circular Dependency"))

---

# Nucleoid throws an error if a variable in a class-level expression is not defined

# There is an Order type
class Order:
    pass

try:
    # any order's total is the order's price times taxRate
    $Order.total = $Order.price * taxRate
catch error:
    assert(error, ReferenceError("taxRate is not defined"))

---

# Nucleoid rejects an unknown function of a standard built-in object

try:
    # result is the wrong math function
    result = Math.wrong(1)
catch error:
    assert(error, TypeError("Math.wrong is not a function"))

---

# Nucleoid throws a property as an error

# There is a Limit type
class Limit:
    pass

# limit1 is a Limit whose max is 5
limit1 = Limit()
limit1.max = 5

try:
    # if limit1's max is less than 10, then throw limit1's max
    if limit1.max < 10:
        throw limit1.max
catch error:
    assert(error, 5)

---

# Nucleoid rolls back an instance if a class-level condition throws

# There is a Payment type,
# which has an amount as a number
class Payment(amount: int):
    this.amount = amount

# If any payment's amount is less than 1, then throw 'INVALID_AMOUNT'
if $Payment.amount < 1:
    throw 'INVALID_AMOUNT'

# payment1 is a Payment whose amount is 100
payment1 = Payment(100)

assert(Payment.length, 1)

try:
    # payment2 is a Payment whose amount is 0
    payment2 = Payment(0)
catch error:
    assert(error, "INVALID_AMOUNT")

assert(Payment.length, 1)

try:
    # payment2
    payment2
catch error:
    assert(error, ReferenceError("payment2 is not defined"))

---

# Nucleoid rolls back related properties in the same transaction

# There is a Battery type
class Battery:
    pass

# Any battery's percentage is the battery's level times 100 divided by the battery's capacity
$Battery.percentage = $Battery.level * 100 / $Battery.capacity

# If any battery's percentage is greater than 100, then throw "INVALID_LEVEL"
if $Battery.percentage > 100:
    throw "INVALID_LEVEL"

# battery1 is a Battery whose capacity is 50 and whose level is 25
battery1 = Battery()
battery1.capacity = 50
battery1.level = 25

assert(battery1.percentage, 50)

try:
    # battery1's level is 75
    battery1.level = 75
catch error:
    assert(error, "INVALID_LEVEL")

assert(battery1.level, 25)
assert(battery1.percentage, 50)

---

# Nucleoid propagates null when a property is deleted

# There is a Report type
class Report:
    pass

# report1 is a Report whose year is 2019 and whose month is 4
report1 = Report()
report1.year = 2019
report1.month = 4

# report1's period is report1's year plus "-" plus report1's month
report1.period = report1.year + "-" + report1.month

# report1's label is "P" plus report1's year
report1.label = "P" + report1.year

assert(report1.period, "2019-4")
assert(report1.label, "P2019")

# report1's month is deleted
delete report1.month

assert(report1.period, null)
assert(report1.label, "P2019")

---

# Nucleoid returns a boolean when deleting a property

# There is a Setting type
class Setting:
    pass

# setting1 is a Setting whose theme is "DARK"
setting1 = Setting()
setting1.theme = "DARK"

# Deleting setting1's theme returns true
assert(delete setting1.theme, true)

# Deleting setting1's theme again returns false
assert(delete setting1.theme, false)

assert(setting1.theme, null)

---

# Nucleoid reuses a variable name after an instance is deleted

# There is a Session type
class Session:
    pass

# session1 is a Session
session1 = Session()

assert(Session.length, 1)

# session1 is deleted
delete session1

assert(Session.length, 0)

# session1 is a Session
session1 = Session()

assert(Session.length, 1)
assert(Session["session1"], { "id": "session1" })

---

# Nucleoid applies a class-level property to an instance created with a constructor

# There is a Book type,
# which has a title as a string
# and a pages as a number
class Book(title: str, pages: int):
    this.title = title
    this.pages = pages

# Any book's summary is the book's title plus " (" plus the book's pages plus " pages)"
$Book.summary = $Book.title + " (" + $Book.pages + " pages)"

# book1 is a Book whose title is "Logic" and whose pages is 320
book1 = Book("Logic", 320)

assert(book1.summary, "Logic (320 pages)")

# book1's pages is 400
book1.pages = 400

assert(book1.summary, "Logic (400 pages)")

---

# Nucleoid creates a subclass with additional attributes

# There is a Vehicle type,
# which has a make as a string
class Vehicle(make: str):
    this.make = make

# There is a Truck type,
# which is a subtype of Vehicle
# and has a payload as a number
class Truck: Vehicle
    def init(self, make, payload):
        super(make)
        this.payload = payload

# truck1 is a Truck whose make is "Volvo" and whose payload is 12000
truck1 = Truck("Volvo", 12000)

assert(truck1, { "id": "truck1", "make": "Volvo", "payload": 12000 })
assert(Truck.length, 1)

---

# Nucleoid finds an instance by a class-level derived property

# There is an Employee type,
# which has a first as a string
# and a last as a string
class Employee(first: str, last: str):
    this.first = first
    this.last = last

# Any employee's full is the employee's first plus " " plus the employee's last
$Employee.full = $Employee.first + " " + $Employee.last

# employee1 is an Employee whose first is "Ada" and whose last is "Lovelace"
employee1 = Employee("Ada", "Lovelace")

# employee2 is an Employee whose first is "Alan" and whose last is "Turing"
employee2 = Employee("Alan", "Turing")

# target is "Alan Turing"
target = "Alan Turing"

# match is the Employee whose full is target
match = Employee.find(e => e.full == target)

assert(match, employee2)

# target is "Ada Lovelace"
target = "Ada Lovelace"

assert(match, employee1)

---

# Nucleoid chains a property on the result of a function call

# There is a Sensor type,
# which has a code as a string
class Sensor(code: str):
    this.code = code

# sensor1 is a Sensor whose code is "S1"
sensor1 = Sensor("S1")

# sensor2 is a Sensor whose code is "S2"
sensor2 = Sensor("S2")

# sensor1's reading is 21
sensor1.reading = 21

# sensor2's reading is 34
sensor2.reading = 34

# code is "S2"
code = "S2"

# reading is the reading of the Sensor whose code is code
reading = Sensor.find(s => s.code == code).reading

assert(reading, 34)

# code is "S1"
code = "S1"

assert(reading, 21)

---

# Nucleoid chains class-level properties

# There is a Rectangle type
class Rectangle:
    pass

# Any rectangle's area is the rectangle's width times the rectangle's height
$Rectangle.area = $Rectangle.width * $Rectangle.height

# Any rectangle's ratio is the rectangle's area divided by the rectangle's width
$Rectangle.ratio = $Rectangle.area / $Rectangle.width

# rectangle1 is a Rectangle whose width is 4 and whose height is 5
rectangle1 = Rectangle()
rectangle1.width = 4
rectangle1.height = 5

assert(rectangle1.area, 20)
assert(rectangle1.ratio, 5)

# rectangle1's height is 10
rectangle1.height = 10

assert(rectangle1.area, 40)
assert(rectangle1.ratio, 10)

---

# Nucleoid updates every instance when a class-level dependency changes

# There is an Invoice type
class Invoice:
    pass

# taxRate is 8
taxRate = 8

# Any invoice's total is the invoice's amount plus the invoice's amount times taxRate divided by 100
$Invoice.total = $Invoice.amount + $Invoice.amount * taxRate / 100

# invoice1 is an Invoice whose amount is 100
invoice1 = Invoice()
invoice1.amount = 100

# invoice2 is an Invoice whose amount is 200
invoice2 = Invoice()
invoice2.amount = 200

assert(invoice1.total, 108)
assert(invoice2.total, 216)

# taxRate is 10
taxRate = 10

assert(invoice1.total, 110)
assert(invoice2.total, 220)

---

# Nucleoid overrides a class-level property on a single instance

# There is a Node type
class Node:
    pass

# Any node's status is "IDLE"
$Node.status = "IDLE"

# node1 is a Node
node1 = Node()

# node2 is a Node
node2 = Node()

assert(node1.status, "IDLE")
assert(node2.status, "IDLE")

# node1's status is "ACTIVE"
node1.status = "ACTIVE"

assert(node1.status, "ACTIVE")
assert(node2.status, "IDLE")

---

# Nucleoid applies a class-level else if statement to every instance

# There is a Grade type
class Grade:
    pass

# grade1 is a Grade whose score is 95
grade1 = Grade()
grade1.score = 95

# grade2 is a Grade whose score is 85
grade2 = Grade()
grade2.score = 85

# grade3 is a Grade whose score is 60
grade3 = Grade()
grade3.score = 60

# If any grade's score is greater than 89, then the grade's letter is "A",
# else if any grade's score is greater than 79, then the grade's letter is "B",
# else the grade's letter is "C"
if $Grade.score > 89:
    $Grade.letter = "A"
elif $Grade.score > 79:
    $Grade.letter = "B"
else:
    $Grade.letter = "C"

assert(grade1.letter, "A")
assert(grade2.letter, "B")
assert(grade3.letter, "C")

# grade3's score is 90
grade3.score = 90

assert(grade3.letter, "A")

---

# Nucleoid validates a class-level property with a regular expression

# There is an Account type
class Account:
    pass

# If any account's code does not match /^[A-Z]{2}-[0-9]{4}$/, then throw 'INVALID_CODE'
if not /^[A-Z]{2}-[0-9]{4}$/.test($Account.code):
    throw 'INVALID_CODE'

# account1 is an Account
account1 = Account()

assert(account1.code, null)

# account1's code is "NY-1234"
account1.code = "NY-1234"

assert(account1.code, "NY-1234")

try:
    # account1's code is "nyc-1"
    account1.code = "nyc-1"
catch error:
    assert(error, "INVALID_CODE")

assert(account1.code, "NY-1234")

---

# Nucleoid uses a class list in a class-level condition

# There is a Room type
class Room:
    pass

# There is a Booking type
class Booking:
    pass

# room1 is a Room
room1 = Room()

# Any room's booked is whether the bookings whose room is the room number more than zero
$Room.booked = Booking.filter(b => b.room == $Room).length > 0

assert(room1.booked, false)

# booking1 is a Booking whose room is room1
booking1 = Booking()
booking1.room = room1

assert(room1.booked, true)

---

# Nucleoid updates a property through a reference to another instance

# There is a Currency type
class Currency:
    pass

# There is a Price type
class Price:
    pass

# usd is a Currency whose rate is 1
usd = Currency()
usd.rate = 1

# price1 is a Price whose amount is 50 and whose currency is usd
price1 = Price()
price1.amount = 50
price1.currency = usd

# price1's converted is price1's amount times price1's currency's rate
price1.converted = price1.amount * price1.currency.rate

assert(price1.converted, 50)

# usd's rate is 1.5
usd.rate = 1.5

assert(price1.converted, 75)

---

# Nucleoid creates an if statement with multiple properties in a condition

# There is a Flight type
class Flight:
    pass

# flight1 is a Flight whose seats is 150 and whose booked is 100
flight1 = Flight()
flight1.seats = 150
flight1.booked = 100

# if flight1's booked is flight1's seats, then flight1's status is "FULL"
if flight1.booked == flight1.seats:
    flight1.status = "FULL"

assert(flight1.status, null)

# flight1's booked is 150
flight1.booked = 150

assert(flight1.status, "FULL")

---

# Nucleoid runs a nested if statement inside an else statement

# temperature is 30
temperature = 30

# humidity is 80
humidity = 80

# if temperature is greater than 35, then advice is "HEAT",
# else if humidity is greater than 70, then advice is "HUMID",
# else advice is "NORMAL"
if temperature > 35:
    advice = "HEAT"
elif humidity > 70:
    advice = "HUMID"
else:
    advice = "NORMAL"

assert(advice, "HUMID")

# humidity is 40
humidity = 40

assert(advice, "NORMAL")

# temperature is 40
temperature = 40

assert(advice, "HEAT")

---

# Nucleoid uses one local variable for multiple assignments in a block

# side is 3
side = 3

# while in the block, area is a local variable that is side squared,
# and surface is area times 6, and volume is area times side
{
    area = Math.pow(side, 2)
    surface = area * 6
    volume = area * side
}

assert(surface, 54)
assert(volume, 27)

# side is 4
side = 4

assert(surface, 96)
assert(volume, 64)

---

# Nucleoid creates a local list in a block

# factor is 2
factor = 2

# while in the block, numbers is a local list of 1, 2 and 3,
# and total is the numbers filtered to those greater than factor
{
    numbers = [1, 2, 3]
    total = numbers.filter(n => n > factor)
}

assert(total.length, 1)
assert(total[0], 3)

# factor is 0
factor = 0

assert(total.length, 3)

---

# Nucleoid returns a local variable from a block

# rate is 7
rate = 7

# while in the block, total is a local variable that is rate times 3,
# and return total
{
    total = rate * 3
    return total
}

# return: 21

---

# Nucleoid returns a property of a local variable from a block

# There is a Job type
class Job:
    pass

# job1 is a Job whose state is "DONE"
job1 = Job()
job1.state = "DONE"

# While in the block, job is a local variable that is the Job whose state is "DONE",
# and if there is no job, then throw "NO_JOB",
# and return the job's id
{
    job = Job.find(j => j.state == "DONE")
    if not job:
        throw "NO_JOB"
    return job.id
}

# return: "job1"

---

# Nucleoid returns an inline object with values from the state

# code is 200
code = 200

# message is "OK"
message = "OK"

# While in the block, return an object whose code is code and whose message is message
{
    return { "code": code, "message": message }
}

# return: { "code": 200, "message": "OK" }

---

# Nucleoid calls a function in a class-level assignment

# discount returns price minus price times rate divided by 100
def discount(price, rate):
    return price - price * rate / 100

# There is a Product type
class Product:
    pass

# percent is 10
percent = 10

# Any product's sale is the result of the discount function call with the product's price and percent
$Product.sale = discount($Product.price, percent)

# product1 is a Product whose price is 200
product1 = Product()
product1.price = 200

assert(product1.sale, 180)

# percent is 25
percent = 25

assert(product1.sale, 150)

---

# Nucleoid calls a function inside another function call

# double returns number times 2
def double(number):
    return number * 2

# increment returns number plus 1
def increment(number):
    return number + 1

# seed is 5
seed = 5

# total is the result of the increment function call with the result of the double function call with seed
total = increment(double(seed))

assert(total, 11)

# seed is 10
seed = 10

assert(total, 21)

---

# Nucleoid uses a standard built-in function in a reactive assignment

# total is 47
total = 47

# size is 10
size = 10

# pages is the floor of total divided by size, plus 1
pages = Math.floor(total / size) + 1

assert(pages, 5)

# total is 100
total = 100

assert(pages, 11)

---

# Nucleoid compares standard built-in date objects

# start is January 1, 2020
start = Date("2020-1-1")

# end is January 2, 2020
end = Date("2020-1-2")

# duration is end's time minus start's time
duration = end.getTime() - start.getTime()

assert(duration, 86400000)
assert(end > start, true)

---

# Nucleoid uses value property in a class-level condition

# There is a Version type
class Version:
    pass

# baseline is 3
baseline = 3

# If any version's number is less than baseline's value, then the version's supported is false
if $Version.number < baseline.value:
    $Version.supported = false

# version1 is a Version whose number is 2
version1 = Version()
version1.number = 2

assert(version1.supported, false)

# version2 is a Version whose number is 5
version2 = Version()
version2.number = 5

assert(version2.supported, null)

# baseline is 10
baseline = 10

assert(version2.supported, null)

---

# Nucleoid uses value property to avoid a circular dependency

# a is 5
a = 5

# b is a's value times 2
b = a.value * 2

assert(b, 10)

# a is b plus 1
a = b + 1

assert(a, 11)
assert(b, 10)

---

# Nucleoid skips values that are not instances in a for of statement

# There is a Task type
class Task:
    pass

# entries is an empty list
entries = []

# "PENDING" is added to entries
entries.push("PENDING")

# an object whose id is "task9" is added to entries
entries.push({ "id": "task9" })

# task1 is a Task, and task1 is added to entries
task1 = Task()
entries.push(task1)

# visited is an empty list
visited = []

# For each entry of entries, the entry is added to visited
for entry of entries:
    visited.push(entry)

assert(visited.length, 1)
assert(visited[0], task1)

---

# Nucleoid supports an else if statement in a for of statement

# There is a Package type,
# which has a weight as a number
class Package(weight: int):
    this.weight = weight

# package1 is a Package whose weight is 30
package1 = Package(30)

# package2 is a Package whose weight is 10
package2 = Package(10)

# package3 is a Package whose weight is 1
package3 = Package(1)

# For each package of Package, if the package's weight is greater than 20,
# then the package's tier is "HEAVY",
# else if the package's weight is greater than 5, then the package's tier is "MEDIUM",
# else the package's tier is "LIGHT"
for package of Package:
    if package.weight > 20:
        package.tier = "HEAVY"
    elif package.weight > 5:
        package.tier = "MEDIUM"
    else:
        package.tier = "LIGHT"

assert(package1.tier, "HEAVY")
assert(package2.tier, "MEDIUM")
assert(package3.tier, "LIGHT")

---

# Nucleoid assigns unique ids to instances created in a for of statement

# There is a Seat type
class Seat:
    pass

# There is a Ticket type,
# which has a seat as a Seat
class Ticket(seat):
    this.seat = seat

# seat1 is a Seat
seat1 = Seat()

# seat2 is a Seat
seat2 = Seat()

# For each seat of Seat, there is a Ticket whose seat is the seat
for seat of Seat:
    Ticket(seat)

assert(Ticket.length, 2)
assert(Ticket[0].seat.id, "seat1")
assert(Ticket[1].seat.id, "seat2")
assert(Ticket[0].id != Ticket[1].id, true)

---

# Nucleoid applies a class-level property declared after a for of statement

# There is a Course type
class Course:
    pass

# course1 is a Course whose credits is 3
course1 = Course()
course1.credits = 3

# course2 is a Course whose credits is 4
course2 = Course()
course2.credits = 4

# There is an Enrollment type,
# which has a course as a Course
class Enrollment(course):
    this.course = course

# For each course of Course, there is an Enrollment whose course is the course
for course of Course:
    Enrollment(course)

# Any enrollment's hours is the enrollment's course's credits times 15
$Enrollment.hours = $Enrollment.course.credits * 15

assert(Enrollment.length, 2)
assert(Enrollment[0].hours, 45)
assert(Enrollment[1].hours, 60)
```
