# Nucleoid Language Reference - Synthesized Use Cases 12

```

# Nucleoid converts a stay length through a chain of variables

# nights is 28
nights = 28

# weeks is nights divided by 7
weeks = nights / 7

# months is weeks divided by 4
months = weeks / 4

assert(weeks, 4)
assert(months, 1)

# nights is 56
nights = 56

assert(weeks, 8)
assert(months, 2)

---

# Nucleoid replaces the source of a room rate

# standard is 200
standard = 200

# discounted is 120
discounted = 120

# nightly is standard divided by 2
nightly = standard / 2

assert(nightly, 100)

# nightly is discounted divided by 2
nightly = discounted / 2

assert(nightly, 60)

# discounted is 160
discounted = 160

assert(nightly, 80)

---

# Nucleoid raises a room count using its own value

# rooms is 80
rooms = 80

# rooms is rooms plus 20
rooms = rooms + 20

assert(rooms, 100)

---

# Nucleoid fixes a seasonal rate with the value property

# season is 50
season = 50

# nights is 4
nights = 4

# quoted is season's value times nights
quoted = season.value * nights

assert(quoted, 200)

# season is 70
season = 70

assert(quoted, 200)

# nights is 6
nights = 6

assert(quoted, 300)

---

# Nucleoid clears a folio when its charge is deleted

# charge is 300
charge = 300

# service is charge divided by 10
service = charge / 10

# folio is charge plus service
folio = charge + service

assert(folio, 330)

# charge is deleted
delete charge

assert(service, null)
assert(folio, null)

try:
    # charge
    charge
catch error:
    assert(error, ReferenceError("charge is not defined"))

---

# Nucleoid assigns an occupancy comparison to a variable

# booked is 90
booked = 90

# capacity is 80
capacity = 80

# overbooked is whether booked is greater than capacity
overbooked = booked > capacity

assert(overbooked, true)

# booked is 60
booked = 60

assert(overbooked, false)

---

# Nucleoid builds a booking reference from a template literal

# hotel is "LIS"
hotel = "LIS"

# number is 8812
number = 8812

# reference is the hotel and number in a template
reference = `${hotel}-BKG-${number}`

assert(reference, "LIS-BKG-8812")

# number is 8813
number = 8813

assert(reference, "LIS-BKG-8813")

---

# Nucleoid combines guest flags with logical operators

# checkedIn is true
checkedIn = true

# blacklisted is false
blacklisted = false

assert(checkedIn and not blacklisted, true)
assert(checkedIn && !blacklisted, true)
assert(blacklisted or checkedIn, true)
assert(blacklisted || checkedIn, true)

---

# Nucleoid respects parentheses in a package formula

# room is 2
room = 2

# board is 3
board = 3

# guests is 4
guests = 4

assert(room + board * guests, 14)
assert((room + board) * guests, 20)

---

# Nucleoid seats a party with the remainder operator

# covers is 47
covers = 47

# perTable is 6
perTable = 6

# spare is covers modulo perTable
spare = covers % perTable

assert(spare, 5)

# covers is 48
covers = 48

assert(spare, 0)

---

# Nucleoid appends a floor number to a wing code

# wing is "WING-"
wing = "WING-"

# floor is 4
floor = 4

# code is wing plus floor
code = wing + floor

assert(code, "WING-4")

# floor is 5
floor = 5

assert(code, "WING-5")

---

# Nucleoid counts the characters of a rate code

# rateCode is "BAR"
rateCode = "BAR"

# width is rateCode's length
width = rateCode.length

assert(width, 3)

# rateCode is "BARFLEX"
rateCode = "BARFLEX"

assert(width, 7)

---

# Nucleoid lowercases a board type as a dependency

# board is "HALFBOARD"
board = "HALFBOARD"

# key is board lowercased
key = board.lower()

assert(key, "halfboard")

# board is "FULLBOARD"
board = "FULLBOARD"

assert(key, "fullboard")

---

# Nucleoid reads the block letter of a room number

# room is "A204"
room = "A204"

# block is the character of room at 0
block = room.charAt(0)

assert(block, "A")

# room is "B204"
room = "B204"

assert(block, "B")

---

# Nucleoid rewrites a rate plan with replace

# plan is "rate/old/season"
plan = "rate/old/season"

# revision is "new"
revision = "new"

# current is plan with "old" replaced by revision
current = plan.replace("old", revision)

assert(current, "rate/new/season")

# revision is "peak"
revision = "peak"

assert(current, "rate/peak/season")

---

# Nucleoid takes the year from the end of a reservation code

# code is "RSV-2023"
code = "RSV-2023"

# year is the last four characters of code
year = code[-4:]

assert(year, "2023")

# code is "RSV-2024"
code = "RSV-2024"

assert(year, "2024")

---

# Nucleoid declares a room type with a constructor

# There is a Room type,
# which has a kind as a string
class Room(kind: str):
    this.kind = kind

# room1 is a Room whose kind is "Double"
room1 = Room("Double")

assert(room1, { "id": "room1", "kind": "Double" })
assert(Room.length, 1)

---

# Nucleoid declares a suite as a subtype of a room

# There is a Room type,
# which has a kind as a string
class Room(kind: str):
    this.kind = kind

# There is a Suite type,
# which is a subtype of Room
# and has a lounge as a boolean
class Suite: Room
    def init(kind, lounge):
        super(kind)
        this.kind = kind
        this.lounge = lounge

# suite1 is a Suite whose kind is "Executive" and whose lounge is true
suite1 = Suite("Executive", true)

assert(suite1, { "id": "suite1", "kind": "Executive", "lounge": true })

---

# Nucleoid numbers every booking with a class-level rule

# There is a Booking type,
# which has a number as a number
class Booking(number: int):
    this.number = number

# Any booking's reference is "BK-" plus the booking's number
$Booking.reference = "BK-" + $Booking.number

# booking1 is a Booking whose number is 21
booking1 = Booking(21)

assert(booking1.reference, "BK-21")

# booking2 is a Booking whose number is 22
booking2 = Booking(22)

assert(booking2.reference, "BK-22")

---

# Nucleoid replaces a class-level rule on a floor

# There is a Floor type
class Floor:
    pass

# floor1 is a Floor whose number is 3
floor1 = Floor()
floor1.number = 3

# Any floor's sign is "F" plus the floor's number
$Floor.sign = "F" + $Floor.number

assert(floor1.sign, "F3")

# Any floor's sign is "FLOOR-" plus the floor's number
$Floor.sign = "FLOOR-" + $Floor.number

assert(floor1.sign, "FLOOR-3")

# floor1's number is 4
floor1.number = 4

assert(floor1.sign, "FLOOR-4")

---

# Nucleoid marks a long stay with a class-level conditional

# There is a Stay type
class Stay:
    pass

# Any stay longer than 14 nights is extended
if $Stay.nights > 14:
    $Stay.extended = true

# stay1 is a Stay whose nights is 3
stay1 = Stay()
stay1.nights = 3

assert(stay1.extended, null)

# stay1's nights is 21
stay1.nights = 21

assert(stay1.extended, true)

---

# Nucleoid tiers a guest with a class-level chain

# There is a Guest type
class Guest:
    pass

# gold is "GOLD", silver is "SILVER", and bronze is "BRONZE"
gold = "GOLD"; silver = "SILVER"; bronze = "BRONZE"

# If any guest's stays is greater than 20, then the guest's tier is gold,
# else if the guest's stays is greater than 5, then the guest's tier is silver,
# else the guest's tier is bronze
if $Guest.stays > 20:
    $Guest.tier = gold
else if $Guest.stays > 5:
    $Guest.tier = silver
else:
    $Guest.tier = bronze

# guest1 is a Guest whose stays is 10
guest1 = Guest()
guest1.stays = 10

assert(guest1.tier, "SILVER")

# silver is "PREFERRED"
silver = "PREFERRED"

assert(guest1.tier, "PREFERRED")

# guest1's stays is 30
guest1.stays = 30

assert(guest1.tier, "GOLD")

---

# Nucleoid counts the bookings of a room with a class-level aggregate

# There is a Room type
class Room:
    pass

# There is a Booking type
class Booking:
    pass

# room1 is a Room
room1 = Room()

# booking1 is a Booking whose room is room1
booking1 = Booking()
booking1.room = room1

# booking2 is a Booking whose room is room1
booking2 = Booking()
booking2.room = room1

# Any room's bookings is the number of bookings whose room is the room
$Room.bookings = Booking.filter(b => b.room == $Room).length

assert(room1.bookings, 2)

# booking3 is a Booking whose room is room1
booking3 = Booking()
booking3.room = room1

assert(room1.bookings, 3)

---

# Nucleoid builds an invoice from a property that arrives later

# There is an Invoice type
class Invoice:
    pass

# invoice1 is an Invoice
invoice1 = Invoice()

# invoice1's full is "INV" plus invoice1's serial
invoice1.full = "INV" + invoice1.serial

assert(invoice1.full, null)

# invoice1's serial is "6620"
invoice1.serial = "6620"

assert(invoice1.full, "INV6620")

---

# Nucleoid reads a guest through a booking reference

# There is a Booking type
class Booking:
    pass

# There is a Guest type
class Guest:
    pass

# booking1 is a Booking
booking1 = Booking()

# guest1 is a Guest whose name is "Tomas"
guest1 = Guest()
guest1.name = "Tomas"

# booking1's guest is guest1
booking1.guest = guest1

# booking1's held is "Held for " plus booking1's guest's name
booking1.held = "Held for " + booking1.guest.name

assert(booking1.held, "Held for Tomas")

# guest1's name is "Tomas Silva"
guest1.name = "Tomas Silva"

assert(booking1.held, "Held for Tomas Silva")

---

# Nucleoid clears a folio line when a charge is deleted

# There is a Folio type
class Folio:
    pass

# folio1 is a Folio
folio1 = Folio()

# folio1's item is "BAR"
folio1.item = "BAR"

# folio1's amount is "45"
folio1.amount = "45"

# folio1's line is folio1's item plus " " plus folio1's amount
folio1.line = folio1.item + " " + folio1.amount

assert(folio1.line, "BAR 45")

# folio1's amount is deleted
delete folio1.amount

assert(folio1.line, null)
assert(folio1.item, "BAR")

---

# Nucleoid computes an occupancy rate in a block

# available is 3650
available = 3650

# rate is null
rate = null

# while in the block, nightly is a local variable that is available divided by 365,
# and rate is nightly times 365
{
    nightly = available / 365
    rate = nightly * 365
}

assert(rate, 3650)

# available is 7300
available = 7300

assert(rate, 7300)

---

# Nucleoid computes a ballroom area in a nested block

# side is 12
side = 12

# while in the block, face is a local variable that is side squared,
# and in a nested block, area is face times 2
{
    face = Math.pow(side, 2)
    {
        area = face * 2
    }
}

assert(area, 288)

---

# Nucleoid raises a housekeeping alert in a nested if inside a block

# rooms is 120
rooms = 120

# cleaned is 60
cleaned = 60

# notify is true
notify = true

# while in the block, pending is a local variable that is rooms minus cleaned,
# and if pending is greater than 40, then alert is notify
{
    pending = rooms - cleaned
    if pending > 40:
        alert = notify
}

assert(alert, true)

# notify is false
notify = false

assert(alert, false)

---

# Nucleoid picks a service level in a nested else inside a block

# guests is 2
guests = 2

# nights is 3
nights = 3

# butler is "BUTLER"
butler = "BUTLER"

# standard is "STANDARD"
standard = "STANDARD"

# while in the block, demand is a local variable that is guests times nights,
# and if demand is greater than 20, then service is butler,
# else service is standard
{
    demand = guests * nights
    if demand > 20:
        service = butler
    else:
        service = standard
}

assert(service, "STANDARD")

# standard is "STD"
standard = "STD"

assert(service, "STD")

---

# Nucleoid computes a stay total in a class-level block

# There is a Reservation type
class Reservation:
    pass

# while in the block, tax is a local variable that is any reservation's net times 20 divided by 100,
# and the reservation's gross is the reservation's net plus tax
{
    tax = $Reservation.net * 20 / 100
    $Reservation.gross = $Reservation.net + tax
}

# reservation1 is a Reservation
reservation1 = Reservation()

assert(reservation1.gross, null)

# reservation1's net is 500
reservation1.net = 500

assert(reservation1.gross, 600)

---

# Nucleoid computes a covers index in a nested class-level block

# There is a Restaurant type
class Restaurant:
    pass

# while in the block, share is a local variable that is 300 divided by any restaurant's tables,
# and in a nested block, the restaurant's index is the floor of share times the restaurant's sittings
{
    share = 300 / $Restaurant.tables
    {
        $Restaurant.index = Math.floor(share * $Restaurant.sittings)
    }
}

# restaurant1 is a Restaurant
restaurant1 = Restaurant()

# restaurant1's tables is 7
restaurant1.tables = 7

# restaurant1's sittings is 3
restaurant1.sittings = 3

assert(restaurant1.index, 128)

---

# Nucleoid flags a no show with an if statement on a property

# There is an Arrival type
class Arrival:
    pass

# arrival1 is an Arrival whose state is "ARRIVED"
arrival1 = Arrival()
arrival1.state = "ARRIVED"

# if arrival1's state is "MISSED", then arrival1's noShow is true
if arrival1.state == "MISSED":
    arrival1.noShow = true

assert(arrival1.noShow, null)

# arrival1's state is "MISSED"
arrival1.state = "MISSED"

assert(arrival1.noShow, true)

---

# Nucleoid chooses a turndown note with an else statement on a property

# There is a Room type
class Room:
    pass

# room1 is a Room whose vip is false
room1 = Room()
room1.vip = false

# special is "TURNDOWN AND FLOWERS"
special = "TURNDOWN AND FLOWERS"

# usual is "TURNDOWN"
usual = "TURNDOWN"

# if room1's vip is true, then room1's note is special,
# else room1's note is usual
if room1.vip == true:
    room1.note = special
else:
    room1.note = usual

assert(room1.note, "TURNDOWN")

# room1's vip is true
room1.vip = true

assert(room1.note, "TURNDOWN AND FLOWERS")

---

# Nucleoid bands a cancellation fee with multiple else if statements on a property

# There is a Cancellation type
class Cancellation:
    pass

# cancellation1 is a Cancellation whose hours is 100
cancellation1 = Cancellation()
cancellation1.hours = 100

# unit is 2
unit = 2

# if cancellation1's hours is less than 24, then cancellation1's fee is cancellation1's hours times unit plus 200,
# else if cancellation1's hours is less than 72, then cancellation1's fee is cancellation1's hours times unit plus 50,
# else cancellation1's fee is cancellation1's hours times unit
if cancellation1.hours < 24:
    cancellation1.fee = cancellation1.hours * unit + 200
else if cancellation1.hours < 72:
    cancellation1.fee = cancellation1.hours * unit + 50
else:
    cancellation1.fee = cancellation1.hours * unit

assert(cancellation1.fee, 200)

# cancellation1's hours is 48
cancellation1.hours = 48

assert(cancellation1.fee, 146)

# cancellation1's hours is 12
cancellation1.hours = 12

assert(cancellation1.fee, 224)

---

# Nucleoid calls a rate function in an assignment

# rate returns the total divided by the nights
def rate(total, nights):
    return total / nights

# spend is 600
spend = 600

# stay is 3
stay = 3

# nightly is the result of the rate function call
nightly = rate(spend, stay)

assert(nightly, 200)

# stay is 6
stay = 6

assert(nightly, 100)

---

# Nucleoid updates a service charge when its function is redefined

# service returns the bill times 10 divided by 100
def service(bill):
    return bill * 10 / 100

# food is 200
food = 200

# added is the result of the service function call with food
added = service(food)

assert(added, 20)

# service returns the bill times 15 divided by 100
def service(bill):
    return bill * 15 / 100

assert(added, 30)

---

# Nucleoid nests a season lookup inside a quote function

# season returns 4 times the month
def season(month):
    return month * 4

# quote returns the nights times the season of the month
def quote(nights, month):
    return nights * season(month)

# stayNights is 5
stayNights = 5

# stayMonth is 3
stayMonth = 3

# price is the result of the quote function call
price = quote(stayNights, stayMonth)

assert(price, 60)

---

# Nucleoid finds a rate with the three lambda forms

# rates is a list of 90, 120 and 150
rates = [90, 120, 150]

assert(rates.find(function(rate) { return rate == 150 }), 150)
assert(rates.find(rate => { return rate == 120 }), 120)
assert(rates.find(rate => rate == 90), 90)

---

# Nucleoid filters a room list by two thresholds

# There is a Room type,
# which has a rate as a number
class Room(rate: int):
    this.rate = rate

# There are Rooms whose rates are 80, 150 and 250
Room(80); Room(150); Room(250)

# ceiling is 200
ceiling = 200

# floorRate is 100
floorRate = 100

# midrange is Rooms whose rate is above floorRate and below ceiling
midrange = Room.filter(r => r.rate > floorRate).filter(r => r.rate < ceiling)

assert(midrange.length, 1)
assert(midrange[0].rate, 150)

# floorRate is 50
floorRate = 50

assert(midrange.length, 2)
assert(midrange[0].rate, 80)

---

# Nucleoid maps nightly rates into stay totals

# nights is a list of 2, 3 and 4
nights = [2, 3, 4]

# nightly is 100
nightly = 100

# totals is nights mapped to the night times nightly
totals = nights.map(n => n * nightly)

assert(totals[0], 200)
assert(totals[2], 400)

# nightly is 150
nightly = 150

assert(totals[2], 600)

---

# Nucleoid reduces a folio into a total

# charges is a list of 120, 180 and 200
charges = [120, 180, 200]

# total is the sum of charges
total = charges.reduce((sum, charge) => sum + charge, 0)

assert(total, 500)

# Add 100 to charges
charges.push(100)

assert(total, 600)

---

# Nucleoid checks whether every room is serviced

# There is a Room type
class Room:
    pass

# room1 is a Room that is serviced
room1 = Room()
room1.serviced = true

# room2 is a Room that is serviced
room2 = Room()
room2.serviced = true

# ready is whether every room is serviced
ready = Room.every(r => r.serviced == true)

assert(ready, true)

# room2 is not serviced
room2.serviced = false

assert(ready, false)

---

# Nucleoid checks whether any booking is unpaid

# There is a Booking type
class Booking:
    pass

# booking1 is a Booking that is paid
booking1 = Booking()
booking1.unpaid = false

# booking2 is a Booking that is paid
booking2 = Booking()
booking2.unpaid = false

# chase is whether any booking is unpaid
chase = Booking.some(b => b.unpaid == true)

assert(chase, false)

# booking2 is unpaid
booking2.unpaid = true

assert(chase, true)

---

# Nucleoid joins an itinerary into a single string

# stops is a list of "CHECKIN", "DINNER" and "SPA"
stops = ["CHECKIN", "DINNER", "SPA"]

# itinerary is stops joined with " | "
itinerary = stops.join(" | ")

assert(itinerary, "CHECKIN | DINNER | SPA")

# Add "CHECKOUT" to stops
stops.push("CHECKOUT")

assert(itinerary, "CHECKIN | DINNER | SPA | CHECKOUT")

---

# Nucleoid creates a key card for every active booking

# There is a Booking type
class Booking:
    pass

# booking1 is a Booking
booking1 = Booking()

# booking2 is a Booking that is cancelled
booking2 = Booking()
booking2.cancelled = true

# booking3 is a Booking
booking3 = Booking()

# There is a Card type,
# which has a booking as a Booking
class Card(booking):
    this.booking = booking

# Any card's kind is "ROOM"
$Card.kind = "ROOM"

# For each booking of Booking, if the booking is not cancelled,
# then there is a Card whose booking is the booking
for booking of Booking:
    if not booking.cancelled:
        Card(booking)

assert(Card.length, 2)
assert(Card[0].booking.id, "booking1")
assert(Card[1].booking.id, "booking3")
assert(Card[0].kind, "ROOM")

---

# Nucleoid rolls back a reservation if a rule throws

# There is a Reservation type
class Reservation:
    pass

# If any reservation's guests is greater than 6, then throw 'PARTY_TOO_LARGE'
if $Reservation.guests > 6:
    throw 'PARTY_TOO_LARGE'

# reservation1 is a Reservation
reservation1 = Reservation()

try:
    # reservation1's guests is 10
    reservation1.guests = 10
catch error:
    assert(error, "PARTY_TOO_LARGE")

assert(reservation1.guests, null)

---

# Nucleoid refuses a cycle between a rate and a total

# rate is 90
rate = 90

# total is rate times 3
total = rate * 3

assert(total, 270)

try:
    # rate is total times 3
    rate = total * 3
catch error:
    assert(error, TypeError("Circular Dependency"))

---

# Nucleoid validates a loyalty number with a regular expression

# There is a Member type
class Member:
    pass

# If any member's number does not match /[0-9]{5}/, then throw 'INVALID_MEMBER'
if not /[0-9]{5}/.test($Member.number):
    throw 'INVALID_MEMBER'

# member1 is a Member
member1 = Member()

assert(member1.number, null)

try:
    # member1's number is '12'
    member1.number = '12'
catch error:
    assert(error, "INVALID_MEMBER")
```
