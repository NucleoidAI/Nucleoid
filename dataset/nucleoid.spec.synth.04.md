# Nucleoid Language Reference - Synthesized Use Cases 04

```

# Nucleoid declares a constructor with a typed attribute and an init function

# There is a Ticket type,
# which has a seat as a string
class Ticket:
    seat: str

    def init(seat: str):
        this.seat = seat

# ticket1 is a Ticket whose seat is "12A"
ticket1 = Ticket("12A")

assert(ticket1, { "id": "ticket1", "seat": "12A" })

---

# Nucleoid leaves a missing constructor argument as null

# There is a Label type,
# which has a text as a string
class Label(text: str):
    this.text = text

# label1 is a Label whose text is "TOP"
label1 = Label("TOP")

assert(label1, { "id": "label1", "text": "TOP" })

# label2 is a Label with no text
label2 = Label()

assert(label2, { "id": "label2", "text": null })

---

# Nucleoid rejects creating an instance of a type that does not exist

try:
    # widget1 is a Widget
    widget1 = Widget()
catch error:
    assert(error, ReferenceError("Widget is not defined"))

# There is a Widget type
class Widget:
    pass

# widget1 is a Widget
widget1 = Widget()

assert(Widget.length, 1)

---

# Nucleoid assigns null when a property in an expression is undefined

# There is a Contact type
class Contact:
    pass

# contact1 is a Contact
contact1 = Contact()

# contact1's surname is "Reed"
contact1.surname = "Reed"

# contact1's display is contact1's forename plus " " plus contact1's surname
contact1.display = contact1.forename + " " + contact1.surname

assert(contact1.display, null)

# contact1's forename is "Ada"
contact1.forename = "Ada"

assert(contact1.display, "Ada Reed")

---

# Nucleoid keeps a property null while a dependency is null

# There is a Job type
class Job:
    pass

# job1 is a Job whose schedule is "0 9 * * *" and whose command is null
job1 = Job()
job1.schedule = "0 9 * * *"
job1.command = null

# job1's entry is job1's schedule plus " " plus job1's command
job1.entry = job1.schedule + " " + job1.command

assert(job1.entry, null)

---

# Nucleoid assigns null when an expression reads through a null pointer

# There is a Batch type
class Batch:
    pass

# batch1 is a Batch
batch1 = Batch()

# grade is batch1's sample's weight
grade = batch1.sample.weight

assert(grade, null)

---

# Nucleoid creates a dependency on the length of a string

# token is "ABC"
token = "ABC"

# size is token's length plus 1
size = token.length + 1

assert(size, 4)

# token is "ABCD"
token = "ABCD"

assert(size, 5)

# if token's length is greater than 5, then flag is size
if token.length > 5:
    flag = size

# token is "ABCDEF"
token = "ABCDEF"

assert(flag, 7)

---

# Nucleoid lowercases a string as a dependency

# name1 is "ADA"
name1 = "ADA"

# name2 is name1 lowercased plus "!"
name2 = name1.lower() + "!"

# name3 is name2 concatenated with name1
name3 = name2 + name1

assert(name2, "ada!")
assert(name3, "ada!ADA")

# name1 is "BOB"
name1 = "BOB"

assert(name2, "bob!")
assert(name3, "bob!BOB")

---

# Nucleoid adds a duration to a class-level date property

# There is a Licence type
class Licence:
    pass

# Any licence's expiry is the licence's issued plus 2592000000
$Licence.expiry = $Licence.issued + 2592000000

# licence1 is a Licence whose issued is the time of January 1, 2020
licence1 = Licence()
licence1.issued = Date("2020-1-1").getTime()

assert(licence1.expiry - licence1.issued, 2592000000)

---

# Nucleoid builds a class-level property from a date string

# There is an Event type
class Event:
    pass

# Any event's label is "on " plus the event's day as a date string
$Event.label = "on " + $Event.day.toDateString()

# event1 is an Event whose day is March 5, 2021
event1 = Event()
event1.day = Date("2021-3-5")

assert(event1.day.toDateString(), "Fri Mar 05 2021")
assert(event1.label, "on Fri Mar 05 2021")

---

# Nucleoid creates nested instances in a block and assigns them to a property

# There is an Invoice type
class Invoice:
    pass

# There is a Line type
class Line:
    pass

# There is an Item type
class Item:
    pass

# Any item's label is "SKU-" plus the item's code
$Item.label = "SKU-" + $Item.code

# invoice1 is an Invoice
invoice1 = Invoice()

# line is a Line whose item is an Item whose code is 4471,
# and invoice1's line is line
{
    line = Line()
    line.item = Item()
    line.item.code = 4471
    invoice1.line = line
}

assert(invoice1.line.item.code, 4471)
assert(invoice1.line.item.label, "SKU-4471")

---

# Nucleoid creates an object in a block and assigns it to a class-level property before instantiation

# There is a Subscriber type
class Subscriber:
    pass

# terms is an Object whose signed is June 1, 2021,
# and any subscriber's terms is terms
{
    terms = Object()
    terms.signed = Date("2021-6-1")
    $Subscriber.terms = terms
}

# subscriber1 is a Subscriber
subscriber1 = Subscriber()

assert(subscriber1.terms.signed.toDateString(), "Tue Jun 01 2021")
assert(subscriber1.terms.expiry, null)

---

# Nucleoid creates an object in a block and assigns it to a class-level property after instantiation

# There is a Depot type
class Depot:
    pass

# depot1 is a Depot
depot1 = Depot()

# region is an Object whose zone is "EU-WEST",
# and any depot's region is region
{
    region = Object()
    region.zone = "EU-WEST"
    $Depot.region = region
}

assert(depot1.region.zone, "EU-WEST")
assert(depot1.region.name, null)

---

# Nucleoid creates a nested object in a block and assigns it to a class-level property

# There is a Wallet type
class Wallet:
    pass

# There is a Currency type
class Currency:
    pass

# Any currency's symbol is "#" plus the currency's code
$Currency.symbol = "#" + $Currency.code

# holding is an Object whose unit is an Object whose code is "EUR",
# and any wallet's holding is holding
{
    holding = Object()
    holding.unit = Object()
    holding.unit.code = "EUR"
    $Wallet.holding = holding
}

# wallet1 is a Wallet
wallet1 = Wallet()

assert(wallet1.holding.unit.code, "EUR")
assert(wallet1.holding.unit.symbol, null)

---

# Nucleoid shares a class-level instance property with every instance

# There is a Desk type
class Desk:
    pass

# Any desk's level is the desk's number divided by 10
$Desk.level = $Desk.number / 10

# There is a Worker type
class Worker:
    pass

# Any worker's desk is a Desk
$Worker.desk = Desk()

# worker1 is a Worker
worker1 = Worker()

# worker1's desk's number is 40
worker1.desk.number = 40

assert(worker1.desk.level, 4)

# worker2 is a Worker
worker2 = Worker()

assert(worker2.desk.number, 40)
assert(worker2.desk.level, 4)

---

# Nucleoid reads a class-level property through an instance created by a class-level rule

# There is a Campus type
class Campus:
    pass

# There is a Hall type
class Hall:
    pass

# Any campus's hall is a Hall
$Campus.hall = Hall()

# Any campus's seats is the campus's hall's rows times 24
$Campus.seats = $Campus.hall.rows * 24

# campus1 is a Campus
campus1 = Campus()

assert(campus1.seats, null)

# campus1's hall's rows is 12
campus1.hall.rows = 12

assert(campus1.seats, 288)

---

# Nucleoid runs a local variable as an object before declaration

# There is a Pump type
class Pump:
    pass

# There is a Tank type
class Tank:
    pass

# pump1 is a Pump whose rate is 25
pump1 = Pump()
pump1.rate = 25

# tank1 is a Tank whose volume is 500
tank1 = Tank()
tank1.volume = 500

# while in the block, tank is any pump's tank,
# and any pump's minutes is tank's volume divided by the pump's rate
{
    tank = $Pump.tank
    $Pump.minutes = tank.volume / $Pump.rate
}

# pump1's tank is tank1
pump1.tank = tank1

assert(pump1.minutes, 20)

---

# Nucleoid runs a local variable as an object after declaration

# There is a Loan type
class Loan:
    pass

# There is a Rate type
class Rate:
    pass

# loan1 is a Loan whose principal is 200000
loan1 = Loan()
loan1.principal = 200000

# rate1 is a Rate whose percent is 0.05
rate1 = Rate()
rate1.percent = 0.05

# loan1's rate is rate1
loan1.rate = rate1

# While in the block, rate is any loan's rate,
# and any loan's interest is the loan's principal times rate's percent
{
    rate = $Loan.rate
    $Loan.interest = $Loan.principal * rate.percent
}

assert(loan1.interest, 10000)

---

# Nucleoid assigns a property on a local variable after initialization

# There is a Shelf type
class Shelf:
    pass

# There is a Slot type
class Slot:
    pass

# shelf1 is a Shelf whose depth is 60
shelf1 = Shelf()
shelf1.depth = 60

# slot1 is a Slot whose count is 3
slot1 = Slot()
slot1.count = 3

# shelf1's slot is slot1
shelf1.slot = slot1

# While in the block, slot is any shelf's slot,
# and slot's space is the shelf's depth times slot's count
{
    slot = $Shelf.slot
    slot.space = $Shelf.depth * slot.count
}

assert(slot1.space, 180)

---

# Nucleoid holds the result of a function in a local variable

# tasks is an empty list
tasks = []

# code is 1
code = 1

# There is a Task type
class Task:
    pass

# task1 is a Task whose code is 1,
# and task1 is added to tasks
task1 = Task()
task1.code = 1
tasks.push(task1)

# task2 is a Task whose code is 2,
# and task2 is added to tasks
task2 = Task()
task2.code = 2
tasks.push(task2)

# While in the block, task is the task in tasks whose code is code,
# and task is flagged
{
    task = tasks.find(t => t.code == code)
    task.flagged = true
}

assert(task1.flagged, true)
assert(task2.flagged, null)

# code is 2
code = 2

assert(task2.flagged, true)

---

# Nucleoid creates local variables inside an if block

# base is 5
base = 5

# extra is 10
extra = 10

# if base is greater than 9, then while in the block, total is base plus extra,
# and scaled is total times 10
if base > 9:
    total = base + extra
    scaled = total * 10

# base is 10
base = 10

assert(scaled, 200)

# base is 15
base = 15

assert(scaled, 250)

# extra is 20
extra = 20

assert(scaled, 350)

---

# Nucleoid runs a nested if statement inside a block

# rate is 12
rate = 12

# hours is 40
hours = 40

# threshold is 400
threshold = 400

# bonus is true
bonus = true

# while in the block, gross is a local variable that is rate times hours,
# and if gross is greater than threshold, then eligible is bonus
{
    gross = rate * hours
    if gross > threshold:
        eligible = bonus
}

assert(eligible, true)

# bonus is false
bonus = false

assert(eligible, false)

---

# Nucleoid runs a nested else statement inside a block

# count is 12
count = 12

# size is 4
size = 4

# kind is "GRID"
kind = "GRID"

# layout is null
layout = null

# fallback is 0
fallback = 0

# while in the block, cells is a local variable that is count times size,
# and if kind is "GRID", then layout is cells divided by 2,
# else layout is fallback
{
    cells = count * size
    if kind == "GRID":
        layout = cells / 2
    else:
        layout = fallback
}

# kind is "LIST"
kind = "LIST"

# fallback is 1
fallback = 1

assert(layout, 1)

---

# Nucleoid runs a class-level block statement before instantiation

# There is a Quote type
class Quote:
    pass

# while in the block, markup is a local variable that is any quote's cost times 20 divided by 100,
# and the quote's price is the quote's cost plus markup
{
    markup = $Quote.cost * 20 / 100
    $Quote.price = $Quote.cost + markup
}

# quote1 is a Quote
quote1 = Quote()

assert(quote1.price, null)

# quote1's cost is 50
quote1.cost = 50

assert(quote1.price, 60)

# quote1's cost is 80
quote1.cost = 80

assert(quote1.price, 96)

---

# Nucleoid runs a class-level block statement after instantiation

# There is a Trip type
class Trip:
    pass

# trip1 is a Trip whose distance is 120
trip1 = Trip()
trip1.distance = 120

# while in the block, fuel is a local variable that is any trip's distance divided by 15,
# and the trip's litres is fuel
{
    fuel = $Trip.distance / 15
    $Trip.litres = fuel
}

assert(trip1.litres, 8)

# trip1's distance is 150
trip1.distance = 150

assert(trip1.litres, 10)

---

# Nucleoid runs a nested class-level block statement

# There is a Mixture type
class Mixture:
    pass

# while in the block, unit is a local variable that is 100 divided by any mixture's parts,
# and in a nested block, the mixture's share is the floor of unit times the mixture's weight
{
    unit = 100 / $Mixture.parts
    {
        $Mixture.share = Math.floor(unit * $Mixture.weight)
    }
}

# mixture1 is a Mixture
mixture1 = Mixture()

# mixture1's parts is 3
mixture1.parts = 3

# mixture1's weight is 7
mixture1.weight = 7

assert(mixture1.share, 233)

---

# Nucleoid runs a nested if statement in a class-level block

# There is a Policy type
class Policy:
    pass

# band1 is "PREFERRED"
band1 = "PREFERRED"

# while in the block, monthly is a local variable that is any policy's annual divided by 12,
# and if monthly is less than 50, then the policy's band is band1
{
    monthly = $Policy.annual / 12
    if monthly < 50:
        $Policy.band = band1
}

# policy1 is a Policy
policy1 = Policy()

# policy1's annual is 480
policy1.annual = 480

assert(policy1.band, "PREFERRED")

# band1 is "P"
band1 = "P"

assert(policy1.band, "P")

---

# Nucleoid runs a nested else statement in a class-level block

# There is a Meter type
class Meter:
    pass

# steady is "STEADY"
steady = "STEADY"

# surge is "SURGE"
surge = "SURGE"

# while in the block, load is a local variable that is any meter's watts,
# and if load is greater than 2000, then the meter's state is surge,
# else the meter's state is steady
{
    load = $Meter.watts
    if load > 2000:
        $Meter.state = surge
    else:
        $Meter.state = steady
}

# meter1 is a Meter
meter1 = Meter()

# meter1's watts is 900
meter1.watts = 900

assert(meter1.state, "STEADY")

# steady is "S"
steady = "S"

assert(meter1.state, "S")

---

# Nucleoid runs a for of statement without dependencies

# There is a Crate type
class Crate:
    pass

# crate1 is a Crate
crate1 = Crate()

# crate2 is a Crate
crate2 = Crate()

# FACTOR is 10
FACTOR = 10

# For each crate of Crate, while in the block, load is a local variable that is 5 times FACTOR,
# and the crate's load is load
for crate of Crate:
    load = 5 * FACTOR
    crate.load = load

# FACTOR is 20
FACTOR = 20

assert(crate1.load, 50)
assert(crate2.load, 50)

# For each crate of Crate, while in the block, load is a local variable that is 5 times FACTOR,
# and the crate's load is load
for crate of Crate:
    load = 5 * FACTOR
    crate.load = load

assert(crate1.load, 100)
assert(crate2.load, 100)

# crate3 is a Crate
crate3 = Crate()

assert(crate3.load, null)

---

# Nucleoid returns null from a function with no return statement

# seed is 1
seed = 1

# store assigns val to kept
def store(val):
    kept = val

# Call store with seed
store(seed)

# return: null

---

# Nucleoid returns a reference from a function call

# holder is an Object
holder = Object()

# index is 1
index = 1

# attach assigns holder to slot and returns the value of the assignment
def attach(slot):
    return slot = holder

assert(attach(index), {})
assert(index, 1)

---

# Nucleoid returns a string from a function call

# rename assigns "renamed" to a name and returns the value of the assignment
def rename(name):
    return name = "renamed"

# original is 1
original = 1

# Call rename with original
rename(original)

# return: "renamed"

---

# Nucleoid concatenates a number onto a string in a function

# suffix returns a plus 45
def suffix(a):
    return a + 45

# prefix is "REF-2"
prefix = "REF-2"

# Call suffix with prefix
suffix(prefix)

# return: "REF-245"

---

# Nucleoid rejects value as a property name in a block

# There is a Timer type
class Timer:
    pass

try:
    # while in the block, value is a local Timer,
    # and value's value is "06:00"
    {
        value = Timer()
        value.value = "06:00"
    }
catch error:
    assert(error, TypeError("Cannot use 'value' as a property"))

---

# Nucleoid rejects value as a property name in a value assignment

# There is a Field type
class Field:
    pass

# value is a Field
value = Field()

assert(value, { "id": "value" })

try:
    # value's value is 1024
    value.value = 1024
catch error:
    assert(error, TypeError("Cannot use 'value' as a property"))

---

# Nucleoid keeps the value of a local when the value property is used

# capacity is 480
capacity = 480

# doubled is null
doubled = null

# while in the block, half is a local variable that is capacity divided by 2,
# and doubled is half's value times 2
{
    half = capacity / 2
    doubled = half.value * 2
}

assert(doubled, 480)

---

# Nucleoid uses the value property in a class-level assignment with a counter

# serial is 0
serial = 0

# There is a Unit type
class Unit:
    pass

# unit1 is a Unit
unit1 = Unit()

# while in the block, any unit's tag is "U" plus serial's value,
# and serial is serial plus 1
{
    $Unit.tag = "U" + serial.value
    serial = serial + 1
}

assert(unit1.tag, "U0")

---

# Nucleoid uses the value property in an if condition on a property

# There is a Notice type
class Notice:
    pass

# notice1 is a Notice whose body is "Please read"
notice1 = Notice()
notice1.body = "Please read"

# if notice1's body is not notice1's body's value, then throw "NOTICE_LOCKED"
if notice1.body != notice1.body.value:
    throw "NOTICE_LOCKED"

assert(notice1.body, "Please read")

try:
    # notice1's body is "Please read again"
    notice1.body = "Please read again"
catch error:
    assert(error, "NOTICE_LOCKED")

---

# Nucleoid reads a class-level property through a chain of instance properties

# There is a Client type
class Client:
    pass

# There is a Site type
class Site:
    pass

# Any site's line is the site's city plus ", " plus the site's country
$Site.line = $Site.city + ", " + $Site.country

# client1 is a Client
client1 = Client()

# client1's site is a Site
client1.site = Site()

# client1's site's city is "Porto"
client1.site.city = "Porto"

# client1's site's country is "PT"
client1.site.country = "PT"

assert(client1.site.line, "Porto, PT")

---

# Nucleoid creates a dependency on behalf of a property with a reference

# There is a Form type
class Form:
    pass

# There is a Layout type
class Layout:
    pass

# form1 is a Form
form1 = Form()

# layout1 is a Layout whose kind is "A"
layout1 = Layout()
layout1.kind = "A"

# form1's layout is layout1
form1.layout = layout1

# form1's layout's title is form1's layout's kind plus "-FORM"
form1.layout.title = form1.layout.kind + "-FORM"

assert(layout1.title, "A-FORM")
assert(form1.layout.title, "A-FORM")

# layout1's kind is "B"
layout1.kind = "B"

assert(layout1.title, "B-FORM")
assert(form1.layout.title, "B-FORM")

---

# Nucleoid creates a dependency on behalf of a local variable with a reference

# There is a Score type
class Score:
    pass

# score1 is a Score whose points is 6
score1 = Score()
score1.points = 6

# There is a Round type
class Round:
    pass

# Any round's average is 0
$Round.average = 0

# Any round's entries is 0
$Round.entries = 0

# round1 is a Round
round1 = Round()

# score1's round is round1
score1.round = round1

# While in the block, round is score1's round,
# and round's average is round's average times round's entries plus score1's points,
# divided by round's entries plus 1,
# and round's entries is round's entries plus 1
{
    round = score1.round
    round.average = (round.average * round.entries + score1.points) / (round.entries + 1)
    round.entries = round.entries + 1
}

assert(round1.average, 6)
assert(round1.entries, 1)

# score1's points is 8
score1.points = 8

assert(round1.average, 7)

---

# Nucleoid creates an instance inside a block without a variable name

# There is a Sample type,
# which has a weight as a number
class Sample(weight: int):
    this.weight = weight

# Any sample's grams is the sample's weight times 1000
$Sample.grams = $Sample.weight * 1000

# weight is 3,
# and there is a Sample whose weight is weight
{
    weight = 3
    Sample(weight)
}

assert(Sample[0].weight, 3)
assert(Sample[0].grams, 3000)

---

# Nucleoid applies class-level rules to an instance created inside a block

# There is a Room type,
# which has a name as a string
class Room(name: str):
    this.name = name

# No room is locked
$Room.locked = false

# Any room's tag is "R-" plus the room's name
$Room.tag = "R-" + $Room.name

# name is "Atrium",
# and room1 is a Room whose name is name
{
    name = "Atrium"
    room1 = Room(name)
}

assert(room1.name, "Atrium")
assert(room1.tag, "R-Atrium")
assert(room1.locked, false)

---

# Nucleoid creates an instance in a block and assigns it to a property

# There is a Studio type
class Studio:
    pass

# There is a Session type
class Session:
    pass

# Any session's slot is the session's day as a date string plus " " plus the session's hour
$Session.slot = $Session.day.toDateString() + " " + $Session.hour

# studio1 is a Studio
studio1 = Studio()

# session is a Session whose day is February 2, 2020 and whose hour is "09:00",
# and studio1's session is session
{
    session = Session()
    session.day = Date("2020-2-2")
    session.hour = "09:00"
    studio1.session = session
}

assert(studio1.session.day.toDateString(), "Sun Feb 02 2020")
assert(studio1.session.slot, "Sun Feb 02 2020 09:00")

---

# Nucleoid updates a property assignment on an instance

# There is a Beam type
class Beam:
    pass

# beam1 is a Beam whose span is 10
beam1 = Beam()
beam1.span = 10

# beam1's load is beam1's span times 8
beam1.load = beam1.span * 8

assert(beam1.load, 80)

# beam1's load is beam1's span times 3
beam1.load = beam1.span * 3

assert(beam1.load, 30)

# beam1's span is 20
beam1.span = 20

assert(beam1.load, 60)
```
