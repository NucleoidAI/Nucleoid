# Nucleoid Language Reference - Synthesized Use Cases 06

```

# Nucleoid converts a weight through a chain of variables

# grams is 5000
grams = 5000

# kilograms is grams divided by 1000
kilograms = grams / 1000

# pounds is kilograms times 2.2
pounds = kilograms * 2.2

assert(kilograms, 5)
assert(pounds, 11)

# grams is 10000
grams = 10000

assert(kilograms, 10)
assert(pounds, 22)

---

# Nucleoid replaces the dependency of a shipping cost

# base is 12
base = 12

# express is 30
express = 30

# postage is base plus 3
postage = base + 3

assert(postage, 15)

# postage is express plus 3
postage = express + 3

assert(postage, 33)

# express is 40
express = 40

assert(postage, 43)

---

# Nucleoid adds to a pallet count using its own value

# pallets is 24
pallets = 24

# pallets is pallets plus 6
pallets = pallets + 6

assert(pallets, 30)

---

# Nucleoid fixes a tariff against later changes with the value property

# tariff is 4
tariff = 4

# volume is 25
volume = 25

# charge is tariff's value times volume
charge = tariff.value * volume

assert(charge, 100)

# tariff is 5
tariff = 5

assert(charge, 100)

# volume is 50
volume = 50

assert(charge, 200)

---

# Nucleoid clears a manifest total when its source is deleted

# crates is 8
crates = 8

# weight is crates times 15
weight = crates * 15

# manifest is weight plus 20
manifest = weight + 20

assert(manifest, 140)

# crates is deleted
delete crates

assert(weight, null)
assert(manifest, null)

try:
    # crates
    crates
catch error:
    assert(error, ReferenceError("crates is not defined"))

---

# Nucleoid assigns an overload comparison to a variable

# capacity is 1000
capacity = 1000

# loaded is 1200
loaded = 1200

# overloaded is whether loaded is greater than capacity
overloaded = loaded > capacity

assert(overloaded, true)

# loaded is 800
loaded = 800

assert(overloaded, false)

---

# Nucleoid builds a tracking link from a template literal

# carrier is "royal"
carrier = "royal"

# parcel is 55210
parcel = 55210

# link is the carrier and parcel in a template
link = `https://${carrier}.example.com/${parcel}`

assert(link, "https://royal.example.com/55210")

# carrier is "swift"
carrier = "swift"

assert(link, "https://swift.example.com/55210")

---

# Nucleoid combines availability flags with logical operators

# stocked is true
stocked = true

# reserved is false
reserved = false

assert(stocked and not reserved, true)
assert(stocked && !reserved, true)
assert(reserved or stocked, true)
assert(reserved || stocked, true)

---

# Nucleoid respects parentheses in a freight formula

# distance is 3
distance = 3

# rate is 4
rate = 4

# surcharge is 5
surcharge = 5

assert(distance + rate * surcharge, 23)
assert((distance + rate) * surcharge, 35)

---

# Nucleoid splits a batch with the remainder operator

# units is 47
units = 47

# perBox is 10
perBox = 10

# loose is units modulo perBox
loose = units % perBox

assert(loose, 7)

# units is 52
units = 52

assert(loose, 2)

---

# Nucleoid appends a sequence number to a depot code

# depot is "DEP-"
depot = "DEP-"

# sequence is 91
sequence = 91

# reference is depot plus sequence
reference = depot + sequence

assert(reference, "DEP-91")

# sequence is 92
sequence = 92

assert(reference, "DEP-92")

---

# Nucleoid counts the characters of a container code

# container is "MSKU"
container = "MSKU"

# width is container's length
width = container.length

assert(width, 4)

# container is "MSKU1234"
container = "MSKU1234"

assert(width, 8)

---

# Nucleoid lowercases a route name as a dependency

# route is "NORTH"
route = "NORTH"

# slug is route lowercased
slug = route.lower()

assert(slug, "north")

# route is "SOUTH"
route = "SOUTH"

assert(slug, "south")

---

# Nucleoid reads the leading letter of a bay code

# bay is "K12"
bay = "K12"

# aisle is the character of bay at 0
aisle = bay.charAt(0)

assert(aisle, "K")

# bay is "M12"
bay = "M12"

assert(aisle, "M")

---

# Nucleoid rewrites a warehouse path with replace

# location = "zone/a/shelf"
location = "zone/a/shelf"

