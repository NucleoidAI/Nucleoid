# Nucleoid Language Reference - Synthesized Use Cases 20

```

# Nucleoid converts a cargo weight through a chain of variables

# kilograms is 45000
kilograms = 45000

# tonnes is kilograms divided by 1000
tonnes = kilograms / 1000

# containers is tonnes divided by 15
containers = tonnes / 15

assert(tonnes, 45)
assert(containers, 3)

# kilograms is 90000
kilograms = 90000

assert(tonnes, 90)
assert(containers, 6)

---

# Nucleoid replaces the source of a berth fee

# deepwater is 800
deepwater = 800

# coastal is 400
coastal = 400

# fee is deepwater divided by 2
fee = deepwater / 2

assert(fee, 400)

# fee is coastal divided by 2
fee = coastal / 2

assert(fee, 200)

# coastal is 600
coastal = 600

assert(fee, 300)

---

# Nucleoid raises a fleet count using its own value

# vessels is 18
vessels = 18

# vessels is vessels plus 6
vessels = vessels + 6

assert(vessels, 24)

---

# Nucleoid fixes a pilotage rate with the value property

# rate is 40
rate = 40

# hours is 6
hours = 6

# quoted is rate's value times hours
quoted = rate.value * hours

assert(quoted, 240)

# rate is 50
rate = 50

assert(quoted, 240)

# hours is 12
hours = 12

assert(quoted, 480)

---

# Nucleoid clears a manifest when its tonnage is deleted

# tonnage is 800
tonnage = 800

# dunnage is tonnage divided by 8
dunnage = tonnage / 8

# gross is tonnage plus dunnage
gross = tonnage + dunnage

assert(gross, 900)

# tonnage is deleted
delete tonnage

assert(dunnage, null)
assert(gross, null)

try:
    # tonnage
    tonnage
catch error:
    assert(error, ReferenceError("tonnage is not defined"))

---

# Nucleoid assigns a draught comparison to a variable

# draught is 14
draught = 14

# depth is 12
depth = 12

# aground is whether draught is greater than depth
aground = draught > depth

assert(aground, true)

# draught is 10
draught = 10

assert(aground, false)

---

# Nucleoid builds a voyage reference from a template literal

# port is "LIS"
port = "LIS"

# voyage is 220
voyage = 220

# reference is the port and voyage in a template
reference = `${port}-VOY-${voyage}`

assert(reference, "LIS-VOY-220")

# voyage is 221
voyage = 221

assert(reference, "LIS-VOY-221")

---

# Nucleoid combines clearance flags with logical operators

# cleared is true
cleared = true

# detained is false
detained = false

assert(cleared and not detained, true)
assert(cleared && !detained, true)
assert(detained or cleared, true)
assert(detained || cleared, true)

---

# Nucleoid respects parentheses in a stowage formula

# holds is 2
holds = 2

# tiers is 3
tiers = 3

# rows is 4
rows = 4

assert(holds + tiers * rows, 14)
assert((holds + tiers) * rows, 20)

---

# Nucleoid stows containers with the remainder operator

# boxes is 437
boxes = 437

# perBay is 40
perBay = 40

# spare is boxes modulo perBay
spare = boxes % perBay

assert(spare, 37)

# boxes is 440
boxes = 440

assert(spare, 0)

---

# Nucleoid appends a berth number to a quay code

# quay is "QY-"
quay = "QY-"

# berth is 7
berth = 7

# code is quay plus berth
code = quay + berth

assert(code, "QY-7")

# berth is 8
berth = 8

assert(code, "QY-8")

---

# Nucleoid counts the characters of a call sign

# callSign is "GABC"
callSign = "GABC"

# width is callSign's length
width = callSign.length

assert(width, 4)

# callSign is "GABCDEF"
callSign = "GABCDEF"

assert(width, 7)

---

# Nucleoid lowercases a cargo type as a dependency

# cargo is "REEFER"
cargo = "REEFER"

# key is cargo lowercased
key = cargo.lower()

assert(key, "reefer")

# cargo is "BULK"
cargo = "BULK"

assert(key, "bulk")

---

# Nucleoid reads the terminal letter of a berth code

# berth is "T04"
berth = "T04"

# terminal is the character of berth at 0
terminal = berth.charAt(0)

assert(terminal, "T")

# berth is "U04"
berth = "U04"

assert(terminal, "U")

---

# Nucleoid rewrites a route path with replace

# route is "leg/old/port"
route = "leg/old/port"

# revision is "new"
revision = "new"

# current is route with "old" replaced by revision
current = route.replace("old", revision)

assert(current, "leg/new/port")

# revision is "alt"
revision = "alt"

assert(current, "leg/alt/port")

---

# Nucleoid takes the year from the end of a voyage code

# voyageCode is "VOY-2021"
voyageCode = "VOY-2021"

# year is the last four characters of voyageCode
year = voyageCode[-4:]

assert(year, "2021")

# voyageCode is "VOY-2022"
voyageCode = "VOY-2022"

assert(year, "2022")

---

# Nucleoid declares a vessel type with a constructor

# There is a Vessel type,
# which has a name as a string
class Vessel(name: str):
    this.name = name

# vessel1 is a Vessel whose name is "Aurora"
vessel1 = Vessel("Aurora")

assert(vessel1, { "id": "vessel1", "name": "Aurora" })
assert(Vessel.length, 1)

---

# Nucleoid declares a tanker as a subtype of a vessel

# There is a Vessel type,
# which has a name as a string
class Vessel(name: str):
    this.name = name

# There is a Tanker type,
# which is a subtype of Vessel
# and has a tanks as a number
class Tanker: Vessel
    def init(name, tanks):
        super(name)
        this.name = name
        this.tanks = tanks

# tanker1 is a Tanker whose name is "Meridian" and whose tanks is 12
tanker1 = Tanker("Meridian", 12)

assert(tanker1, { "id": "tanker1", "name": "Meridian", "tanks": 12 })

---

# Nucleoid numbers every call with a class-level rule

# There is a Call type,
# which has a number as a number
class Call(number: int):
    this.number = number

# Any call's reference is "PC-" plus the call's number
$Call.reference = "PC-" + $Call.number

# call1 is a Call whose number is 14
call1 = Call(14)

assert(call1.reference, "PC-14")

# call2 is a Call whose number is 15
call2 = Call(15)

assert(call2.reference, "PC-15")

---

# Nucleoid replaces a class-level rule on a crane

# There is a Crane type
class Crane:
    pass

# crane1 is a Crane whose number is 3
crane1 = Crane()
crane1.number = 3

# Any crane's sign is "C" plus the crane's number
$Crane.sign = "C" + $Crane.number

assert(crane1.sign, "C3")

# Any crane's sign is "CRANE-" plus the crane's number
$Crane.sign = "CRANE-" + $Crane.number

assert(crane1.sign, "CRANE-3")

# crane1's number is 4
crane1.number = 4

assert(crane1.sign, "CRANE-4")

---

# Nucleoid marks a hazardous load with a class-level conditional

# There is a Load type
class Load:
    pass

# Any load above class 3 is hazardous
if $Load.class > 3:
    $Load.hazardous = true

# load1 is a Load whose class is 1
load1 = Load()
load1.class = 1

assert(load1.hazardous, null)

# load1's class is 6
load1.class = 6

assert(load1.hazardous, true)

---

# Nucleoid rates a port state inspection with a class-level chain

# There is an Inspection type
class Inspection:
    pass

# detain is "DETAIN", rectify is "RECTIFY", and clear is "CLEAR"
detain = "DETAIN"; rectify = "RECTIFY"; clear = "CLEAR"

# If any inspection's deficiencies is greater than 10, then the inspection's outcome is detain,
# else if the inspection's deficiencies is greater than 3, then the inspection's outcome is rectify,
# else the inspection's outcome is clear
if $Inspection.deficiencies > 10:
    $Inspection.outcome = detain
else if $Inspection.deficiencies > 3:
    $Inspection.outcome = rectify
else:
    $Inspection.outcome = clear

# inspection1 is an Inspection whose deficiencies is 5
inspection1 = Inspection()
inspection1.deficiencies = 5

assert(inspection1.outcome, "RECTIFY")

# rectify is "REMEDY"
rectify = "REMEDY"

assert(inspection1.outcome, "REMEDY")

# inspection1's deficiencies is 15
inspection1.deficiencies = 15

assert(inspection1.outcome, "DETAIN")

---

# Nucleoid counts the calls of a port with a class-level aggregate

# There is a Port type
class Port:
    pass

# There is a Call type
class Call:
    pass

# port1 is a Port
port1 = Port()

# call1 is a Call whose port is port1
call1 = Call()
call1.port = port1

# call2 is a Call whose port is port1
call2 = Call()
call2.port = port1

# Any port's calls is the number of calls whose port is the port
$Port.calls = Call.filter(c => c.port == $Port).length

assert(port1.calls, 2)

# call3 is a Call whose port is port1
call3 = Call()
call3.port = port1

assert(port1.calls, 3)

---

# Nucleoid builds a bill of lading from a property that arrives later

# There is a Bill type
class Bill:
    pass

# bill1 is a Bill
bill1 = Bill()

# bill1's full is "BL" plus bill1's serial
bill1.full = "BL" + bill1.serial

assert(bill1.full, null)

# bill1's serial is "4410"
bill1.serial = "4410"

assert(bill1.full, "BL4410")

---

# Nucleoid reads a master through a vessel reference

# There is a Vessel type
class Vessel:
    pass

# There is a Master type
class Master:
    pass

# vessel1 is a Vessel
vessel1 = Vessel()

# master1 is a Master whose name is "Sousa"
master1 = Master()
master1.name = "Sousa"

# vessel1's master is master1
vessel1.master = master1

# vessel1's commanded is "Master " plus vessel1's master's name
vessel1.commanded = "Master " + vessel1.master.name

assert(vessel1.commanded, "Master Sousa")

# master1's name is "Sousa Pinto"
master1.name = "Sousa Pinto"

assert(vessel1.commanded, "Master Sousa Pinto")

---

# Nucleoid clears a schedule line when a time is deleted

# There is a Schedule type
class Schedule:
    pass

# schedule1 is a Schedule
schedule1 = Schedule()

# schedule1's eta is "0600"
schedule1.eta = "0600"

# schedule1's berth is "T04"
schedule1.berth = "T04"

# schedule1's line is schedule1's eta plus " " plus schedule1's berth
schedule1.line = schedule1.eta + " " + schedule1.berth

assert(schedule1.line, "0600 T04")

# schedule1's eta is deleted
delete schedule1.eta

assert(schedule1.line, null)
assert(schedule1.berth, "T04")

---

# Nucleoid computes a turnaround in a block

# hours is 720
hours = 720

# turnaround is null
turnaround = null

# while in the block, daily is a local variable that is hours divided by 24,
# and turnaround is daily times 24
{
    daily = hours / 24
    turnaround = daily * 24
}

assert(turnaround, 720)

# hours is 960
hours = 960

assert(turnaround, 960)

---

# Nucleoid computes a yard area in a nested block

# side is 25
side = 25

# while in the block, face is a local variable that is side squared,
# and in a nested block, area is face times 2
{
    face = Math.pow(side, 2)
    {
        area = face * 2
    }
}

assert(area, 1250)

---

# Nucleoid raises a demurrage alert in a nested if inside a block

# allowed is 120
allowed = 120

# used is 200
used = 200

# notify is true
notify = true

# while in the block, over is a local variable that is used minus allowed,
# and if over is greater than 50, then alert is notify
{
    over = used - allowed
    if over > 50:
        alert = notify
}

assert(alert, true)

# notify is false
notify = false

assert(alert, false)

---

# Nucleoid picks a towage plan in a nested else inside a block

# beam is 4
beam = 4

# wind is 3
wind = 3

# tugs is "TWO TUGS"
tugs = "TWO TUGS"

# single is "ONE TUG"
single = "ONE TUG"

# while in the block, difficulty is a local variable that is beam times wind,
# and if difficulty is greater than 20, then towage is tugs,
# else towage is single
{
    difficulty = beam * wind
    if difficulty > 20:
        towage = tugs
    else:
        towage = single
}

assert(towage, "ONE TUG")

# single is "SOLO"
single = "SOLO"

assert(towage, "SOLO")

---

# Nucleoid computes a port due in a class-level block

# There is a Call type
class Call:
    pass

# while in the block, levy is a local variable that is any call's tonnage times 5 divided by 100,
# and the call's due is the call's tonnage plus levy
{
    levy = $Call.tonnage * 5 / 100
    $Call.due = $Call.tonnage + levy
}

# call1 is a Call
call1 = Call()

assert(call1.due, null)

# call1's tonnage is 4000
call1.tonnage = 4000

assert(call1.due, 4200)

---

# Nucleoid computes a berth index in a nested class-level block

# There is a Terminal type
class Terminal:
    pass

# while in the block, share is a local variable that is 700 divided by any terminal's berths,
# and in a nested block, the terminal's index is the floor of share times the terminal's cranes
{
    share = 700 / $Terminal.berths
    {
        $Terminal.index = Math.floor(share * $Terminal.cranes)
    }
}

# terminal1 is a Terminal
terminal1 = Terminal()

# terminal1's berths is 3
terminal1.berths = 3

# terminal1's cranes is 4
terminal1.cranes = 4

assert(terminal1.index, 933)

---

# Nucleoid flags a quarantine with an if statement on a property

# There is an Arrival type
class Arrival:
    pass

# arrival1 is an Arrival whose state is "FREE"
arrival1 = Arrival()
arrival1.state = "FREE"

# if arrival1's state is "INFECTED", then arrival1's quarantine is true
if arrival1.state == "INFECTED":
    arrival1.quarantine = true

assert(arrival1.quarantine, null)

# arrival1's state is "INFECTED"
arrival1.state = "INFECTED"

assert(arrival1.quarantine, true)

---

# Nucleoid chooses a lashing note with an else statement on a property

# There is a Deck type
class Deck:
    pass

# deck1 is a Deck whose exposed is false
deck1 = Deck()
deck1.exposed = false

# heavy is "HEAVY WEATHER LASHING"
heavy = "HEAVY WEATHER LASHING"

# usual is "STANDARD LASHING"
usual = "STANDARD LASHING"

# if deck1's exposed is true, then deck1's lashing is heavy,
# else deck1's lashing is usual
if deck1.exposed == true:
    deck1.lashing = heavy
else:
    deck1.lashing = usual

assert(deck1.lashing, "STANDARD LASHING")

# deck1's exposed is true
deck1.exposed = true

assert(deck1.lashing, "HEAVY WEATHER LASHING")

---

# Nucleoid bands a demurrage charge with multiple else if statements on a property

# There is a Charge type
class Charge:
    pass

# charge1 is a Charge whose hours is 10
charge1 = Charge()
charge1.hours = 10

# unit is 50
unit = 50

# if charge1's hours is greater than 72, then charge1's amount is charge1's hours times unit plus 2000,
# else if charge1's hours is greater than 24, then charge1's amount is charge1's hours times unit plus 500,
# else charge1's amount is charge1's hours times unit
if charge1.hours > 72:
    charge1.amount = charge1.hours * unit + 2000
else if charge1.hours > 24:
    charge1.amount = charge1.hours * unit + 500
else:
    charge1.amount = charge1.hours * unit

assert(charge1.amount, 500)

# charge1's hours is 48
charge1.hours = 48

assert(charge1.amount, 2900)

# charge1's hours is 96
charge1.hours = 96

assert(charge1.amount, 6800)

---

# Nucleoid calls a speed function in an assignment

# speed returns the miles divided by the hours
def speed(miles, hours):
    return miles / hours

# passage is 480
passage = 480

# duration is 24
duration = 24

# knots is the result of the speed function call
knots = speed(passage, duration)

assert(knots, 20)

# duration is 48
duration = 48

assert(knots, 10)

---

# Nucleoid updates a levy when its function is redefined

# levy returns the freight times 3 divided by 100
def levy(freight):
    return freight * 3 / 100

# revenue is 20000
revenue = 20000

# duty is the result of the levy function call with revenue
duty = levy(revenue)

assert(duty, 600)

# levy returns the freight times 6 divided by 100
def levy(freight):
    return freight * 6 / 100

assert(duty, 1200)

---

# Nucleoid nests a band lookup inside a due function

# band returns 4 times the grade
def band(grade):
    return grade * 4

# due returns the tonnage times the band of the grade
def due(tonnage, grade):
    return tonnage * band(grade)

# gross is 5
gross = 5

# portGrade is 3
portGrade = 3

# payable is the result of the due function call
payable = due(gross, portGrade)

assert(payable, 60)

---

# Nucleoid finds a berth with the three lambda forms

# berths is a list of 1, 4 and 7
berths = [1, 4, 7]

assert(berths.find(function(berth) { return berth == 7 }), 7)
assert(berths.find(berth => { return berth == 4 }), 4)
assert(berths.find(berth => berth == 1), 1)

---

# Nucleoid filters a fleet by two thresholds

# There is a Ship type,
# which has a tonnage as a number
class Ship(tonnage: int):
    this.tonnage = tonnage

# There are Ships whose tonnages are 5000, 20000 and 60000
Ship(5000); Ship(20000); Ship(60000)

# ceiling is 40000
ceiling = 40000

# floorTonnage is 10000
floorTonnage = 10000

# midrange is Ships whose tonnage is above floorTonnage and below ceiling
midrange = Ship.filter(s => s.tonnage > floorTonnage).filter(s => s.tonnage < ceiling)

assert(midrange.length, 1)
assert(midrange[0].tonnage, 20000)

# floorTonnage is 1000
floorTonnage = 1000

assert(midrange.length, 2)
assert(midrange[0].tonnage, 5000)

---

# Nucleoid maps tonnages into dues

# tonnages is a list of 100, 200 and 300
tonnages = [100, 200, 300]

# perTonne is 3
perTonne = 3

# dues is tonnages mapped to the tonnage times perTonne
dues = tonnages.map(t => t * perTonne)

assert(dues[0], 300)
assert(dues[2], 900)

# perTonne is 5
perTonne = 5

assert(dues[2], 1500)

---

# Nucleoid reduces a set of legs into a total

# legs is a list of 300, 400 and 500
legs = [300, 400, 500]

# total is the sum of legs
total = legs.reduce((sum, leg) => sum + leg, 0)

assert(total, 1200)

# Add 300 to legs
legs.push(300)

assert(total, 1500)

---

# Nucleoid checks whether every container is sealed

# There is a Container type
class Container:
    pass

# container1 is a Container that is sealed
container1 = Container()
container1.sealed = true

# container2 is a Container that is sealed
container2 = Container()
container2.sealed = true

# ready is whether every container is sealed
ready = Container.every(c => c.sealed == true)

assert(ready, true)

# container2 is not sealed
container2.sealed = false

assert(ready, false)

---

# Nucleoid checks whether any vessel is delayed

# There is a Vessel type
class Vessel:
    pass

# vessel1 is a Vessel that is on time
vessel1 = Vessel()
vessel1.delayed = false

# vessel2 is a Vessel that is on time
vessel2 = Vessel()
vessel2.delayed = false

# replan is whether any vessel is delayed
replan = Vessel.some(v => v.delayed == true)

assert(replan, false)

# vessel2 is delayed
vessel2.delayed = true

assert(replan, true)

---

# Nucleoid joins a voyage plan into a single string

# ports is a list of "LIS", "CAS" and "DKR"
ports = ["LIS", "CAS", "DKR"]

# plan is ports joined with " - "
plan = ports.join(" - ")

assert(plan, "LIS - CAS - DKR")

# Add "ABJ" to ports
ports.push("ABJ")

assert(plan, "LIS - CAS - DKR - ABJ")

---

# Nucleoid creates a docket for every working call

# There is a Call type
class Call:
    pass

# call1 is a Call
call1 = Call()

# call2 is a Call that is cancelled
call2 = Call()
call2.cancelled = true

# call3 is a Call
call3 = Call()

# There is a Docket type,
# which has a call as a Call
class Docket(call):
    this.call = call

# Any docket's kind is "DISCHARGE"
$Docket.kind = "DISCHARGE"

# For each call of Call, if the call is not cancelled,
# then there is a Docket whose call is the call
for call of Call:
    if not call.cancelled:
        Docket(call)

assert(Docket.length, 2)
assert(Docket[0].call.id, "call1")
assert(Docket[1].call.id, "call3")
assert(Docket[0].kind, "DISCHARGE")

---

# Nucleoid rolls back a loading if a rule throws

# There is a Loading type
class Loading:
    pass

# If any loading's tonnes is greater than 50000, then throw 'OVER_DEADWEIGHT'
if $Loading.tonnes > 50000:
    throw 'OVER_DEADWEIGHT'

# loading1 is a Loading
loading1 = Loading()

try:
    # loading1's tonnes is 80000
    loading1.tonnes = 80000
catch error:
    assert(error, "OVER_DEADWEIGHT")

assert(loading1.tonnes, null)

---

# Nucleoid refuses a cycle between a rate and a freight

# rate is 12
rate = 12

# freight is rate times 9
freight = rate * 9

assert(freight, 108)

try:
    # rate is freight times 9
    rate = freight * 9
catch error:
    assert(error, TypeError("Circular Dependency"))
```
