# Nucleoid Language Reference - Synthesized Use Cases 05

```

# Nucleoid asserts a comparison in an expression statement

# volts is 12
volts = 12

assert(volts == 12, true)
assert(volts + 8, 20)
assert(volts > 20, false)

---

# Nucleoid returns the value of a variable

# amps is 5
amps = 5

amps

# return: 5

---

# Nucleoid returns a negative number from a variable

# offset is -7
offset = -7

offset

# return: -7

---

# Nucleoid returns the assigned value of a variable assignment

# ohms is 9
ohms = 9

# return: 9

---

# Nucleoid throws an error raised inside a conditional block

# depth is 120
depth = 120

try:
    # if depth is greater than or equal to 100, then throw "TOO_DEEP"
    if depth >= 100:
        throw "TOO_DEEP"
catch error:
    assert(error, "TOO_DEEP")

---

# Nucleoid creates an if statement on a boolean variable

# armed is false
armed = false

# ready is false
ready = false

# if armed is true, then ready is armed and true
if armed == true:
    ready = armed and true

assert(ready, false)

# armed is true
armed = true

assert(ready, true)

---

# Nucleoid creates an else statement on a variable

# ratio is 0.9
ratio = 0.9

# rich is "RICH"
rich = "RICH"

# lean is "LEAN"
lean = "LEAN"

# if ratio is greater than 0.5, then mixture is rich,
# else mixture is lean
if ratio > 0.5:
    mixture = rich
else:
    mixture = lean

assert(mixture, "RICH")

# ratio is 0.2
ratio = 0.2

assert(mixture, "LEAN")

# lean is "L"
lean = "L"

assert(mixture, "L")

---

# Nucleoid creates an else if statement on a variable

# altitude is 12
altitude = 12

# sea is 1013
sea = 1013

# alpine is 850
alpine = 850

# factor is 2
factor = 2

# if altitude is greater than 10, then pressure is sea times factor,
# else if altitude is greater than 3, then pressure is alpine times factor
if altitude > 10:
    pressure = sea * factor
else if altitude > 3:
    pressure = alpine * factor

# altitude is 5
altitude = 5

assert(pressure, 1700)

# alpine is 860
alpine = 860

assert(pressure, 1720)

---

# Nucleoid creates multiple else if statements on a variable

# delta is -2
delta = -2

# weight is 1
weight = 1

# if delta is greater than 10, then adjustment is delta times weight times 3,
# else if delta is greater than 0, then adjustment is delta times weight times 2,
# else adjustment is delta times weight
if delta > 10:
    adjustment = delta * weight * 3
else if delta > 0:
    adjustment = delta * weight * 2
else:
    adjustment = delta * weight

assert(adjustment, -2)

# weight is 4
weight = 4

assert(adjustment, -8)

---

# Nucleoid creates an if statement on a property

# There is a Lamp type
class Lamp:
    pass

# lamp1 is a Lamp whose colour is "AMBER"
lamp1 = Lamp()
lamp1.colour = "AMBER"

# if lamp1's colour is "GREEN", then lamp1's state is "GO"
if lamp1.colour == "GREEN":
    lamp1.state = "GO"

assert(lamp1.state, null)

# lamp1's colour is "GREEN"
lamp1.colour = "GREEN"

assert(lamp1.state, "GO")

---

# Nucleoid updates an if block on a property

# There is a Wallet type
class Wallet:
    pass

# wallet1 is a Wallet whose balance is 200
wallet1 = Wallet()
wallet1.balance = 200

# if wallet1's balance is less than 500, then wallet1's tier is "BASIC"
if wallet1.balance < 500:
    wallet1.tier = "BASIC"

assert(wallet1.tier, "BASIC")

# if wallet1's balance is less than 500, then wallet1's tier is "STARTER"
if wallet1.balance < 500:
    wallet1.tier = "STARTER"

assert(wallet1.tier, "STARTER")

---

# Nucleoid creates an else statement on a property with property dependencies

# There is an Entry type
class Entry:
    pass

# entry1 is an Entry whose kind is "PERSON",
# whose given is "Ada" and whose family is "Lovelace"
entry1 = Entry()
entry1.kind = "PERSON"
entry1.given = "Ada"
entry1.family = "Lovelace"

# if entry1's kind is "COMPANY", then entry1's title is "C " plus entry1's given,
# else entry1's title is entry1's given plus " " plus entry1's family
if entry1.kind == "COMPANY":
    entry1.title = "C " + entry1.given
else:
    entry1.title = entry1.given + " " + entry1.family

assert(entry1.title, "Ada Lovelace")

# entry1's given is "A" and entry1's family is "L"
entry1.given = "A"
entry1.family = "L"

assert(entry1.title, "A L")

# entry1's kind is "COMPANY"
entry1.kind = "COMPANY"

assert(entry1.title, "C A")

---

# Nucleoid creates multiple else if statements on a property

# There is a Shipment type
class Shipment:
    pass

# shipment1 is a Shipment whose weight is 400 and whose zone is 1
shipment1 = Shipment()
shipment1.weight = 400
shipment1.zone = 1

# unit is 3
unit = 3

# if shipment1's zone is greater than 4,
# then shipment1's cost is shipment1's weight times unit divided by 100 plus 20,
# else if shipment1's zone is greater than 2,
# then shipment1's cost is shipment1's weight times unit divided by 100 plus 10,
# else shipment1's cost is shipment1's weight times unit divided by 100
if shipment1.zone > 4:
    shipment1.cost = shipment1.weight * unit / 100 + 20
else if shipment1.zone > 2:
    shipment1.cost = shipment1.weight * unit / 100 + 10
else:
    shipment1.cost = shipment1.weight * unit / 100

assert(shipment1.cost, 12)

# unit is 5
unit = 5

assert(shipment1.cost, 20)

# shipment1's zone is 3
shipment1.zone = 3

assert(shipment1.cost, 30)

# shipment1's zone is 5
shipment1.zone = 5

assert(shipment1.cost, 40)

---

# Nucleoid creates a class-level conditional before instantiation

# There is a Permit type
class Permit:
    pass

# Any permit issued after January 1, 2000 is current
if $Permit.issued > Date("2000-1-1"):
    $Permit.status = "CURRENT"

# permit1 is a Permit
permit1 = Permit()

assert(permit1.status, null)

# permit1's issued is February 1, 2000
permit1.issued = Date("2000-2-1")

assert(permit1.status, "CURRENT")

# permit2 is a Permit
permit2 = Permit()

assert(permit2.status, null)

---

# Nucleoid creates a class-level conditional after instantiation

# There is a Pupil type
class Pupil:
    pass

# pupil1 is a Pupil whose age is 6 and whose group is "Infants"
pupil1 = Pupil()
pupil1.age = 6
pupil1.group = "Infants"

# pupil2 is a Pupil whose age is 6 and whose group is "Infants"
pupil2 = Pupil()
pupil2.age = 6
pupil2.group = "Infants"

# Any pupil whose age is 7 is in Juniors
if $Pupil.age == 7:
    $Pupil.group = "Juniors"

assert(pupil1.group, "Infants")
assert(pupil2.group, "Infants")

# pupil1's age is 7
pupil1.age = 7

assert(pupil1.group, "Juniors")
assert(pupil2.group, "Infants")

---

# Nucleoid updates a class-level if block

# There is a Bin type
class Bin:
    pass

# bin1 is a Bin whose count is 0
bin1 = Bin()
bin1.count = 0

# bin2 is a Bin whose count is 50
bin2 = Bin()
bin2.count = 50

# Any bin whose count is 0 needs refilling
if $Bin.count == 0:
    $Bin.refill = true

assert(bin1.refill, true)
assert(bin2.refill, null)

# Any bin whose count is 0 does not need refilling
if $Bin.count == 0:
    $Bin.refill = false

assert(bin1.refill, false)
assert(bin2.refill, null)

---

# Nucleoid creates a class-level else statement before instantiation

# There is a Queue type
class Queue:
    pass

# If any queue's depth is greater than 100, then the queue's mode is busy,
# else the queue's mode is calm
if $Queue.depth > 100:
    $Queue.mode = busy
else:
    $Queue.mode = calm

# busy is "BUSY"
busy = "BUSY"

# calm is "CALM"
calm = "CALM"

# queue1 is a Queue
queue1 = Queue()

# queue1's depth is 20
queue1.depth = 20

assert(queue1.mode, "CALM")

# calm is "C"
calm = "C"

assert(queue1.mode, "C")

---

# Nucleoid creates multiple class-level else if statements after instantiation

# There is a Figure type
class Figure:
    pass

# figure1 is a Figure whose kind is "RECTANGLE",
# whose base is 5 and whose height is 6
figure1 = Figure()
figure1.kind = "RECTANGLE"
figure1.base = 5
figure1.height = 6

# If any figure's kind is "SQUARE", then the figure's area is the figure's base squared,
# else if any figure's kind is "TRIANGLE",
# then the figure's area is the figure's base times the figure's height divided by 2,
# else the figure's area is the figure's base times the figure's height
if $Figure.kind == "SQUARE":
    $Figure.area = Math.pow($Figure.base, 2)
else if $Figure.kind == "TRIANGLE":
    $Figure.area = $Figure.base * $Figure.height / 2
else:
    $Figure.area = $Figure.base * $Figure.height

assert(figure1.area, 30)

# figure1's base is 7
figure1.base = 7

assert(figure1.area, 42)

# figure1's kind is "TRIANGLE"
figure1.kind = "TRIANGLE"

assert(figure1.area, 21)

---

# Nucleoid creates a class-level property before instantiation

# There is a Poll type
class Poll:
    pass

# Any poll's mean is the poll's total divided by 4
$Poll.mean = $Poll.total / 4

# poll1 is a Poll
poll1 = Poll()

assert(poll1.mean, null)

# poll1's total is 26
poll1.total = 26

assert(poll1.mean, 6.5)

---

# Nucleoid runs a block statement on a property

# There is a Parcel type
class Parcel:
    pass

# parcel1 is a Parcel
parcel1 = Parcel()

# parcel1's ref is "0000001"
parcel1.ref = "0000001"

# while in the block, marked is a local variable that is "UK" plus parcel1's ref,
# and parcel1's marked is marked
{
    marked = "UK" + parcel1.ref
    parcel1.marked = marked
}

assert(parcel1.marked, "UK0000001")

# parcel1's ref is "0000002"
parcel1.ref = "0000002"

assert(parcel1.marked, "UK0000002")

---

# Nucleoid runs a nested block statement on a property

# There is a Block type
class Block:
    pass

# block1 is a Block
block1 = Block()

# block1's side is 9
block1.side = 9

# block1's depth is 10
block1.depth = 10

# while in the block, face is a local variable that is block1's side squared,
# and in a nested block, block1's volume is face times block1's depth
{
    face = Math.pow(block1.side, 2)
    {
        block1.volume = face * block1.depth
    }
}

assert(block1.volume, 810)

# block1's depth is 9
block1.depth = 9

assert(block1.volume, 729)

---

# Nucleoid runs a nested if statement on a property

# There is an Order type
class Order:
    pass

# order1 is an Order
order1 = Order()

# order1's unit is 50
order1.unit = 50

# order1's count is 2
order1.count = 2

# while in the block, total is a local variable that is order1's unit times order1's count,
# and if total is greater than 100, then order1's levy is total times 10 divided by 100
{
    total = order1.unit * order1.count
    if total > 100:
        order1.levy = total * 10 / 100
}

assert(order1.levy, null)

# order1's count is 3
order1.count = 3

assert(order1.levy, 15)

---

# Nucleoid creates a nested else statement on a property

# There is a Gauge type
class Gauge:
    pass

# gauge1 is a Gauge
gauge1 = Gauge()

# gauge1's percent is 1
gauge1.percent = 1

# over is "OVER"
over = "OVER"

# under is "UNDER"
under = "UNDER"

# while in the block, share is a local variable that is gauge1's percent divided by 100,
# and if share is greater than 1, then gauge1's state is over,
# else gauge1's state is under
{
    share = gauge1.percent / 100
    if share > 1:
        gauge1.state = over
    else:
        gauge1.state = under
}

assert(gauge1.state, "UNDER")

# under is "U"
under = "U"

assert(gauge1.state, "U")

---

# Nucleoid runs dependent statements in the same transaction

# There is a Fleet type,
# and any fleet's code is "FL-" plus the fleet's number
class Fleet:
    pass

$Fleet.code = "FL-" + $Fleet.number

# fleet1 is a Fleet
fleet1 = Fleet()

# fleet1's number is "8842"
fleet1.number = "8842"

assert(fleet1.code, "FL-8842")

---

# Nucleoid searches a local variable in scope before the state

# phi is 1.618
phi = 1.618

# reading is null
reading = null

# a local phi shadows the outer phi inside the block,
# and reading is the local phi
{
    phi = 2
    reading = phi
}

assert(reading, 2)

---

# Nucleoid assigns a variable to a reference

# source is 1
source = 1

# mirror is source
mirror = source

assert(mirror, 1)

# source is 2
source = 2

assert(mirror, 2)

---

# Nucleoid creates a property assignment after a property is declared

# There is an Account type
class Account:
    pass

# account1 is an Account
account1 = Account()

# account1's handle is "ada"
account1.handle = "ada"

# account1's address is account1's handle plus "@example.com"
account1.address = account1.handle + "@example.com"

assert(account1.address, "ada@example.com")

# account1's handle is "ada2"
account1.handle = "ada2"

assert(account1.address, "ada2@example.com")

---

# Nucleoid assigns an instance to a property before its class rule exists

# There is a Courier type
class Courier:
    pass

# There is a Leg type
class Leg:
    pass

# Any leg's length is the square root of the leg's x squared plus the leg's y squared
$Leg.length = Math.sqrt($Leg.x * $Leg.x + $Leg.y * $Leg.y)

# courier1 is a Courier
courier1 = Courier()

# courier1's leg is a Leg
courier1.leg = Leg()

assert(courier1.leg.length, null)

# courier1's leg's x is 6 and its y is 8
courier1.leg.x = 6
courier1.leg.y = 8

assert(courier1.leg.length, 10)

---

# Nucleoid assigns an instance to a property after its class rule exists

# There is a Crop type
class Crop:
    pass

# crop1 is a Crop
crop1 = Crop()

# There is a Grade type
class Grade:
    pass

# crop1's grade is a Grade whose score is 25
crop1.grade = Grade()
crop1.grade.score = 25

assert(crop1.grade.band, null)

# Any grade's band is the character with the code 65
# plus the floor of the grade's score divided by 10
$Grade.band = String.fromCharCode(65 + Math.floor($Grade.score / 10))

assert(crop1.grade.band, "C")

---

# Nucleoid creates a class-level property from a chain of two classes

# There is a Host type
class Host:
    pass

# There is a Kernel type
class Kernel:
    pass

# Any host's kernel is a Kernel
$Host.kernel = Kernel()

# host1 is a Host
host1 = Host()

# host1's kernel's release is 6
host1.kernel.release = 6

# Any host's stamp is the host's kernel's release plus ".1024"
$Host.stamp = $Host.kernel.release + ".1024"

assert(host1.stamp, "6.1024")

# host1's kernel's release is 7
host1.kernel.release = 7

assert(host1.stamp, "7.1024")

---

# Nucleoid creates a local variable in a block and uses it in an assignment

# metres is 60
metres = 60

# paces is null
paces = null

# while in the block, steps is a local variable that is metres divided by 2,
# and paces is steps times 2
{
    steps = metres / 2
    paces = steps * 2
}

assert(paces, 60)

# metres is 80
metres = 80

assert(paces, 80)

---

# Nucleoid creates a built-in object as a local variable inside a block

# while in the block, flag is a local Boolean that is false,
# and locked is flag
{
    flag = Boolean(false)
    locked = flag
}

assert(locked, false)

---

# Nucleoid creates a built-in object as a property of a local variable

# There is a Delivery type
class Delivery:
    pass

# While in the block, delivery is a local Delivery whose date is April 4, 2019,
# and delivery1 is delivery
{
    delivery = Delivery()
    delivery.date = Date("2019-4-4")
    delivery1 = delivery
}

assert(delivery1.date.toDateString(), "Thu Apr 04 2019")

---

# Nucleoid creates a property of a local variable found by id

# There is a Member type
class Member:
    pass

# member0 is a Member
member0 = Member()

# While in the block, member is a local variable that is the Member whose id is "member0",
# and if there is a member, then the member's role is "ADMIN"
{
    member = Member["member0"]
    if member:
        member.role = "ADMIN"
}

assert(member0.role, "ADMIN")

---

# Nucleoid rejects a property assignment on an undefined property of a local

# There is a Booking type
class Booking:
    pass

try:
    # while in the block, booking is a local Booking,
    # and booking's venue's city is "LISBON"
    {
        booking = Booking()
        booking.venue.city = "LISBON"
    }
catch error:
    assert(error, ReferenceError("booking.venue is not defined"))

---

# Nucleoid returns a local variable found by a query from a block

# There is a Probe type,
# which has a code as a string
class Probe(code: str):
    this.code = code

# probe1 is a Probe whose code is "P0"
probe1 = Probe("P0")

# probe2 is a Probe whose code is "P1"
probe2 = Probe("P1")

# While in the block, probe is a local variable that is the Probe whose code is "P0",
# and if there is no probe, then throw "INVALID_PROBE",
# and return probe
{
    probe = Probe.find(p => p.code == "P0")
    if not probe:
        throw "INVALID_PROBE"
    return probe
}

# return: { "id": "probe1", "code": "P0" }

---

# Nucleoid updates a query when its parameter changes

# There is a Runner type
class Runner:
    pass

# runner1 is a Runner whose bib is 11
runner1 = Runner()
runner1.bib = 11

# runner2 is a Runner whose bib is 12
runner2 = Runner()
runner2.bib = 12

# runner3 is a Runner whose bib is 13
runner3 = Runner()
runner3.bib = 13

# wanted is 12
wanted = 12

# found is the Runner whose bib is wanted
found = Runner.find(r => r.bib == wanted)

assert(found, runner2)
assert(found, { "id": "runner2", "bib": 12 })

# wanted is 13
wanted = 13

assert(found, runner3)
assert(found, { "id": "runner3", "bib": 13 })

---

# Nucleoid shadows a parameter inside a lambda

# readings is a list of 10, 20, 30 and 40
readings = [10, 20, 30, 40]

# element is 15
element = 15

# The element parameter shadows the outer element in each function
assert(readings.find(function(element) { return element == 30 }), 30)
assert(readings.find(element => { return element == 20 }), 20)
assert(readings.find(element => element == 10), 10)
assert(element, 15)

---

# Nucleoid rejects an unknown function of the date object

try:
    # stamp is the missing date
    stamp = Date.missing()
catch error:
    assert(error, TypeError("Date.missing is not a function"))

---

# Nucleoid creates a date from the time of another date

# first is the current date
first = Date()

# second is a date whose time is first's time
second = Date(first.getTime())

assert(first.getTime() == second.getTime(), true)

---

# Nucleoid returns the instance itself when an instance is created

# There is a Coupon type,
# which has an amount as a number
class Coupon(amount: int):
    this.amount = amount

# There is a Coupon whose amount is 250
Coupon(250)

assert(Coupon[0].amount, 250)
assert(Coupon[0].id != null, true)

# return: { "id": "[UUID]", "amount": 250 }

---

# Nucleoid deletes an instance and clears its address

# There is a Marker type
class Marker:
    pass

# marker1 is a Marker
marker1 = Marker()

# marker1 is deleted
delete marker1

assert(Marker["marker1"], null)
assert(Marker.find(marker => marker.id == "marker1"), null)

try:
    # marker1
    marker1
catch error:
    assert(error, ReferenceError("marker1 is not defined"))

---

# Nucleoid rejects deleting an instance that holds an instance as a property

# There is a Frame type
class Frame:
    pass

# There is a Glass type
class Glass:
    pass

# frame1 is a Frame
frame1 = Frame()

# frame1's glass is a Glass
frame1.glass = Glass()

try:
    # frame1 is deleted
    delete frame1
catch error:
    assert(error, TypeError("Cannot delete object 'frame1'"))

# frame1's glass is deleted
delete frame1.glass

# frame1 is deleted
delete frame1

assert(Frame["frame1"], null)

---

# Nucleoid rolls back a property assignment if a class-level rule throws

# There is a Part type
class Part:
    pass

# If any part's code is 'X', then throw 'INVALID_CODE'
if $Part.code == 'X':
    throw 'INVALID_CODE'

# part1 is a Part
part1 = Part()

try:
    # part1's code is 'X'
    part1.code = 'X'
catch error:
    assert(error, "INVALID_CODE")

assert(part1.code, null)

---

# Nucleoid creates a variable from JSON inside a block

# While in the block, request is a local variable whose path is "/health",
# and echo is a local variable whose target is request's path
{
    request = { "path": "/health" }
    echo = { "target": request.path }
    assert(echo.target, "/health")
}

# reply is an object whose status is 200
reply = { "status": 200 }

assert(reply.status, 200)
```