# zone is "b"
zone = "b"

# moved is location with "a" replaced by zone
moved = location.replace("a", zone)

assert(moved, "zone/b/shelf")

# zone is "c"
zone = "c"

assert(moved, "zone/c/shelf")

---

# Nucleoid takes the checksum digits from the end of a barcode

# barcode is "50098765432"
barcode = "50098765432"

# checksum is the last two characters of barcode
checksum = barcode[-2:]

assert(checksum, "32")

# barcode is "50098765488"
barcode = "50098765488"

assert(checksum, "88")

---

# Nucleoid declares a carrier type with a constructor

# There is a Carrier type,
# which has a name as a string
class Carrier(name: str):
    this.name = name

# carrier1 is a Carrier whose name is "Fastline"
carrier1 = Carrier("Fastline")

assert(carrier1, { "id": "carrier1", "name": "Fastline" })
assert(Carrier.length, 1)

---

# Nucleoid declares a courier as a subtype of a carrier

# There is a Carrier type,
# which has a name as a string
class Carrier(name: str):
    this.name = name

# There is a Courier type,
# which is a subtype of Carrier
# and has a bike as a boolean
class Courier: Carrier
    def init(name, bike):
        super(name)
        this.name = name
        this.bike = bike

# courier1 is a Courier whose name is "Pedal" and whose bike is true
courier1 = Courier("Pedal", true)

assert(courier1, { "id": "courier1", "name": "Pedal", "bike": true })

---

# Nucleoid labels every consignment with a class-level rule

# There is a Consignment type,
# which has a number as a number
class Consignment(number: int):
    this.number = number

# Any consignment's label is "CN-" plus the consignment's number
$Consignment.label = "CN-" + $Consignment.number

# consignment1 is a Consignment whose number is 4
consignment1 = Consignment(4)

assert(consignment1.label, "CN-4")

# consignment2 is a Consignment whose number is 5
consignment2 = Consignment(5)

assert(consignment2.label, "CN-5")

---

# Nucleoid replaces a class-level rule on a dock

# There is a Dock type
class Dock:
    pass

# dock1 is a Dock whose bay is 3
dock1 = Dock()
dock1.bay = 3

# Any dock's sign is "D" plus the dock's bay
$Dock.sign = "D" + $Dock.bay

assert(dock1.sign, "D3")

# Any dock's sign is "DOCK-" plus the dock's bay
$Dock.sign = "DOCK-" + $Dock.bay

assert(dock1.sign, "DOCK-3")

# dock1's bay is 4
dock1.bay = 4

assert(dock1.sign, "DOCK-4")

---

# Nucleoid marks a pallet as heavy with a class-level conditional

# There is a Pallet type
class Pallet:
    pass

# Any pallet heavier than 500 is heavy
if $Pallet.mass > 500:
    $Pallet.heavy = true

# pallet1 is a Pallet whose mass is 300
pallet1 = Pallet()
pallet1.mass = 300

assert(pallet1.heavy, null)

# pallet1's mass is 700
pallet1.mass = 700

assert(pallet1.heavy, true)

---

# Nucleoid grades a delivery window with a class-level chain

# There is a Delivery type
class Delivery:
    pass

# fast is "FAST", normal is "NORMAL", and slow is "SLOW"
fast = "FAST"; normal = "NORMAL"; slow = "SLOW"

# If any delivery's days is less than 2, then the delivery's speed is fast,
# else if the delivery's days is less than 5, then the delivery's speed is normal,
# else the delivery's speed is slow
if $Delivery.days < 2:
    $Delivery.speed = fast
else if $Delivery.days < 5:
    $Delivery.speed = normal
else:
    $Delivery.speed = slow

# delivery1 is a Delivery whose days is 3
delivery1 = Delivery()
delivery1.days = 3

assert(delivery1.speed, "NORMAL")

# normal is "STANDARD"
normal = "STANDARD"

assert(delivery1.speed, "STANDARD")

# delivery1's days is 9
delivery1.days = 9

assert(delivery1.speed, "SLOW")

---

# Nucleoid counts the parcels of a depot with a class-level aggregate

# There is a Depot type
class Depot:
    pass

# There is a Parcel type
class Parcel:
    pass

# depot1 is a Depot
depot1 = Depot()

# parcel1 is a Parcel whose depot is depot1
parcel1 = Parcel()
parcel1.depot = depot1

