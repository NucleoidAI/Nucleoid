# Nucleoid Language Reference - Synthesized Use Cases 14

```

# Nucleoid converts a journey distance through a chain of variables

# metres is 42000
metres = 42000

# kilometres is metres divided by 1000
kilometres = metres / 1000

# stages is kilometres divided by 6
stages = kilometres / 6

assert(kilometres, 42)
assert(stages, 7)

# metres is 84000
metres = 84000

assert(kilometres, 84)
assert(stages, 14)

---

# Nucleoid replaces the source of a fare

# peak is 40
peak = 40

# offPeak is 20
offPeak = 20

# fare is peak divided by 2
fare = peak / 2

assert(fare, 20)

# fare is offPeak divided by 2
fare = offPeak / 2

assert(fare, 10)

# offPeak is 30
offPeak = 30

assert(fare, 15)

---

# Nucleoid raises a seat count using its own value

# seats is 320
seats = 320

# seats is seats plus 80
seats = seats + 80

assert(seats, 400)

---

# Nucleoid fixes a timetable slot with the value property

# headway is 6
headway = 6

# hours is 5
hours = 5

# services is headway's value times hours
services = headway.value * hours

assert(services, 30)

# headway is 10
headway = 10

assert(services, 30)

# hours is 8
hours = 8

assert(services, 48)

---

# Nucleoid clears a journey time when its distance is deleted

# distance is 240
distance = 240

# hours is distance divided by 120
hours = distance / 120

# minutes is hours times 60
minutes = hours * 60

assert(minutes, 120)

# distance is deleted
delete distance

assert(hours, null)
assert(minutes, null)

try:
    # distance
    distance
catch error:
    assert(error, ReferenceError("distance is not defined"))

---

# Nucleoid assigns a delay comparison to a variable

# actual = 15
actual = 15

# scheduled is 10
scheduled = 10

# delayed is whether actual is greater than scheduled
delayed = actual > scheduled

assert(delayed, true)

# actual is 8
actual = 8

assert(delayed, false)

---

# Nucleoid builds a service code from a template literal

# route is "WC"
route = "WC"

# departure is 1435
departure = 1435

# service is the route and departure in a template
service = `${route}-${departure}`

assert(service, "WC-1435")

# departure is 1535
departure = 1535

assert(service, "WC-1535")

---

# Nucleoid combines platform flags with logical operators

# staffed is true
staffed = true

# closed is false
closed = false

assert(staffed and not closed, true)
assert(staffed && !closed, true)
assert(closed or staffed, true)
assert(closed || staffed, true)

---

# Nucleoid respects parentheses in a capacity formula

# coaches is 2
coaches = 2

# rows is 3
rows = 3

# seats is 4
seats = 4

assert(coaches + rows * seats, 14)
assert((coaches + rows) * seats, 20)

---

# Nucleoid allocates carriages with the remainder operator

# passengers is 431
passengers = 431

# perCoach is 60
perCoach = 60

# standing is passengers modulo perCoach
standing = passengers % perCoach

assert(standing, 11)

# passengers is 420
passengers = 420

assert(standing, 0)

---

# Nucleoid appends a platform number to a station code

# station is "PLT-"
station = "PLT-"

# platform is 9
platform = 9

# code is station plus platform
code = station + platform

assert(code, "PLT-9")

# platform is 10
platform = 10

assert(code, "PLT-10")

---

# Nucleoid counts the characters of a headcode

# headcode is "1A20"
headcode = "1A20"

# width is headcode's length
width = headcode.length

assert(width, 4)

# headcode is "1A20XYZ"
headcode = "1A20XYZ"

assert(width, 7)

---

# Nucleoid lowercases an operator name as a dependency

# operator is "NORTHERN"
operator = "NORTHERN"

# key is operator lowercased
key = operator.lower()

assert(key, "northern")

# operator is "SOUTHERN"
operator = "SOUTHERN"

assert(key, "southern")

---

# Nucleoid reads the region letter of a depot code

# depot is "E44"
depot = "E44"

# region is the character of depot at 0
region = depot.charAt(0)

assert(region, "E")

# depot is "W44"
depot = "W44"

assert(region, "W")

---

# Nucleoid rewrites a diagram path with replace

# diagram is "plan/old/turn"
diagram = "plan/old/turn"

# revision is "new"
revision = "new"

# current is diagram with "old" replaced by revision
current = diagram.replace("old", revision)

assert(current, "plan/new/turn")

# revision is "sun"
revision = "sun"

assert(current, "plan/sun/turn")

---

# Nucleoid takes the time from the end of a service label

# label is "SVC-0730"
label = "SVC-0730"

# time is the last four characters of label
time = label[-4:]

assert(time, "0730")

# label is "SVC-0830"
label = "SVC-0830"

assert(time, "0830")

---

# Nucleoid declares a station type with a constructor

# There is a Station type,
# which has a name as a string
class Station(name: str):
    this.name = name

# station1 is a Station whose name is "Central"
station1 = Station("Central")

assert(station1, { "id": "station1", "name": "Central" })
assert(Station.length, 1)

---

# Nucleoid declares a terminus as a subtype of a station

# There is a Station type,
# which has a name as a string
class Station(name: str):
    this.name = name

# There is a Terminus type,
# which is a subtype of Station
# and has a platforms as a number
class Terminus: Station
    def init(name, platforms):
        super(name)
        this.name = name
        this.platforms = platforms

# terminus1 is a Terminus whose name is "Kings" and whose platforms is 12
terminus1 = Terminus("Kings", 12)

assert(terminus1, { "id": "terminus1", "name": "Kings", "platforms": 12 })

---

# Nucleoid numbers every service with a class-level rule

# There is a Service type,
# which has a number as a number
class Service(number: int):
    this.number = number

# Any service's headcode is "1A" plus the service's number
$Service.headcode = "1A" + $Service.number

# service1 is a Service whose number is 12
service1 = Service(12)

assert(service1.headcode, "1A12")

# service2 is a Service whose number is 13
service2 = Service(13)

assert(service2.headcode, "1A13")

---

# Nucleoid replaces a class-level rule on a platform

# There is a Platform type
class Platform:
    pass

# platform1 is a Platform whose number is 4
platform1 = Platform()
platform1.number = 4

# Any platform's sign is "P" plus the platform's number
$Platform.sign = "P" + $Platform.number

assert(platform1.sign, "P4")

# Any platform's sign is "PLATFORM-" plus the platform's number
$Platform.sign = "PLATFORM-" + $Platform.number

assert(platform1.sign, "PLATFORM-4")

# platform1's number is 5
platform1.number = 5

assert(platform1.sign, "PLATFORM-5")

---

# Nucleoid marks a cancellation with a class-level conditional

# There is a Train type
class Train:
    pass

# Any train more than 60 minutes late is cancelled
if $Train.late > 60:
    $Train.cancelled = true

# train1 is a Train whose late is 10
train1 = Train()
train1.late = 10

assert(train1.cancelled, null)

# train1's late is 90
train1.late = 90

assert(train1.cancelled, true)

---

# Nucleoid classifies a delay with a class-level chain

# There is a Delay type
class Delay:
    pass

# severe is "SEVERE", moderate is "MODERATE", and slight is "SLIGHT"
severe = "SEVERE"; moderate = "MODERATE"; slight = "SLIGHT"

# If any delay's minutes is greater than 30, then the delay's grade is severe,
# else if the delay's minutes is greater than 5, then the delay's grade is moderate,
# else the delay's grade is slight
if $Delay.minutes > 30:
    $Delay.grade = severe
else if $Delay.minutes > 5:
    $Delay.grade = moderate
else:
    $Delay.grade = slight

# delay1 is a Delay whose minutes is 15
delay1 = Delay()
delay1.minutes = 15

assert(delay1.grade, "MODERATE")

# moderate is "MINOR"
moderate = "MINOR"

assert(delay1.grade, "MINOR")

# delay1's minutes is 45
delay1.minutes = 45

assert(delay1.grade, "SEVERE")

---

# Nucleoid counts the calls of a route with a class-level aggregate

# There is a Route type
class Route:
    pass

# There is a Call type
class Call:
    pass

# route1 is a Route
route1 = Route()

# call1 is a Call whose route is route1
call1 = Call()
call1.route = route1

# call2 is a Call whose route is route1
call2 = Call()
call2.route = route1

# Any route's calls is the number of calls whose route is the route
$Route.calls = Call.filter(c => c.route == $Route).length

assert(route1.calls, 2)

# call3 is a Call whose route is route1
call3 = Call()
call3.route = route1

assert(route1.calls, 3)

---

# Nucleoid builds a ticket from a property that arrives later

# There is a Ticket type
class Ticket:
    pass

# ticket1 is a Ticket
ticket1 = Ticket()

# ticket1's full is "TK" plus ticket1's serial
ticket1.full = "TK" + ticket1.serial

assert(ticket1.full, null)

# ticket1's serial is "5590"
ticket1.serial = "5590"

assert(ticket1.full, "TK5590")

---

# Nucleoid reads a driver through a service reference

# There is a Service type
class Service:
    pass

# There is a Driver type
class Driver:
    pass

# service1 is a Service
service1 = Service()

# driver1 is a Driver whose name is "Rowan"
driver1 = Driver()
driver1.name = "Rowan"

# service1's driver is driver1
service1.driver = driver1

# service1's crewed is "Driven by " plus service1's driver's name
service1.crewed = "Driven by " + service1.driver.name

assert(service1.crewed, "Driven by Rowan")

# driver1's name is "Rowan Hale"
driver1.name = "Rowan Hale"

assert(service1.crewed, "Driven by Rowan Hale")

---

# Nucleoid clears a board entry when a time is deleted

# There is a Board type
class Board:
    pass

# board1 is a Board
board1 = Board()

# board1's destination is "YORK"
board1.destination = "YORK"

# board1's time is "1030"
board1.time = "1030"

# board1's entry is board1's time plus " " plus board1's destination
board1.entry = board1.time + " " + board1.destination

assert(board1.entry, "1030 YORK")

# board1's time is deleted
delete board1.time

assert(board1.entry, null)
assert(board1.destination, "YORK")

---

# Nucleoid computes an average speed in a block

# distance is 720
distance = 720

# speed is null
speed = null

# while in the block, hours is a local variable that is distance divided by 6,
# and speed is hours times 1
{
    hours = distance / 6
    speed = hours * 1
}

assert(speed, 120)

# distance is 960
distance = 960

assert(speed, 160)

---

# Nucleoid computes a concourse area in a nested block

# side is 20
side = 20

# while in the block, face is a local variable that is side squared,
# and in a nested block, area is face times 2
{
    face = Math.pow(side, 2)
    {
        area = face * 2
    }
}

assert(area, 800)

---

# Nucleoid raises a crowding alert in a nested if inside a block

# onboard is 500
onboard = 500

# seated is 380
seated = 380

# notify is true
notify = true

# while in the block, standing is a local variable that is onboard minus seated,
# and if standing is greater than 100, then alert is notify
{
    standing = onboard - seated
    if standing > 100:
        alert = notify
}

assert(alert, true)

# notify is false
notify = false

assert(alert, false)

---

# Nucleoid picks a replacement mode in a nested else inside a block

# stops is 3
stops = 3

# hours is 2
hours = 2

# rail is "RAIL"
rail = "RAIL"

# bus is "BUS"
bus = "BUS"

# while in the block, effort is a local variable that is stops times hours,
# and if effort is greater than 20, then mode is rail,
# else mode is bus
{
    effort = stops * hours
    if effort > 20:
        mode = rail
    else:
        mode = bus
}

assert(mode, "BUS")

# bus is "COACH"
bus = "COACH"

assert(mode, "COACH")

---

# Nucleoid computes a fare total in a class-level block

# There is a Journey type
class Journey:
    pass

# while in the block, levy is a local variable that is any journey's fare times 10 divided by 100,
# and the journey's total is the journey's fare plus levy
{
    levy = $Journey.fare * 10 / 100
    $Journey.total = $Journey.fare + levy
}

# journey1 is a Journey
journey1 = Journey()

assert(journey1.total, null)

# journey1's fare is 60
journey1.fare = 60

assert(journey1.total, 66)

---

# Nucleoid computes a path index in a nested class-level block

# There is a Line type
class Line:
    pass

# while in the block, share is a local variable that is 600 divided by any line's tracks,
# and in a nested block, the line's index is the floor of share times the line's signals
{
    share = 600 / $Line.tracks
    {
        $Line.index = Math.floor(share * $Line.signals)
    }
}

# line1 is a Line
line1 = Line()

# line1's tracks is 7
line1.tracks = 7

# line1's signals is 4
line1.signals = 4

assert(line1.index, 342)

---

# Nucleoid flags an evacuation with an if statement on a property

# There is a Station type
class Station:
    pass

# station1 is a Station whose state is "OPEN"
station1 = Station()
station1.state = "OPEN"

# if station1's state is "ALARM", then station1's evacuate is true
if station1.state == "ALARM":
    station1.evacuate = true

assert(station1.evacuate, null)

# station1's state is "ALARM"
station1.state = "ALARM"

assert(station1.evacuate, true)

---

# Nucleoid chooses a boarding note with an else statement on a property

# There is a Coach type
class Coach:
    pass

# coach1 is a Coach whose accessible is false
coach1 = Coach()
coach1.accessible = false

# assisted = "ASSISTED BOARDING"
assisted = "ASSISTED BOARDING"

# usual is "STANDARD BOARDING"
usual = "STANDARD BOARDING"

# if coach1's accessible is true, then coach1's note is assisted,
# else coach1's note is usual
if coach1.accessible == true:
    coach1.note = assisted
else:
    coach1.note = usual

assert(coach1.note, "STANDARD BOARDING")

# coach1's accessible is true
coach1.accessible = true

assert(coach1.note, "ASSISTED BOARDING")

---

# Nucleoid bands a compensation with multiple else if statements on a property

# There is a Claim type
class Claim:
    pass

# claim1 is a Claim whose minutes is 20
claim1 = Claim()
claim1.minutes = 20

# unit is 2
unit = 2

# if claim1's minutes is greater than 120, then claim1's award is claim1's minutes times unit plus 100,
# else if claim1's minutes is greater than 60, then claim1's award is claim1's minutes times unit plus 40,
# else claim1's award is claim1's minutes times unit
if claim1.minutes > 120:
    claim1.award = claim1.minutes * unit + 100
else if claim1.minutes > 60:
    claim1.award = claim1.minutes * unit + 40
else:
    claim1.award = claim1.minutes * unit

assert(claim1.award, 40)

# claim1's minutes is 90
claim1.minutes = 90

assert(claim1.award, 220)

# claim1's minutes is 180
claim1.minutes = 180

assert(claim1.award, 460)

---

# Nucleoid calls a speed function in an assignment

# speed returns the distance divided by the hours
def speed(distance, hours):
    return distance / hours

# route is 300
route = 300

# duration is 3
duration = 3

# average is the result of the speed function call
average = speed(route, duration)

assert(average, 100)

# duration is 6
duration = 6

assert(average, 50)

---

# Nucleoid updates a surcharge when its function is redefined

# surcharge returns the fare times 5 divided by 100
def surcharge(fare):
    return fare * 5 / 100

# base is 200
base = 200

# extra is the result of the surcharge function call with base
extra = surcharge(base)

assert(extra, 10)

# surcharge returns the fare times 15 divided by 100
def surcharge(fare):
    return fare * 15 / 100

assert(extra, 30)

---

# Nucleoid nests a zone lookup inside a fare function

# zone returns 3 times the ring
def zone(ring):
    return ring * 3

# fare returns the stops times the zone of the ring
def fare(stops, ring):
    return stops * zone(ring)

# stopCount is 5
stopCount = 5

# ringNumber is 2
ringNumber = 2

# price is the result of the fare function call
price = fare(stopCount, ringNumber)

assert(price, 30)

---

# Nucleoid finds a platform with the three lambda forms

# platforms is a list of 1, 4 and 9
platforms = [1, 4, 9]

assert(platforms.find(function(platform) { return platform == 9 }), 9)
assert(platforms.find(platform => { return platform == 4 }), 4)
assert(platforms.find(platform => platform == 1), 1)

---

# Nucleoid filters a timetable by two thresholds

# There is a Departure type,
# which has a minute as a number
class Departure(minute: int):
    this.minute = minute

# There are Departures whose minutes are 5, 25 and 45
Departure(5); Departure(25); Departure(45)

# ceiling is 35
ceiling = 35

# floorMinute is 15
floorMinute = 15

# window is Departures whose minute is above floorMinute and below ceiling
window = Departure.filter(d => d.minute > floorMinute).filter(d => d.minute < ceiling)

assert(window.length, 1)
assert(window[0].minute, 25)

# floorMinute is 1
floorMinute = 1

assert(window.length, 2)
assert(window[0].minute, 5)

---

# Nucleoid maps distances into times

# legs is a list of 60, 120 and 180
legs = [60, 120, 180]

# perKm is 2
perKm = 2

# times is legs mapped to the leg times perKm
times = legs.map(l => l * perKm)

assert(times[0], 120)
assert(times[2], 360)

# perKm is 3
perKm = 3

assert(times[2], 540)

---

# Nucleoid reduces a set of legs into a total

# legs is a list of 40, 60 and 80
legs = [40, 60, 80]

# total is the sum of legs
total = legs.reduce((sum, leg) => sum + leg, 0)

assert(total, 180)

# Add 20 to legs
legs.push(20)

assert(total, 200)

---

# Nucleoid checks whether every coach is cleaned

# There is a Coach type
class Coach:
    pass

# coach1 is a Coach that is cleaned
coach1 = Coach()
coach1.cleaned = true

# coach2 is a Coach that is cleaned
coach2 = Coach()
coach2.cleaned = true

# ready is whether every coach is cleaned
ready = Coach.every(c => c.cleaned == true)

assert(ready, true)

# coach2 is not cleaned
coach2.cleaned = false

assert(ready, false)

---

# Nucleoid checks whether any signal is failed

# There is a Signal type
class Signal:
    pass

# signal1 is a Signal that is working
signal1 = Signal()
signal1.failed = false

# signal2 is a Signal that is working
signal2 = Signal()
signal2.failed = false

# disrupted is whether any signal is failed
disrupted = Signal.some(s => s.failed == true)

assert(disrupted, false)

# signal2 is failed
signal2.failed = true

assert(disrupted, true)

---

# Nucleoid joins a calling pattern into a single string

# calls is a list of "LDS", "YRK" and "NCL"
calls = ["LDS", "YRK", "NCL"]

# pattern is calls joined with " - "
pattern = calls.join(" - ")

assert(pattern, "LDS - YRK - NCL")

# Add "EDB" to calls
calls.push("EDB")

assert(pattern, "LDS - YRK - NCL - EDB")

---

# Nucleoid creates a diagram for every running service

# There is a Service type
class Service:
    pass

# service1 is a Service
service1 = Service()

# service2 is a Service that is cancelled
service2 = Service()
service2.cancelled = true

# service3 is a Service
service3 = Service()

# There is a Diagram type,
# which has a service as a Service
class Diagram(service):
    this.service = service

# Any diagram's kind is "WEEKDAY"
$Diagram.kind = "WEEKDAY"

# For each service of Service, if the service is not cancelled,
# then there is a Diagram whose service is the service
for service of Service:
    if not service.cancelled:
        Diagram(service)

assert(Diagram.length, 2)
assert(Diagram[0].service.id, "service1")
assert(Diagram[1].service.id, "service3")
assert(Diagram[0].kind, "WEEKDAY")

---

# Nucleoid rolls back a booking if a rule throws

# There is a Booking type
class Booking:
    pass

# If any booking's seats is greater than 9, then throw 'GROUP_BOOKING_REQUIRED'
if $Booking.seats > 9:
    throw 'GROUP_BOOKING_REQUIRED'

# booking1 is a Booking
booking1 = Booking()

try:
    # booking1's seats is 20
    booking1.seats = 20
catch error:
    assert(error, "GROUP_BOOKING_REQUIRED")

assert(booking1.seats, null)

---

# Nucleoid refuses a cycle between a fare and a total

# fare is 15
fare = 15

# total is fare times 4
total = fare * 4

assert(total, 60)

try:
    # fare is total times 4
    fare = total * 4
catch error:
    assert(error, TypeError("Circular Dependency"))

---

# Nucleoid validates a headcode with a regular expression

# There is a Working type
class Working:
    pass

# If any working's headcode does not match /[0-9][A-Z][0-9]{2}/, then throw 'INVALID_HEADCODE'
if not /[0-9][A-Z][0-9]{2}/.test($Working.headcode):
    throw 'INVALID_HEADCODE'

# working1 is a Working
working1 = Working()

assert(working1.headcode, null)

try:
    # working1's headcode is 'AA'
    working1.headcode = 'AA'
catch error:
    assert(error, "INVALID_HEADCODE")
```
