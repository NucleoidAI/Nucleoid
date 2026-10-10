# Nucleoid Language Reference - Synthesized Use Cases 21

```

# Nucleoid converts a flight distance through a chain of variables

# metres is 900000
metres = 900000

# kilometres is metres divided by 1000
kilometres = metres / 1000

# sectors is kilometres divided by 300
sectors = kilometres / 300

assert(kilometres, 900)
assert(sectors, 3)

# metres is 1800000
metres = 1800000

assert(kilometres, 1800)
assert(sectors, 6)

---

# Nucleoid replaces the source of a seat price

# flexible is 400
flexible = 400

# saver is 200
saver = 200

# price is flexible divided by 2
price = flexible / 2

assert(price, 200)

# price is saver divided by 2
price = saver / 2

assert(price, 100)

# saver is 300
saver = 300

assert(price, 150)

---

# Nucleoid raises a fleet count using its own value

# aircraft is 40
aircraft = 40

# aircraft is aircraft plus 10
aircraft = aircraft + 10

assert(aircraft, 50)

---

# Nucleoid fixes a landing fee with the value property

# fee is 60
fee = 60

# movements is 5
movements = 5

# quoted is fee's value times movements
quoted = fee.value * movements

assert(quoted, 300)

# fee is 80
fee = 80

assert(quoted, 300)

# movements is 10
movements = 10

assert(quoted, 600)

---

# Nucleoid clears a payload when its weight is deleted

# weight is 9000
weight = 9000

# fuel is weight divided by 9
fuel = weight / 9

# payload is weight minus fuel
payload = weight - fuel

assert(payload, 8000)

# weight is deleted
delete weight

assert(fuel, null)
assert(payload, null)

try:
    # weight
    weight
catch error:
    assert(error, ReferenceError("weight is not defined"))

---

# Nucleoid assigns a delay comparison to a variable

# actual is 40
actual = 40

# scheduled is 15
scheduled = 15

# delayed is whether actual is greater than scheduled
delayed = actual > scheduled

assert(delayed, true)

# actual is 10
actual = 10

assert(delayed, false)

---

# Nucleoid builds a flight number from a template literal

# carrier is "TP"
carrier = "TP"

# number is 1234
number = 1234

# flight is the carrier and number in a template
flight = `${carrier}${number}`

assert(flight, "TP1234")

# number is 1235
number = 1235

assert(flight, "TP1235")

---

# Nucleoid combines clearance flags with logical operators

# boarded is true
boarded = true

# grounded is false
grounded = false

assert(boarded and not grounded, true)
assert(boarded && !grounded, true)
assert(grounded or boarded, true)
assert(grounded || boarded, true)

---

# Nucleoid respects parentheses in a loading formula

# crew is 2
crew = 2

# rows is 3
rows = 3

# seats is 4
seats = 4

assert(crew + rows * seats, 14)
assert((crew + rows) * seats, 20)

---

# Nucleoid boards passengers with the remainder operator

# passengers is 187
passengers = 187

# perRow is 6
perRow = 6

# spare is passengers modulo perRow
spare = passengers % perRow

assert(spare, 1)

# passengers is 186
passengers = 186

assert(spare, 0)

---

# Nucleoid appends a stand number to a gate code

# gate is "GT-"
gate = "GT-"

# stand is 22
stand = 22

# code is gate plus stand
code = gate + stand

assert(code, "GT-22")

# stand is 23
stand = 23

assert(code, "GT-23")

---

# Nucleoid counts the characters of a registration

# registration is "CSTV"
registration = "CSTV"

# width is registration's length
width = registration.length

assert(width, 4)

# registration is "CSTVA12"
registration = "CSTVA12"

assert(width, 7)

---

# Nucleoid lowercases an airport name as a dependency

# airport is "LISBON"
airport = "LISBON"

# key is airport lowercased
key = airport.lower()

assert(key, "lisbon")

# airport is "PORTO"
airport = "PORTO"

assert(key, "porto")

---

# Nucleoid reads the terminal letter of a gate code

# gate is "A24"
gate = "A24"

# terminal is the character of gate at 0
terminal = gate.charAt(0)

assert(terminal, "A")

# gate is "B24"
gate = "B24"

assert(terminal, "B")

---

# Nucleoid rewrites a route path with replace

# route is "leg/old/waypoint"
route = "leg/old/waypoint"

# revision is "new"
revision = "new"

# current is route with "old" replaced by revision
current = route.replace("old", revision)

assert(current, "leg/new/waypoint")

# revision is "alt"
revision = "alt"

assert(current, "leg/alt/waypoint")

---

# Nucleoid takes the time from the end of a slot code

# slot is "SLT-0645"
slot = "SLT-0645"

# time is the last four characters of slot
time = slot[-4:]

assert(time, "0645")

# slot is "SLT-0745"
slot = "SLT-0745"

assert(time, "0745")

---

# Nucleoid declares an aircraft type with a constructor

# There is an Aircraft type,
# which has a registration as a string
class Aircraft(registration: str):
    this.registration = registration

# aircraft1 is an Aircraft whose registration is "CS-TVA"
aircraft1 = Aircraft("CS-TVA")

assert(aircraft1, { "id": "aircraft1", "registration": "CS-TVA" })
assert(Aircraft.length, 1)

---

# Nucleoid declares a freighter as a subtype of an aircraft

# There is an Aircraft type,
# which has a registration as a string
class Aircraft(registration: str):
    this.registration = registration

# There is a Freighter type,
# which is a subtype of Aircraft
# and has a pallets as a number
class Freighter: Aircraft
    def init(registration, pallets):
        super(registration)
        this.registration = registration
        this.pallets = pallets

# freighter1 is a Freighter whose registration is "CS-TFR" and whose pallets is 18
freighter1 = Freighter("CS-TFR", 18)

assert(freighter1, { "id": "freighter1", "registration": "CS-TFR", "pallets": 18 })

---

# Nucleoid numbers every sector with a class-level rule

# There is a Sector type,
# which has a number as a number
class Sector(number: int):
    this.number = number

# Any sector's reference is "SEC-" plus the sector's number
$Sector.reference = "SEC-" + $Sector.number

# sector1 is a Sector whose number is 2
sector1 = Sector(2)

assert(sector1.reference, "SEC-2")

# sector2 is a Sector whose number is 3
sector2 = Sector(3)

assert(sector2.reference, "SEC-3")

---

# Nucleoid replaces a class-level rule on a stand

# There is a Stand type
class Stand:
    pass

# stand1 is a Stand whose number is 11
stand1 = Stand()
stand1.number = 11

# Any stand's sign is "S" plus the stand's number
$Stand.sign = "S" + $Stand.number

assert(stand1.sign, "S11")

# Any stand's sign is "STAND-" plus the stand's number
$Stand.sign = "STAND-" + $Stand.number

assert(stand1.sign, "STAND-11")

# stand1's number is 12
stand1.number = 12

assert(stand1.sign, "STAND-12")

---

# Nucleoid marks a long haul with a class-level conditional

# There is a Flight type
class Flight:
    pass

# Any flight longer than 6 hours is long haul
if $Flight.hours > 6:
    $Flight.longHaul = true

# flight1 is a Flight whose hours is 2
flight1 = Flight()
flight1.hours = 2

assert(flight1.longHaul, null)

# flight1's hours is 11
flight1.hours = 11

assert(flight1.longHaul, true)

---

# Nucleoid classifies a delay with a class-level chain

# There is a Delay type
class Delay:
    pass

# major is "MAJOR", minor is "MINOR", and slight is "SLIGHT"
major = "MAJOR"; minor = "MINOR"; slight = "SLIGHT"

# If any delay's minutes is greater than 180, then the delay's grade is major,
# else if the delay's minutes is greater than 15, then the delay's grade is minor,
# else the delay's grade is slight
if $Delay.minutes > 180:
    $Delay.grade = major
else if $Delay.minutes > 15:
    $Delay.grade = minor
else:
    $Delay.grade = slight

# delay1 is a Delay whose minutes is 60
delay1 = Delay()
delay1.minutes = 60

assert(delay1.grade, "MINOR")

# minor is "MODERATE"
minor = "MODERATE"

assert(delay1.grade, "MODERATE")

# delay1's minutes is 240
delay1.minutes = 240

assert(delay1.grade, "MAJOR")

---

# Nucleoid counts the flights of an aircraft with a class-level aggregate

# There is an Aircraft type
class Aircraft:
    pass

# There is a Flight type
class Flight:
    pass

# aircraft1 is an Aircraft
aircraft1 = Aircraft()

# flight1 is a Flight whose aircraft is aircraft1
flight1 = Flight()
flight1.aircraft = aircraft1

# flight2 is a Flight whose aircraft is aircraft1
flight2 = Flight()
flight2.aircraft = aircraft1

# Any aircraft's rotations is the number of flights whose aircraft is the aircraft
$Aircraft.rotations = Flight.filter(f => f.aircraft == $Aircraft).length

assert(aircraft1.rotations, 2)

# flight3 is a Flight whose aircraft is aircraft1
flight3 = Flight()
flight3.aircraft = aircraft1

assert(aircraft1.rotations, 3)

---

# Nucleoid builds a booking from a property that arrives later

# There is a Booking type
class Booking:
    pass

# booking1 is a Booking
booking1 = Booking()

# booking1's full is "PNR" plus booking1's serial
booking1.full = "PNR" + booking1.serial

assert(booking1.full, null)

# booking1's serial is "8823"
booking1.serial = "8823"

assert(booking1.full, "PNR8823")

---

# Nucleoid reads a commander through a flight reference

# There is a Flight type
class Flight:
    pass

# There is a Pilot type
class Pilot:
    pass

# flight1 is a Flight
flight1 = Flight()

# pilot1 is a Pilot whose name is "Costa"
pilot1 = Pilot()
pilot1.name = "Costa"

# flight1's pilot is pilot1
flight1.pilot = pilot1

# flight1's commanded is "Commander " plus flight1's pilot's name
flight1.commanded = "Commander " + flight1.pilot.name

assert(flight1.commanded, "Commander Costa")

# pilot1's name is "Costa Dias"
pilot1.name = "Costa Dias"

assert(flight1.commanded, "Commander Costa Dias")

---

# Nucleoid clears a board entry when a gate is deleted

# There is a Board type
class Board:
    pass

# board1 is a Board
board1 = Board()

# board1's gate is "A12"
board1.gate = "A12"

# board1's destination is "OPO"
board1.destination = "OPO"

# board1's entry is board1's gate plus " " plus board1's destination
board1.entry = board1.gate + " " + board1.destination

assert(board1.entry, "A12 OPO")

# board1's gate is deleted
delete board1.gate

assert(board1.entry, null)
assert(board1.destination, "OPO")

---

# Nucleoid computes a block time in a block

# minutes is 1440
minutes = 1440

# block is null
block = null

# while in the block, hourly is a local variable that is minutes divided by 60,
# and block is hourly times 60
{
    hourly = minutes / 60
    block = hourly * 60
}

assert(block, 1440)

# minutes is 2880
minutes = 2880

assert(block, 2880)

---

# Nucleoid computes an apron area in a nested block

# side is 30
side = 30

# while in the block, face is a local variable that is side squared,
# and in a nested block, area is face times 2
{
    face = Math.pow(side, 2)
    {
        area = face * 2
    }
}

assert(area, 1800)

---

# Nucleoid raises a load alert in a nested if inside a block

# booked is 180
booked = 180

# seats is 150
seats = 150

# notify is true
notify = true

# while in the block, excess is a local variable that is booked minus seats,
# and if excess is greater than 20, then alert is notify
{
    excess = booked - seats
    if excess > 20:
        alert = notify
}

assert(alert, true)

# notify is false
notify = false

assert(alert, false)

---

# Nucleoid picks a de-icing plan in a nested else inside a block

# temperature is 4
temperature = 4

# moisture is 3
moisture = 3

# full is "FULL DE-ICE"
full = "FULL DE-ICE"

# spot is "SPOT CHECK"
spot = "SPOT CHECK"

# while in the block, risk is a local variable that is temperature times moisture,
# and if risk is greater than 20, then plan is full,
# else plan is spot
{
    risk = temperature * moisture
    if risk > 20:
        plan = full
    else:
        plan = spot
}

assert(plan, "SPOT CHECK")

# spot is "SPOT"
spot = "SPOT"

assert(plan, "SPOT")

---

# Nucleoid computes a ticket total in a class-level block

# There is a Fare type
class Fare:
    pass

# while in the block, tax is a local variable that is any fare's net times 20 divided by 100,
# and the fare's gross is the fare's net plus tax
{
    tax = $Fare.net * 20 / 100
    $Fare.gross = $Fare.net + tax
}

# fare1 is a Fare
fare1 = Fare()

assert(fare1.gross, null)

# fare1's net is 250
fare1.net = 250

assert(fare1.gross, 300)

---

# Nucleoid computes a stand index in a nested class-level block

# There is a Pier type
class Pier:
    pass

# while in the block, share is a local variable that is 600 divided by any pier's stands,
# and in a nested block, the pier's index is the floor of share times the pier's gates
{
    share = 600 / $Pier.stands
    {
        $Pier.index = Math.floor(share * $Pier.gates)
    }
}

# pier1 is a Pier
pier1 = Pier()

# pier1's stands is 7
pier1.stands = 7

# pier1's gates is 4
pier1.gates = 4

assert(pier1.index, 342)

---

# Nucleoid flags a diversion with an if statement on a property

# There is an Approach type
class Approach:
    pass

# approach1 is an Approach whose state is "CLEAR"
approach1 = Approach()
approach1.state = "CLEAR"

# if approach1's state is "FOGGED", then approach1's divert is true
if approach1.state == "FOGGED":
    approach1.divert = true

assert(approach1.divert, null)

# approach1's state is "FOGGED"
approach1.state = "FOGGED"

assert(approach1.divert, true)

---

# Nucleoid chooses a boarding note with an else statement on a property

# There is a Gate type
class Gate:
    pass

# gate1 is a Gate whose remote is false
gate1 = Gate()
gate1.remote = false

# bussed is "BUS BOARDING"
bussed = "BUS BOARDING"

# bridged is "AIRBRIDGE"
bridged = "AIRBRIDGE"

# if gate1's remote is true, then gate1's note is bussed,
# else gate1's note is bridged
if gate1.remote == true:
    gate1.note = bussed
else:
    gate1.note = bridged

assert(gate1.note, "AIRBRIDGE")

# gate1's remote is true
gate1.remote = true

assert(gate1.note, "BUS BOARDING")

---

# Nucleoid bands a compensation with multiple else if statements on a property

# There is a Claim type
class Claim:
    pass

# claim1 is a Claim whose hours is 2
claim1 = Claim()
claim1.hours = 2

# unit is 50
unit = 50

# if claim1's hours is greater than 4, then claim1's award is claim1's hours times unit plus 400,
# else if claim1's hours is greater than 3, then claim1's award is claim1's hours times unit plus 200,
# else claim1's award is claim1's hours times unit
if claim1.hours > 4:
    claim1.award = claim1.hours * unit + 400
else if claim1.hours > 3:
    claim1.award = claim1.hours * unit + 200
else:
    claim1.award = claim1.hours * unit

assert(claim1.award, 100)

# claim1's hours is 4
claim1.hours = 4

assert(claim1.award, 400)

# claim1's hours is 6
claim1.hours = 6

assert(claim1.award, 700)

---

# Nucleoid calls a burn function in an assignment

# burn returns the fuel divided by the hours
def burn(fuel, hours):
    return fuel / hours

# uplift is 9000
uplift = 9000

# duration is 3
duration = 3

# hourly is the result of the burn function call
hourly = burn(uplift, duration)

assert(hourly, 3000)

# duration is 6
duration = 6

assert(hourly, 1500)

---

# Nucleoid updates a levy when its function is redefined

# levy returns the fare times 5 divided by 100
def levy(fare):
    return fare * 5 / 100

# ticket is 400
ticket = 400

# duty is the result of the levy function call with ticket
duty = levy(ticket)

assert(duty, 20)

# levy returns the fare times 10 divided by 100
def levy(fare):
    return fare * 10 / 100

assert(duty, 40)

---

# Nucleoid nests a band lookup inside a charge function

# band returns 6 times the grade
def band(grade):
    return grade * 6

# charge returns the tonnes times the band of the grade
def charge(tonnes, grade):
    return tonnes * band(grade)

# mass is 5
mass = 5

# noiseGrade is 2
noiseGrade = 2

# payable is the result of the charge function call
payable = charge(mass, noiseGrade)

assert(payable, 60)

---

# Nucleoid finds a gate with the three lambda forms

# gates is a list of 4, 12 and 24
gates = [4, 12, 24]

assert(gates.find(function(gate) { return gate == 24 }), 24)
assert(gates.find(gate => { return gate == 12 }), 12)
assert(gates.find(gate => gate == 4), 4)

---

# Nucleoid filters a fleet by two thresholds

# There is a Jet type,
# which has a range as a number
class Jet(range: int):
    this.range = range

# There are Jets whose ranges are 2000, 6000 and 12000
Jet(2000); Jet(6000); Jet(12000)

# ceiling is 9000
ceiling = 9000

# floorRange is 4000
floorRange = 4000

# midrange is Jets whose range is above floorRange and below ceiling
midrange = Jet.filter(j => j.range > floorRange).filter(j => j.range < ceiling)

assert(midrange.length, 1)
assert(midrange[0].range, 6000)

# floorRange is 1000
floorRange = 1000

assert(midrange.length, 2)
assert(midrange[0].range, 2000)

---

# Nucleoid maps sectors into block hours

# sectors is a list of 2, 4 and 6
sectors = [2, 4, 6]

# perSector is 3
perSector = 3

# hours is sectors mapped to the sector times perSector
hours = sectors.map(s => s * perSector)

assert(hours[0], 6)
assert(hours[2], 18)

# perSector is 5
perSector = 5

assert(hours[2], 30)

---

# Nucleoid reduces a set of sectors into a total

# legs is a list of 400, 500 and 600
legs = [400, 500, 600]

# total is the sum of legs
total = legs.reduce((sum, leg) => sum + leg, 0)

assert(total, 1500)

# Add 500 to legs
legs.push(500)

assert(total, 2000)

---

# Nucleoid checks whether every aircraft is airworthy

# There is an Aircraft type
class Aircraft:
    pass

# aircraft1 is an Aircraft that is airworthy
aircraft1 = Aircraft()
aircraft1.airworthy = true

# aircraft2 is an Aircraft that is airworthy
aircraft2 = Aircraft()
aircraft2.airworthy = true

# ready is whether every aircraft is airworthy
ready = Aircraft.every(a => a.airworthy == true)

assert(ready, true)

# aircraft2 is not airworthy
aircraft2.airworthy = false

assert(ready, false)

---

# Nucleoid checks whether any flight is cancelled

# There is a Flight type
class Flight:
    pass

# flight1 is a Flight that is operating
flight1 = Flight()
flight1.cancelled = false

# flight2 is a Flight that is operating
flight2 = Flight()
flight2.cancelled = false

# rebook is whether any flight is cancelled
rebook = Flight.some(f => f.cancelled == true)

assert(rebook, false)

# flight2 is cancelled
flight2.cancelled = true

assert(rebook, true)

---

# Nucleoid joins a routing into a single string

# waypoints is a list of "LIS", "MAD" and "CDG"
waypoints = ["LIS", "MAD", "CDG"]

# routing is waypoints joined with "/"
routing = waypoints.join("/")

assert(routing, "LIS/MAD/CDG")

# Add "AMS" to waypoints
waypoints.push("AMS")

assert(routing, "LIS/MAD/CDG/AMS")

---

# Nucleoid creates a load sheet for every operating flight

# There is a Flight type
class Flight:
    pass

# flight1 is a Flight
flight1 = Flight()

# flight2 is a Flight that is cancelled
flight2 = Flight()
flight2.cancelled = true

# flight3 is a Flight
flight3 = Flight()

# There is a Sheet type,
# which has a flight as a Flight
class Sheet(flight):
    this.flight = flight

# Any sheet's kind is "LOAD"
$Sheet.kind = "LOAD"

# For each flight of Flight, if the flight is not cancelled,
# then there is a Sheet whose flight is the flight
for flight of Flight:
    if not flight.cancelled:
        Sheet(flight)

assert(Sheet.length, 2)
assert(Sheet[0].flight.id, "flight1")
assert(Sheet[1].flight.id, "flight3")
assert(Sheet[0].kind, "LOAD")

---

# Nucleoid rolls back a loading if a rule throws

# There is a Loading type
class Loading:
    pass

# If any loading's kilograms is greater than 20000, then throw 'OVER_PAYLOAD'
if $Loading.kilograms > 20000:
    throw 'OVER_PAYLOAD'

# loading1 is a Loading
loading1 = Loading()

try:
    # loading1's kilograms is 30000
    loading1.kilograms = 30000
catch error:
    assert(error, "OVER_PAYLOAD")

assert(loading1.kilograms, null)

---

# Nucleoid refuses a cycle between a fare and a revenue

# fare is 90
fare = 90

# revenue is fare times 150
revenue = fare * 150

assert(revenue, 13500)

try:
    # fare is revenue times 150
    fare = revenue * 150
catch error:
    assert(error, TypeError("Circular Dependency"))

---

# Nucleoid validates a registration with a regular expression

# There is a Record type
class Record:
    pass

# If any record's registration does not match /[A-Z]{2}-[A-Z]{3}/, then throw 'INVALID_REGISTRATION'
if not /[A-Z]{2}-[A-Z]{3}/.test($Record.registration):
    throw 'INVALID_REGISTRATION'

# record1 is a Record
record1 = Record()

assert(record1.registration, null)

try:
    # record1's registration is 'CS'
    record1.registration = 'CS'
catch error:
    assert(error, "INVALID_REGISTRATION")

---

# Nucleoid keeps a quoted fare fixed inside lists and labels

# fare and movements are independent inputs
fare = 60
movements = 5

# The quote fixes the fare but follows the number of movements
quote = [fare.value, movements]
label = `${fare.value}-${movements}`
carrier = "TAP"
start = 1
code = carrier[start.value:]

fare = 80
movements = 10
start = 2
carrier = "FLY"

assert(quote, [60, 10])
assert(label, "60-10")
assert(code, "LY")

---

# Nucleoid refreshes inherited aircraft status without replacing subtype rules

class Aircraft:
    pass

$Aircraft.clearance = "STANDARD"

class CargoAircraft: Aircraft
    pass

class RestrictedCargo: CargoAircraft
    pass

$RestrictedCargo.clearance = "SPECIAL"
cargo = CargoAircraft()
restricted = RestrictedCargo()

# A new fleet rule reaches existing cargo aircraft but not subtype overrides
$Aircraft.clearance = "EXPRESS"

assert(cargo.clearance, "EXPRESS")
assert(restricted.clearance, "SPECIAL")
assert(Aircraft.length, 0)
assert(CargoAircraft.length, 1)

---

# Nucleoid calls each freight constructor through the correct supertype

class Consignment(weight):
    this.weight = weight

class InsuredConsignment: Consignment
    def init(weight):
        super(weight + 10)

class ExpressConsignment: InsuredConsignment
    def init(weight):
        super(weight * 2)

# Express packaging doubles the weight before insurance adds ten
shipment = ExpressConsignment(10)

assert(shipment.weight, 30)

---

# Nucleoid rejects an aircraft inheritance loop

class Aircraft:
    pass

class Freighter: Aircraft
    pass

try:
    # Aircraft cannot inherit from its own subtype
    class Aircraft: Freighter
        pass
catch error:
    assert(error, TypeError("Circular Inheritance"))

aircraft = Aircraft()

assert(aircraft.id, "aircraft")
assert(Class.length, 2)

---

# Nucleoid rejects a cyclic boarding group list

class Boarding:
    pass

$Boarding.group = [$Boarding.code]

try:
    # A boarding code cannot depend on the group that already depends on it
    $Boarding.code = $Boarding.group[0]
catch error:
    assert(error, TypeError("Circular Dependency"))

$Boarding.code = "A"
boarding = Boarding()

assert(boarding.group, ["A"])

---

# Nucleoid validates departure window bounds before there are flights

class Departure:
    pass

try:
    # The lower bound must exist when this rule is declared
    $Departure.window = "ABCDEFG"[start:]
catch error:
    assert(error, ReferenceError("start is not defined"))

start = 1
end = 3
$Departure.window = "ABCDEFG"[start:end]
departure = Departure()

assert(departure.window, "BC")

end = 4

assert(departure.window, "BCD")

---

# Nucleoid keeps indexed shipment weights null until supplied

class Freight:
    pass

shipment = Freight()
shipment.weight = null
doubled = shipment["weight"] * 2

assert(doubled, null)

shipment.weight = 1200

assert(doubled, 2400)

delete shipment.weight

assert(doubled, null)

---

# Nucleoid updates the count of declared logistics types

kinds = Class.length

class Route:
    pass

assert(kinds, 1)

class Cargo: Route
    pass

assert(kinds, 2)

class Route:
    pass

assert(kinds, 2)

try:
    class Temporary:
        pass
    throw "NO_IMPORT"
catch error:
    assert(error, "NO_IMPORT")

assert(kinds, 2)
assert(Class.length, 2)

---

# Nucleoid keeps boarding policies in declaration order after selective updates

gate = 0
cleared = false
priority = 0

if cleared and gate > 0:
    priority = 1

if gate > 0:
    priority = 2

cleared = true
gate = 1

assert(priority, 2)

if cleared and gate > 0:
    priority = 3

gate = 2

assert(priority, 3)

---

# Nucleoid keeps a multi-stage flight budget current after redeclaration

fuel = 100
reserve = fuel + 20
contingency = reserve / 10
budget = fuel + contingency

reserve = fuel + 30

assert(budget, 113)

fuel = 200

assert(reserve, 230)
assert(contingency, 23)
assert(budget, 223)

---

# Nucleoid validates cargo capacity only after derived values settle

class Cargo(weight):
    this.weight = weight

$Cargo.capacity = $Cargo.weight * 2
$Cargo.spare = $Cargo.capacity - $Cargo.weight

if $Cargo.spare < 0:
    throw "INVALID_CAPACITY"

shipment = Cargo(100)

$Cargo.capacity = $Cargo.weight * 3

shipment.weight = 500

assert(shipment.capacity, 1500)
assert(shipment.spare, 1000)

---

# Nucleoid settles interacting handover rules and rolls back a rejected handover

tower = 0
approach = 0
total = tower + approach

if tower > 0 and tower < 4:
    approach = tower + 1

if approach > 0 and approach < 4:
    tower = approach + 1

tower = 1

assert(tower, 3)
assert(approach, 4)
assert(total, 7)

if approach == 3:
    throw "INVALID_HANDOVER"

try:
    tower = 2
catch error:
    assert(error, "INVALID_HANDOVER")

assert(tower, 3)
assert(approach, 4)
assert(total, 7)

if approach == 3:
    pass

tower = 2

assert(tower, 4)
assert(approach, 3)

---

# Nucleoid stops repeating unchanged clearance and sensor readings

cleared = 0
ready = 0

if cleared > 0:
    ready = 1

if ready > 0:
    cleared = 1

cleared = 1

assert(cleared, 1)
assert(ready, 1)

attempts = 0
readings = [0]

if attempts > 0 and attempts < 4:
    readings = [Number.NaN]

if Number.isNaN(readings[0]):
    attempts = attempts + 1

attempts = 1

assert(attempts, 2)
assert(Number.isNaN(readings[0]), true)

---

# Nucleoid propagates the sign of a zero flight correction

descending = false
correction = 0.0
inverse = 1 / correction

if descending:
    correction = -0.0
else:
    correction = 0.0

assert(inverse, Number.POSITIVE_INFINITY)

descending = true

assert(inverse, Number.NEGATIVE_INFINITY)

descending = false

assert(inverse, Number.POSITIVE_INFINITY)

---

# Nucleoid does not turn a conditional dispatch loop into a standing rule

class Flight:
    pass

flight = Flight()
flight.delay = 0
threshold = 0

for current of Flight:
    if threshold > 0:
        current.delay = threshold

threshold = 10

assert(flight.delay, 0)

for current of Flight:
    if threshold > 0:
        current.delay = threshold

assert(flight.delay, 10)

threshold = 20

assert(flight.delay, 10)

---

# Nucleoid cancels fuel inspections queued by an aborted dispatch

dispatch = 0
fuel = 100
inspections = 0

if fuel > 0:
    inspections = inspections + 1

if dispatch > 0:
    try:
        fuel = 200
        throw "NO_DISPATCH"
    catch error:
        if error != "NO_DISPATCH":
            throw error

estimate = dispatch + 10

dispatch = 1

assert(fuel, 100)
assert(inspections, 1)
assert(estimate, 11)

fuel = 150

assert(inspections, 2)

---

# Nucleoid preserves an earlier pressure inspection across nested aborts

stage = 0
pressure = 0
inspections = 0

if pressure > 0:
    inspections = inspections + 1

if stage > 0:
    pressure = 1
    try:
        pressure = 2
        try:
            pressure = 3
            throw "INNER_ABORT"
        catch inner:
            if inner != "INNER_ABORT":
                throw inner
        throw "OUTER_ABORT"
    catch outer:
        if outer != "OUTER_ABORT":
            throw outer

stage = 1

assert(pressure, 1)
assert(inspections, 1)

---

# Nucleoid clears aborted cargo deletion markers before recovery

dispatch = 0
absence = null

if dispatch > 0:
    try:
        temporaryCargo = 1
        delete temporaryCargo
        throw "NO_CARGO"
    catch error:
        if error != "NO_CARGO":
            throw error
        try:
            temporaryCargo
        catch missing:
            absence = missing

dispatch = 1

assert(absence, ReferenceError("temporaryCargo is not defined"))

---

# Nucleoid rejects a removed seat count after its deletion has propagated

departure = 0
seats = 1
total = seats + 1

if departure > 0:
    delete seats

def inspect_after_departure():
    departure = 1
    try:
        seats
    catch error:
        return error
    return "NO_ERROR"

assert(inspect_after_departure(), ReferenceError("seats is not defined"))
assert(total, null)

seats = 3

assert(total, 4)

---

# Nucleoid forwards a live landing levy through a function reference

# levy calculates a fee from the number of tonnes
def levy(tonnes):
    return tonnes * 5

# select_levy returns the current levy function
def select_levy():
    return levy

quoted = select_levy()
charge = quoted(20)

assert(charge, 100)

# A revised levy updates the forwarded quote
def levy(tonnes):
    return tonnes * 6

assert(charge, 120)

---

# Nucleoid updates named fuel callbacks over a list

loads = [10, 20, 30]

# reserve adds five units to each load
def reserve(load):
    return load + 5

planned = loads.map(reserve)

assert(planned, [15, 25, 35])

# A revised reserve policy updates the whole plan
def reserve(load):
    return load + 10

assert(planned, [20, 30, 40])

---

# Nucleoid tracks a pricing function passed to a fare calculator

def levy(fare):
    return fare * 2

def quote(policy, fare):
    return policy(fare)

fare = 100
total = quote(levy, fare)

assert(total, 200)

def levy(fare):
    return fare * 3

assert(total, 300)

fare = 120

assert(total, 360)

---

# Nucleoid updates a callable baggage allowance variable

bags = 2
allowance = count => count * 20
kilograms = allowance(bags)

assert(kilograms, 40)

# The allowance is replaced by a different lambda
allowance = count => count * 25

assert(kilograms, 50)

bags = 3

assert(kilograms, 75)

---

# Nucleoid tracks dotted and indexed cargo tariff calls

class Tariff:
    pass

tariff = Tariff()
dotted = tariff.calculate(4)
indexed = tariff["calculate"](4)

assert(dotted, null)
assert(indexed, null)

tariff.calculate = tonnes => tonnes * 10

assert(dotted, 40)
assert(indexed, 40)

tariff.calculate = tonnes => tonnes * 15

assert(dotted, 60)
assert(indexed, 60)

---

# Nucleoid freezes a fuel formula while keeping its price dependency live

price = 2

def cost(litres):
    return litres * price

formula = cost.value
quoted = formula(100)
current = cost(100)

def cost(litres):
    return litres * price + 50

assert(quoted, 200)
assert(current, 250)

price = 3

assert(quoted, 300)
assert(current, 350)

---

# Nucleoid defers and restores a nullable boarding policy

policy = null
priority = 0
group = policy(3) + priority

assert(group, null)

policy = seats => seats + 1

assert(group, 4)

delete policy

assert(group, null)

priority = 1

assert(group, null)

policy = seats => seats + 2

assert(group, 6)

---

# Nucleoid preserves a frozen airport fee after deleting its callable property

class Airport:
    pass

airport = Airport()
airport.fee = movements => movements * 10
quotedFee = airport.fee.value
quoted = quotedFee(3)
dotted = airport.fee(3)
indexed = airport["fee"](3)

airport.fee = null

assert(dotted, null)
assert(indexed, null)
assert(quoted, 30)

airport.fee = movements => movements * 20

assert(dotted, 60)
assert(indexed, 60)

delete airport.fee

assert(dotted, null)
assert(indexed, null)

airport.fee = movements => movements * 30

assert(dotted, 90)
assert(indexed, 90)
assert(quoted, 30)

---

# Nucleoid lets an absent local fee policy shadow a fleet function

def policy(weight):
    return 999

def estimate(policy, weight):
    return policy(weight)

localPolicy = null
charge = estimate(localPolicy, 10)

assert(charge, null)

localPolicy = weight => weight * 5

assert(charge, 50)

failure = null

try:
    estimate(7, 10)
catch error:
    failure = error

assert(failure, TypeError("policy is not a function"))
assert(policy(10), 999)

---

# Nucleoid rolls back a non callable replacement of an active dispatch policy

policy = delay => delay + 5
departure = policy(10)
failure = null

try:
    policy = 7
catch error:
    failure = error

assert(failure, TypeError("policy is not a function"))
assert(departure, 15)
assert(policy(20), 25)

---

# Nucleoid restores a fuel callback rejected during redefinition

def reserve(load):
    return load + 10

loads = [20, 30]
planned = loads.map(reserve)
failure = null

try:
    def reserve(load):
        if load == 30:
            throw "INVALID_RESERVE"
        return load + 20
catch error:
    failure = error

assert(failure, "INVALID_RESERVE")
assert(planned, [30, 40])
assert(reserve(40), 50)

def reserve(load):
    return load + 30

assert(planned, [50, 60])

---

# Nucleoid replaces a named levy with a callable variable and back again

def levy(tonnes):
    return tonnes * 5

policy = levy
direct = levy(10)
forwarded = policy(10)

# A lambda assigned to the same name replaces the named definition
levy = tonnes => tonnes * 6

assert(direct, 60)
assert(forwarded, 60)

# A new named definition replaces the callable variable
def levy(tonnes):
    return tonnes * 7

assert(direct, 70)
assert(forwarded, 70)

levy = null

assert(direct, null)
assert(forwarded, null)

def levy(tonnes):
    return tonnes * 8

assert(direct, 80)
assert(forwarded, 80)

---

# Nucleoid keeps a delay impact report live as a flight plan changes

# delay is a source with no scheduled consequences
delay = 10
schedule = affects delay

assert(schedule, [])

# Departure and arrival are new relationships to the unchanged delay
departure = delay + 5
arrival = departure + 20

assert(schedule.map(step => step.node), ["departure", "arrival"])

# A fixed departure withdraws the delay's support without changing its value
departure = 15

assert(schedule, [])

departure = delay + 5

assert(schedule.map(step => step.node), ["departure", "arrival"])

---

# Nucleoid refreshes a fuel explanation when an unchanged estimate gains an input

distance = 100
consumption = 2
fuel = distance * consumption
trace = why fuel
reserve = 0

# The additional reserve changes the derivation before it changes the fuel
fuel = distance * consumption + reserve

assert(trace[0].holds, 200)
assert(trace[0].from, ["distance", "consumption", "reserve"])
assert(trace.some(step => step.node == "reserve"), true)

reserve = 50

assert(trace[0].holds, 250)

fuel = distance * consumption
reserve = 100

assert(trace[0].holds, 200)
assert(trace.some(step => step.node == "reserve"), false)

---

# Nucleoid keeps an initially empty capacity model live

fleet = model |> why

assert(fleet, [])

capacity = 100
seats = capacity - 10

assert(fleet.map(step => step.node), ["capacity", "seats"])

delete seats

assert(fleet.map(step => step.node), ["capacity"])

seats = capacity - 20
explanation = why seats

assert(fleet.map(step => step.node), ["capacity", "seats"])
assert(fleet[1].holds, 80)

# Replacing an observer with a relationship changes model membership
explanation = capacity - 20

assert(fleet.map(step => step.node), ["capacity", "seats", "explanation"])

explanation = why seats

assert(fleet.map(step => step.node), ["capacity", "seats"])

---

# Nucleoid restores a staffing explanation after rejecting a crew rewrite

crew = 1
reserveCrew = 1
staffing = crew + 1
trace = why staffing

try:
    staffing = reserveCrew + 1
    throw "REJECTED_CREW_PLAN"
catch error:
    assert(error, "REJECTED_CREW_PLAN")

assert(trace[0].from, ["crew"])

reserveCrew = 4

assert(trace[0].holds, 2)

crew = 2

assert(trace[0].holds, 3)
```