# parcel2 is a Parcel whose depot is depot1
parcel2 = Parcel()
parcel2.depot = depot1

# Any depot's held is the number of parcels whose depot is the depot
$Depot.held = Parcel.filter(p => p.depot == $Depot).length

assert(depot1.held, 2)

# parcel3 is a Parcel whose depot is depot1
parcel3 = Parcel()
parcel3.depot = depot1

assert(depot1.held, 3)

---

# Nucleoid builds a waybill from a property that arrives later

# There is a Waybill type
class Waybill:
    pass

# waybill1 is a Waybill
waybill1 = Waybill()

# waybill1's full is "WB" plus waybill1's serial
waybill1.full = "WB" + waybill1.serial

assert(waybill1.full, null)

# waybill1's serial is "77120"
waybill1.serial = "77120"

assert(waybill1.full, "WB77120")

---

# Nucleoid reads a driver through a vehicle reference

# There is a Vehicle type
class Vehicle:
    pass

# There is a Driver type
class Driver:
    pass

# vehicle1 is a Vehicle
vehicle1 = Vehicle()

# driver1 is a Driver whose name is "Sam"
driver1 = Driver()
driver1.name = "Sam"

# vehicle1's driver is driver1
vehicle1.driver = driver1

# vehicle1's crew is "Driver: " plus vehicle1's driver's name
vehicle1.crew = "Driver: " + vehicle1.driver.name

assert(vehicle1.crew, "Driver: Sam")

# driver1's name is "Alex"
driver1.name = "Alex"

assert(vehicle1.crew, "Driver: Alex")

---

# Nucleoid clears a route summary when a leg is deleted

# There is a Route type
class Route:
    pass

# route1 is a Route
route1 = Route()

# route1's origin is "LIS"
route1.origin = "LIS"

# route1's target is "OPO"
route1.target = "OPO"

# route1's summary is route1's origin plus "-" plus route1's target
route1.summary = route1.origin + "-" + route1.target

assert(route1.summary, "LIS-OPO")

# route1's origin is deleted
delete route1.origin

assert(route1.summary, null)
assert(route1.target, "OPO")

---

# Nucleoid computes a load factor in a block

# gross is 900
gross = 900

# factor is null
factor = null

# while in the block, net is a local variable that is gross divided by 3,
# and factor is net times 2
{
    net = gross / 3
    factor = net * 2
}

assert(factor, 600)

# gross is 1200
gross = 1200

assert(factor, 800)

---

# Nucleoid computes a stack height in a nested block

# tiers is 4
tiers = 4

# while in the block, layer is a local variable that is tiers squared,
# and in a nested block, height is layer times 5
{
    layer = Math.pow(tiers, 2)
    {
        height = layer * 5
    }
}

assert(height, 80)

---

# Nucleoid raises an alert in a nested if inside a block

# stock is 20
stock = 20

# reorder is 50
reorder = 50

# alarm is true
alarm = true

# while in the block, gap is a local variable that is reorder minus stock,
# and if gap is greater than 10, then alert is alarm
{
    gap = reorder - stock
    if gap > 10:
        alert = alarm
}

assert(alert, true)

# alarm is false
alarm = false

assert(alert, false)

---

# Nucleoid picks a handling class in a nested else inside a block

# length is 300
length = 300

# girth is 100
girth = 100

# oversize is "OVERSIZE"
oversize = "OVERSIZE"

# standard is "STANDARD"
standard = "STANDARD"

# while in the block, total is a local variable that is length plus girth,
# and if total is greater than 500, then handling is oversize,
# else handling is standard
{
    total = length + girth
    if total > 500:
        handling = oversize
    else:
        handling = standard
}

assert(handling, "STANDARD")

# standard is "STD"
standard = "STD"

assert(handling, "STD")

---

# Nucleoid prices a container in a class-level block

# There is a Container type
class Container:
    pass

# while in the block, levy is a local variable that is any container's worth times 5 divided by 100,
# and the container's total is the container's worth plus levy
{
    levy = $Container.worth * 5 / 100
    $Container.total = $Container.worth + levy
}

# container1 is a Container
container1 = Container()

assert(container1.total, null)

# container1's worth is 200
container1.worth = 200

assert(container1.total, 210)

---

# Nucleoid computes a bay index in a nested class-level block

# There is a Bay type
class Bay:
    pass

