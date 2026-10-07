# Nucleoid Language Reference - Synthesized Use Cases 03

```

# Nucleoid reports the type of a class, its list and an instance

# There is a Sensor type
class Sensor:
    pass

assert(typeof $Sensor, Class)
assert(typeof Sensor, List)

# sensor1 is a Sensor
sensor1 = Sensor()

assert(typeof sensor1, Object)

---

# Nucleoid counts declared types in the class list

assert(Class.length, 0)

# There is a Gauge type
class Gauge:
    pass

assert(Class.length, 1)

# There is a Meter type
class Meter:
    pass

assert(Class.length, 2)

---

# Nucleoid rejects a declaration without a definition

try:
    # capacity is declared as a number but not defined
    capacity: int
catch error:
    assert(error, ReferenceError("Missing definition"))

---

# Nucleoid preserves instances when a class is redefined with a constructor

# There is a Signal type
class Signal:
    pass

# No signal is muted
$Signal.muted = false

# signal1 is a Signal
signal1 = Signal()

# There is a Signal type,
# which has a band as a string
class Signal(band: str):
    this.band = band

assert(signal1.muted, false)
assert(signal1.band, null)

# signal2 is a Signal whose band is "VHF"
signal2 = Signal("VHF")

assert(signal2.muted, false)
assert(signal2.band, "VHF")

---

# Nucleoid deletes an instance addressed by its id

# There is a Badge type
class Badge:
    pass

# badge1 is a Badge
badge1 = Badge()

# badge2 is a Badge
badge2 = Badge()

assert(Badge["badge1"], { "id": "badge1" })

# The Badge whose id is "badge1" is deleted
delete Badge["badge1"]

assert(Badge["badge1"], null)
assert(Badge["badge2"], { "id": "badge2" })

---

# Nucleoid rejects deleting an instance that still has a property

# There is a Valve type
class Valve:
    pass

# valve1 is a Valve whose pressure is 30
valve1 = Valve()
valve1.pressure = 30

try:
    # valve1 is deleted
    delete valve1
catch error:
    assert(error, TypeError("Cannot delete object 'valve1'"))

assert(valve1.pressure, 30)

# valve1's pressure is deleted
delete valve1.pressure

# valve1 is deleted
delete valve1

assert(Valve["valve1"], null)

---

# Nucleoid returns a boolean when deleting an instance

# There is a Beacon type
class Beacon:
    pass

# beacon1 is a Beacon
beacon1 = Beacon()

# Deleting beacon1 returns true
assert(delete beacon1, true)

# Deleting beacon2, which is not defined, returns false
assert(delete beacon2, false)

---

# Nucleoid generates an id for an instance created without a variable name

# There is a Packet type
class Packet:
    pass

# There is a Packet
Packet()

assert(Packet.length, 1)
assert(Packet[0].id != null, true)

---

# Nucleoid runs several statements written on one line

# red is "RED", green is "GREEN" and blue is "BLUE"
red = "RED"; green = "GREEN"; blue = "BLUE"

assert(red, "RED")
assert(green, "GREEN")
assert(blue, "BLUE")

---

# Nucleoid slices the end of a string

# code is "ORDER-2024-0007"
code = "ORDER-2024-0007"

# tail is the last four characters of code
tail = code[-4:]

assert(tail, "0007")

# code is "ORDER-2024-0042"
code = "ORDER-2024-0042"

assert(tail, "0042")

---

# Nucleoid slices a range from a string

# stamp is "2024-05-17"
stamp = "2024-05-17"

# year is the first four characters of stamp
year = stamp[:4]

# day is the last two characters of stamp
day = stamp[8:]

assert(year, "2024")
assert(day, "17")

---

# Nucleoid reads the character at an index of a string

# label is "NUCLEOID"
label = "NUCLEOID"

# index is 0
index = 0

# initial is the character of label at index
initial = label.charAt(index)

assert(initial, "N")

# index is 4
index = 4

assert(initial, "E")

---

# Nucleoid replaces part of a string as a dependency

# path is "/api/v1/orders"
path = "/api/v1/orders"

# version is "v2"
version = "v2"

# upgraded is path with "v1" replaced by version
upgraded = path.replace("v1", version)

assert(upgraded, "/api/v2/orders")

# version is "v3"
version = "v3"

assert(upgraded, "/api/v3/orders")

---

# Nucleoid builds a string from a character code

# offset is 0
offset = 0

# letter is the character with the code 65 plus offset
letter = String.fromCharCode(65 + offset)

assert(letter, "A")

# offset is 2
offset = 2

assert(letter, "C")

---

# Nucleoid raises a number to a power in a dependency

# side is 3
side = 3

# cube is side raised to the power of 3
cube = Math.pow(side, 3)

assert(cube, 27)

# side is 4
side = 4

assert(cube, 64)

---

# Nucleoid takes the square root in a class-level property

# There is a Square type
class Square:
    pass

# Any square's side is the square root of the square's area
$Square.side = Math.sqrt($Square.area)

# square1 is a Square whose area is 144
square1 = Square()
square1.area = 144

assert(square1.side, 12)

# square1's area is 169
square1.area = 169

assert(square1.side, 13)

---

# Nucleoid takes the maximum of two dependencies

# bid is 120
bid = 120

# floorPrice is 100
floorPrice = 100

# price is the maximum of bid and floorPrice
price = Math.max(bid, floorPrice)

assert(price, 120)

# bid is 80
bid = 80

assert(price, 100)

---

# Nucleoid takes the minimum of two dependencies

# requested is 50
requested = 50

# available is 30
available = 30

# granted is the minimum of requested and available
granted = Math.min(requested, available)

assert(granted, 30)

# available is 70
available = 70

assert(granted, 50)

---

# Nucleoid reads the largest integer from the number object

# limit is the maximum integer
limit = Number.MAX_INTEGER

assert(limit, 9007199254740991)

---

# Nucleoid creates a boolean with the standard built-in object

# enabled is a Boolean that is true
enabled = Boolean(true)

assert(enabled, true)

# disabled is a Boolean that is false
disabled = Boolean(false)

assert(disabled, false)

---

# Nucleoid parses a date string into a number

# epoch is the parsed date of "04 Dec 1995 00:12:00 GMT"
epoch = Date.parse("04 Dec 1995 00:12:00 GMT")

assert(epoch, 818035920000)

---

# Nucleoid reads the year of a date

# launch is July 24, 2019
launch = Date("2019-7-24")

assert(launch.getYear(), 119)
assert(launch.toDateString(), "Wed Jul 24 2019")

---

# Nucleoid measures the difference between two dates

# start is January 1, 2020
start = Date("2020-1-1")

# finish is January 8, 2020
finish = Date("2020-1-8")

# span is finish's time minus start's time
span = finish.getTime() - start.getTime()

assert(span, 604800000)

---

# Nucleoid creates a plain object and assigns properties to it

# origin is an Object whose latitude is 40.7 and whose longitude is 74.2
origin = Object()
origin.latitude = 40.7
origin.longitude = 74.2

assert(origin.latitude, 40.7)
assert(origin.longitude, 74.2)
assert(origin.altitude, null)

---

# Nucleoid creates a variable from a JSON object with nested data

# payload is an object whose kind is "ORDER"
# and whose item's sku is "A-1"
payload = { "kind": "ORDER", "item": { "sku": "A-1" } }

assert(payload.kind, "ORDER")
assert(payload.item.sku, "A-1")

---

# Nucleoid returns an inline object with bare keys

# While in the block, return an object whose min is 1 and whose max is 9
{
    return { min: 1, max: 9 }
}

# return: { "min": 1, "max": 9 }

---

# Nucleoid returns an inline list of objects from a block

# While in the block, return a list of an object whose code is "A"
# and an object whose code is "B"
{
    return [{ "code": "A" }, { "code": "B" }]
}

# return: [{ "code": "A" }, { "code": "B" }]

---

# Nucleoid uses only the value when a property references itself

# There is a Budget type
class Budget:
    pass

# budget1 is a Budget whose amount is 500
budget1 = Budget()
budget1.amount = 500

# budget1's amount is budget1's amount plus 250
budget1.amount = budget1.amount + 250

assert(budget1.amount, 750)

---

# Nucleoid uses the value of a null property as zero

# There is a Fee type
class Fee:
    pass

# fee1 is a Fee whose rate is 5 and whose base is null
fee1 = Fee()
fee1.rate = 5
fee1.base = null

# fee1's total is fee1's rate times fee1's base's value
fee1.total = fee1.rate * fee1.base.value

assert(fee1.total, 0)

# fee1's base is 200
fee1.base = 200

assert(fee1.total, 0)

---

# Nucleoid rejects the value of a property that is not defined

# There is a Route type
class Route:
    pass

# route1 is a Route whose speed is 80
route1 = Route()
route1.speed = 80

# route1's duration is route1's distance divided by route1's speed
route1.duration = route1.distance / route1.speed

assert(route1.duration, null)

try:
    # route1's time is route1's distance's value divided by route1's speed
    route1.time = route1.distance.value / route1.speed
catch error:
    assert(error, ReferenceError("route1.distance is not defined"))

---

# Nucleoid uses the value property on a class-level property chain

# There is a Report type,
# which has a survey as a Survey
class Report(survey):
    this.survey = survey

# There is a Survey type
class Survey:
    pass

# Any report's total is the value of the report's survey's total
$Report.total = $Report.survey.total.value

# survey1 is a Survey whose total is 20
survey1 = Survey()
survey1.total = 20

# report1 is a Report whose survey is survey1
report1 = Report(survey1)

assert(report1.total, 20)

# survey1's total is 25
survey1.total = 25

assert(survey1.total, 25)
assert(report1.total, 20)

---

# Nucleoid iterates a list of instances in a for of statement

# There is a Node type
class Node:
    pass

# node1 is a Node
node1 = Node()

# node2 is a Node
node2 = Node()

# nodes is a list of node1 and node2
nodes = []
nodes.push(node1)
nodes.push(node2)

# For each node of nodes, the node's active is true
for node of nodes:
    node.active = true

assert(node1.active, true)
assert(node2.active, true)

---

# Nucleoid nests a function call inside a regular expression test

# tag is "A1"
tag = "A1"

# position is 0
position = 0

# if the character of tag at position is not an uppercase letter,
# then throw "INVALID_TAG"
if not /[A-Z]/.test(tag.charAt(position)):
    throw "INVALID_TAG"

assert(tag, "A1")

try:
    # tag is "1A"
    tag = "1A"
catch error:
    assert(error, "INVALID_TAG")

---

# Nucleoid returns the value of an assignment from a function

# reset assigns 0 to a counter and returns the value of the assignment
def reset(counter):
    return counter = 0

# tally is 7
tally = 7

# Call reset with tally
reset(tally)

# return: 0

---

# Nucleoid returns the first return statement in a block

# While in the block, return 42, and return "later"
{
    return 42
    return "later"
}

# return: 42

---

# Nucleoid adds an instance to a list from a class-level block

# There is a Reading type
class Reading:
    pass

# highs is an empty list
highs = []

# reading1 is a Reading whose level is 9
reading1 = Reading()
reading1.level = 9

# While in the block, level is any reading's level,
# and if level is greater than 5, then the reading is added to highs
{
    level = $Reading.level
    if level > 5:
        highs.push($Reading)
}

assert(highs.pop(), reading1)

---

# Nucleoid removes the last element of a list as a dependency

# queue is a list of 1, 2 and 3
queue = [1, 2, 3]

# size is the length of queue
size = queue.length

assert(size, 3)

# Remove the last item from queue
queue.pop()

assert(size, 2)

---

# Nucleoid rejects writing through an undefined property of an instance

# There is a Printer type
class Printer:
    pass

# printer1 is a Printer
printer1 = Printer()

try:
    # printer1's tray's size is "A4"
    printer1.tray.size = "A4"
catch error:
    assert(error, ReferenceError("printer1.tray is not defined"))

---

# Nucleoid rejects a class-level write through an undefined reference

# There is a Camera type
class Camera:
    pass

try:
    # any camera's lens's zoom is 3
    $Camera.lens.zoom = 3
catch error:
    assert(error, ReferenceError("Camera.lens is not defined"))

---

# Nucleoid throws a number as an error

try:
    # throw 503
    throw 503
catch error:
    assert(error, 503)

---

# Nucleoid throws a reference error when the thrown value is not defined

try:
    # throw reason
    throw reason
catch error:
    assert(error, ReferenceError("reason is not defined"))

---

# Nucleoid rejects defining a class-level rule inside a block

# There is a Panel type
class Panel:
    pass

# panel1 is a Panel whose width is 4 and whose height is 3
panel1 = Panel()
panel1.width = 4
panel1.height = 3

try:
    # while in the block, width is a local variable that is panel1's width,
    # and height is a local variable that is panel1's height,
    # and any panel's area is width times height
    {
        width = panel1.width
        height = panel1.height
        $Panel.area = width * height
    }
catch error:
    assert(error, SyntaxError("Cannot define class declaration in non-class block"))

---

# Nucleoid reads the innermost binding of a shadowed name

# depth is "outer"
depth = "outer"

# reading is null
reading = null

# a local depth shadows the outer depth inside the block,
# and a nested block shadows it again,
# and reading is the innermost depth
{
    depth = "inner"
    {
        depth = "innermost"
        reading = depth
    }
}

assert(reading, "innermost")

---

# Nucleoid applies a class-level rule to instances made before and after it

# There is a Tile type
class Tile:
    pass

# tile1 is a Tile whose edge is 3
tile1 = Tile()
tile1.edge = 3

# Any tile's area is the tile's edge times the tile's edge
$Tile.area = $Tile.edge * $Tile.edge

assert(tile1.area, 9)

# tile2 is a Tile whose edge is 5
tile2 = Tile()
tile2.edge = 5

assert(tile2.area, 25)

# tile1's edge is 4
tile1.edge = 4

assert(tile1.area, 16)
assert(tile2.area, 25)

---

# Nucleoid updates a conditional property when the branch value changes

# There is a Plan type
class Plan:
    pass

# plan1 is a Plan whose tier is "FREE"
plan1 = Plan()
plan1.tier = "FREE"

# paid is "PAID" and free is "FREE"
paid = "PAID"; free = "FREE"

# if plan1's tier is "PRO", then plan1's label is paid,
# else plan1's label is free
if plan1.tier == "PRO":
    plan1.label = paid
else:
    plan1.label = free

assert(plan1.label, "FREE")

# free is "Free tier"
free = "Free tier"

assert(plan1.label, "Free tier")

# plan1's tier is "PRO"
plan1.tier = "PRO"

assert(plan1.label, "PAID")
```