# while in the block, span is a local variable that is 90 divided by any bay's rows,
# and in a nested block, the bay's index is the floor of span times the bay's columns
{
    span = 90 / $Bay.rows
    {
        $Bay.index = Math.floor(span * $Bay.columns)
    }
}

# bay1 is a Bay
bay1 = Bay()

# bay1's rows is 4
bay1.rows = 4

# bay1's columns is 7
bay1.columns = 7

assert(bay1.index, 157)

---

# Nucleoid flags a damaged crate with an if statement on a property

# There is a Crate type
class Crate:
    pass

# crate1 is a Crate whose state is "SEALED"
crate1 = Crate()
crate1.state = "SEALED"

# if crate1's state is "OPEN", then crate1's inspect is true
if crate1.state == "OPEN":
    crate1.inspect = true

assert(crate1.inspect, null)

# crate1's state is "OPEN"
crate1.state = "OPEN"

assert(crate1.inspect, true)

---

# Nucleoid chooses a packing note with an else statement on a property

# There is a Box type
class Box:
    pass

# box1 is a Box whose fragile is false
box1 = Box()
box1.fragile = false

# care is "HANDLE WITH CARE"
care = "HANDLE WITH CARE"

# plain is "STANDARD PACKING"
plain = "STANDARD PACKING"

# if box1's fragile is true, then box1's note is care,
# else box1's note is plain
if box1.fragile == true:
    box1.note = care
else:
    box1.note = plain

assert(box1.note, "STANDARD PACKING")

# box1's fragile is true
box1.fragile = true

assert(box1.note, "HANDLE WITH CARE")

---

# Nucleoid bands a haulage charge with multiple else if statements on a property

# There is a Haul type
class Haul:
    pass

# haul1 is a Haul whose miles is 40
haul1 = Haul()
haul1.miles = 40

# unit is 2
unit = 2

# if haul1's miles is greater than 200, then haul1's fee is haul1's miles times unit plus 50,
# else if haul1's miles is greater than 100, then haul1's fee is haul1's miles times unit plus 20,
# else haul1's fee is haul1's miles times unit
if haul1.miles > 200:
    haul1.fee = haul1.miles * unit + 50
else if haul1.miles > 100:
    haul1.fee = haul1.miles * unit + 20
else:
    haul1.fee = haul1.miles * unit

assert(haul1.fee, 80)

# haul1's miles is 150
haul1.miles = 150

assert(haul1.fee, 320)

# haul1's miles is 300
haul1.miles = 300

assert(haul1.fee, 650)

---

# Nucleoid calls a volume function in an assignment

# volume returns the product of three sides
def volume(a, b, c):
    return a * b * c

# width is 2
width = 2

# depth is 3
depth = 3

# height is 4
height = 4

# capacity is the result of the volume function call
capacity = volume(width, depth, height)

assert(capacity, 24)

# height is 5
height = 5

assert(capacity, 30)

---

# Nucleoid updates a duty when its function is redefined

# duty returns the amount times 10 divided by 100
def duty(amount):
    return amount * 10 / 100

# goods is 500
goods = 500

# payable is the result of the duty function call with goods
payable = duty(goods)

assert(payable, 50)

# duty returns the amount times 20 divided by 100
def duty(amount):
    return amount * 20 / 100

assert(payable, 100)

---

# Nucleoid nests a rate lookup inside a total function

# rate returns 3 times the zone
def rate(zone):
    return zone * 3

# total returns the weight times the rate of the zone
def total(weight, zone):
    return weight * rate(zone)

# parcelWeight is 4
parcelWeight = 4

# parcelZone is 2
parcelZone = 2

# cost is the result of the total function call
cost = total(parcelWeight, parcelZone)

assert(cost, 24)

---

# Nucleoid finds a shipment with the three lambda forms

# codes is a list of 11, 22 and 33
codes = [11, 22, 33]

assert(codes.find(function(code) { return code == 33 }), 33)
assert(codes.find(code => { return code == 22 }), 22)
assert(codes.find(code => code == 11), 11)

---

# Nucleoid filters a fleet by two thresholds

# There is a Truck type,
# which has an axles as a number
class Truck(axles: int):
    this.axles = axles

# There are Trucks whose axles are 2, 4 and 6
Truck(2); Truck(4); Truck(6)

# upper is 5
upper = 5

# lower is 3
lower = 3

# midsize is Trucks whose axles are above lower and below upper
midsize = Truck.filter(t => t.axles > lower).filter(t => t.axles < upper)

assert(midsize.length, 1)
assert(midsize[0].axles, 4)

# lower is 1
lower = 1

assert(midsize.length, 2)
assert(midsize[0].axles, 2)

---

# Nucleoid maps parcel weights into charges

# weights is a list of 2, 4 and 6
weights = [2, 4, 6]

# perKilo is 3
perKilo = 3

# charges is weights mapped to the weight times perKilo
charges = weights.map(w => w * perKilo)

assert(charges[0], 6)
assert(charges[2], 18)

# perKilo is 5
perKilo = 5

assert(charges[2], 30)

---

# Nucleoid reduces a manifest into a total weight

# loads is a list of 10, 20 and 30
loads = [10, 20, 30]

# gross is the sum of loads
gross = loads.reduce((sum, load) => sum + load, 0)

assert(gross, 60)

# Add 40 to loads
loads.push(40)

assert(gross, 100)

---

# Nucleoid checks whether every crate is sealed

# There is a Crate type
class Crate:
    pass

# crate1 is a Crate that is sealed
crate1 = Crate()
crate1.sealed = true

# crate2 is a Crate that is sealed
crate2 = Crate()
crate2.sealed = true

# ready is whether every crate is sealed
ready = Crate.every(c => c.sealed == true)

assert(ready, true)

# crate2 is not sealed
crate2.sealed = false

assert(ready, false)

---

# Nucleoid checks whether any depot is closed

# There is a Depot type
class Depot:
    pass

# depot1 is a Depot that is open
depot1 = Depot()
depot1.closed = false

# depot2 is a Depot that is open
depot2 = Depot()
depot2.closed = false

# disrupted is whether any depot is closed
disrupted = Depot.some(d => d.closed == true)

assert(disrupted, false)

# depot2 is closed
depot2.closed = true

assert(disrupted, true)

---

# Nucleoid joins a route into a single string

# stops is a list of "LIS", "MAD" and "PAR"
stops = ["LIS", "MAD", "PAR"]

# path is stops joined with "-"
path = stops.join("-")

assert(path, "LIS-MAD-PAR")

# Add "LON" to stops
stops.push("LON")

assert(path, "LIS-MAD-PAR-LON")

---

# Nucleoid creates a summary for every open order

# There is an Order type
class Order:
    pass

# order1 is an Order
order1 = Order()

# order2 is an Order that is cancelled
order2 = Order()
order2.cancelled = true

# order3 is an Order
order3 = Order()

# There is a Docket type,
# which has an order as an Order
class Docket(order):
    this.order = order

# Any docket's kind is "PICK"
$Docket.kind = "PICK"

# For each order of Order, if the order is not cancelled,
# then there is a Docket whose order is the order
for order of Order:
    if not order.cancelled:
        Docket(order)

assert(Docket.length, 2)
assert(Docket[0].order.id, "order1")
assert(Docket[1].order.id, "order3")
assert(Docket[0].kind, "PICK")

---

# Nucleoid rolls back a dispatch if a rule throws

# There is a Dispatch type
class Dispatch:
    pass

# If any dispatch's weight is greater than 1000, then throw 'OVERWEIGHT'
if $Dispatch.weight > 1000:
    throw 'OVERWEIGHT'

# dispatch1 is a Dispatch
dispatch1 = Dispatch()

try:
    # dispatch1's weight is 1500
    dispatch1.weight = 1500
catch error:
    assert(error, "OVERWEIGHT")

assert(dispatch1.weight, null)

---

# Nucleoid refuses a cycle between a load and a total

# load is 40
load = 40

# total is load times 2
total = load * 2

assert(total, 80)

try:
    # load is total times 2
    load = total * 2
catch error:
    assert(error, TypeError("Circular Dependency"))

---

# Nucleoid validates a postcode with a regular expression

# There is an Address type
class Address:
    pass

# If any address's postcode does not match /[0-9]{4}/, then throw 'INVALID_POSTCODE'
if not /[0-9]{4}/.test($Address.postcode):
    throw 'INVALID_POSTCODE'

# address1 is an Address
address1 = Address()

assert(address1.postcode, null)

try:
    # address1's postcode is '12'
    address1.postcode = '12'
catch error:
    assert(error, "INVALID_POSTCODE")
```
