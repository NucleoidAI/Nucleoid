# Nucleoid Language Reference - Synthesized Use Cases 10

```

# Nucleoid converts a run size through a chain of variables

# pieces is 12000
pieces = 12000

# boxes is pieces divided by 100
boxes = pieces / 100

# pallets is boxes divided by 20
pallets = boxes / 20

assert(boxes, 120)
assert(pallets, 6)

# pieces is 24000
pieces = 24000

assert(boxes, 240)
assert(pallets, 12)

---

# Nucleoid replaces the source of a cycle time

# manual is 180
manual = 180

# automated is 60
automated = 60

# cycle is manual divided by 2
cycle = manual / 2

assert(cycle, 90)

# cycle is automated divided by 2
cycle = automated / 2

assert(cycle, 30)

# automated is 90
automated = 90

assert(cycle, 45)

---

# Nucleoid raises an output target using its own value

# target is 800
target = 800

# target is target plus 200
target = target + 200

assert(target, 1000)

---

# Nucleoid fixes a machine rate with the value property

# rate is 20
rate = 20

# hours is 6
hours = 6

# planned is rate's value times hours
planned = rate.value * hours

assert(planned, 120)

# rate is 25
rate = 25

assert(planned, 120)

# hours is 8
hours = 8

assert(planned, 160)

---

# Nucleoid clears a yield when its input is deleted

# input is 500
input = 500

# scrap is input divided by 10
scrap = input / 10

# output is input minus scrap
output = input - scrap

assert(output, 450)

# input is deleted
delete input

assert(scrap, null)
assert(output, null)

try:
    # input
    input
catch error:
    assert(error, ReferenceError("input is not defined"))

---

# Nucleoid assigns a tolerance comparison to a variable

# measured is 12
measured = 12

# nominal is 10
nominal = 10

# outOfSpec is whether measured is greater than nominal
outOfSpec = measured > nominal

assert(outOfSpec, true)

# measured is 9
measured = 9

assert(outOfSpec, false)

---

# Nucleoid builds a batch reference from a template literal

# line is "L2"
line = "L2"

# shift is 3
shift = 3

# batch is the line and shift in a template
batch = `${line}-SHIFT-${shift}`

assert(batch, "L2-SHIFT-3")

# shift is 4
shift = 4

assert(batch, "L2-SHIFT-4")

---

# Nucleoid combines machine flags with logical operators

# running is true
running = true

# faulted is false
faulted = false

assert(running and not faulted, true)
assert(running && !faulted, true)
assert(faulted or running, true)
assert(faulted || running, true)

---

# Nucleoid respects parentheses in a takt formula

# setup is 2
setup = 2

# run is 3
run = 3

# units is 4
units = 4

assert(setup + run * units, 14)
assert((setup + run) * units, 20)

---

# Nucleoid splits a run with the remainder operator

# parts is 97
parts = 97

# perTray is 12
perTray = 12

# odd is parts modulo perTray
odd = parts % perTray

assert(odd, 1)

# parts is 100
parts = 100

assert(odd, 4)

---

# Nucleoid appends a station number to a line code

# lineCode is "LINE-"
lineCode = "LINE-"

# station is 6
station = 6

# code is lineCode plus station
code = lineCode + station

assert(code, "LINE-6")

# station is 7
station = 7

assert(code, "LINE-7")

---

# Nucleoid counts the characters of a part number

# part is "PN12"
part = "PN12"

# width is part's length
width = part.length

assert(width, 4)

# part is "PN123456"
part = "PN123456"

assert(width, 8)

---

# Nucleoid lowercases a process name as a dependency

# process is "MOULDING"
process = "MOULDING"

# key is process lowercased
key = process.lower()

assert(key, "moulding")

# process is "WELDING"
process = "WELDING"

assert(key, "welding")

---

# Nucleoid reads the plant letter of an asset code

# asset is "P204"
asset = "P204"

# plant is the character of asset at 0
plant = asset.charAt(0)

assert(plant, "P")

# asset is "Q204"
asset = "Q204"

assert(plant, "Q")

---

# Nucleoid rewrites a routing path with replace

# routing = "cell/old/step"
routing = "cell/old/step"

# revision is "new"
revision = "new"

# current is routing with "old" replaced by revision
current = routing.replace("old", revision)

assert(current, "cell/new/step")

# revision is "rev2"
revision = "rev2"

assert(current, "cell/rev2/step")

---

# Nucleoid takes the lot digits from the end of a serial

# serial is "SN-2021-0088"
serial = "SN-2021-0088"

# lot is the last four characters of serial
lot = serial[-4:]

assert(lot, "0088")

# serial is "SN-2021-0099"
serial = "SN-2021-0099"

assert(lot, "0099")

---

# Nucleoid declares a machine type with a constructor

# There is a Machine type,
# which has a name as a string
class Machine(name: str):
    this.name = name

# machine1 is a Machine whose name is "Press"
machine1 = Machine("Press")

assert(machine1, { "id": "machine1", "name": "Press" })
assert(Machine.length, 1)

---

# Nucleoid declares a robot as a subtype of a machine

# There is a Machine type,
# which has a name as a string
class Machine(name: str):
    this.name = name

# There is a Robot type,
# which is a subtype of Machine
# and has an axes as a number
class Robot: Machine
    def init(name, axes):
        super(name)
        this.name = name
        this.axes = axes

# robot1 is a Robot whose name is "Arm" and whose axes is 6
robot1 = Robot("Arm", 6)

assert(robot1, { "id": "robot1", "name": "Arm", "axes": 6 })

---

# Nucleoid stamps every unit with a class-level rule

# There is a Unit type,
# which has a number as a number
class Unit(number: int):
    this.number = number

# Any unit's stamp is "U-" plus the unit's number
$Unit.stamp = "U-" + $Unit.number

# unit1 is a Unit whose number is 3
unit1 = Unit(3)

assert(unit1.stamp, "U-3")

# unit2 is a Unit whose number is 4
unit2 = Unit(4)

assert(unit2.stamp, "U-4")

---

# Nucleoid replaces a class-level rule on a cell

# There is a Cell type
class Cell:
    pass

# cell1 is a Cell whose number is 2
cell1 = Cell()
cell1.number = 2

# Any cell's sign is "C" plus the cell's number
$Cell.sign = "C" + $Cell.number

assert(cell1.sign, "C2")

# Any cell's sign is "CELL-" plus the cell's number
$Cell.sign = "CELL-" + $Cell.number

assert(cell1.sign, "CELL-2")

# cell1's number is 3
cell1.number = 3

assert(cell1.sign, "CELL-3")

---

# Nucleoid marks a reject with a class-level conditional

# There is a Part type
class Part:
    pass

# Any part with more than 2 defects is rejected
if $Part.defects > 2:
    $Part.rejected = true

# part1 is a Part whose defects is 1
part1 = Part()
part1.defects = 1

assert(part1.rejected, null)

# part1's defects is 4
part1.defects = 4

assert(part1.rejected, true)

---

# Nucleoid classifies a downtime with a class-level chain

# There is a Stoppage type
class Stoppage:
    pass

# major is "MAJOR", minor is "MINOR", and brief is "BRIEF"
major = "MAJOR"; minor = "MINOR"; brief = "BRIEF"

# If any stoppage's minutes is greater than 60, then the stoppage's kind is major,
# else if the stoppage's minutes is greater than 10, then the stoppage's kind is minor,
# else the stoppage's kind is brief
if $Stoppage.minutes > 60:
    $Stoppage.kind = major
else if $Stoppage.minutes > 10:
    $Stoppage.kind = minor
else:
    $Stoppage.kind = brief

# stoppage1 is a Stoppage whose minutes is 30
stoppage1 = Stoppage()
stoppage1.minutes = 30

assert(stoppage1.kind, "MINOR")

# minor is "SHORT"
minor = "SHORT"

assert(stoppage1.kind, "SHORT")

# stoppage1's minutes is 90
stoppage1.minutes = 90

assert(stoppage1.kind, "MAJOR")

---

# Nucleoid counts the jobs of a machine with a class-level aggregate

# There is a Machine type
class Machine:
    pass

# There is a Job type
class Job:
    pass

# machine1 is a Machine
machine1 = Machine()

# job1 is a Job whose machine is machine1
job1 = Job()
job1.machine = machine1

# job2 is a Job whose machine is machine1
job2 = Job()
job2.machine = machine1

# Any machine's queued is the number of jobs whose machine is the machine
$Machine.queued = Job.filter(j => j.machine == $Machine).length

assert(machine1.queued, 2)

# job3 is a Job whose machine is machine1
job3 = Job()
job3.machine = machine1

assert(machine1.queued, 3)

---

# Nucleoid builds a work order from a property that arrives later

# There is an Order type
class Order:
    pass

# order1 is an Order
order1 = Order()

# order1's full is "WO" plus order1's serial
order1.full = "WO" + order1.serial

assert(order1.full, null)

# order1's serial is "3312"
order1.serial = "3312"

assert(order1.full, "WO3312")

---

# Nucleoid reads an operator through a shift reference

# There is a Shift type
class Shift:
    pass

# There is an Operator type
class Operator:
    pass

# shift1 is a Shift
shift1 = Shift()

# operator1 is an Operator whose name is "Kai"
operator1 = Operator()
operator1.name = "Kai"

# shift1's operator is operator1
shift1.operator = operator1

# shift1's staffed is "Run by " plus shift1's operator's name
shift1.staffed = "Run by " + shift1.operator.name

assert(shift1.staffed, "Run by Kai")

# operator1's name is "Mia"
operator1.name = "Mia"

assert(shift1.staffed, "Run by Mia")

---

# Nucleoid clears an inspection line when a reading is deleted

# There is an Inspection type
class Inspection:
    pass

# inspection1 is an Inspection
inspection1 = Inspection()

# inspection1's gauge is "G1"
inspection1.gauge = "G1"

# inspection1's reading is "0.02"
inspection1.reading = "0.02"

# inspection1's line is inspection1's gauge plus "=" plus inspection1's reading
inspection1.line = inspection1.gauge + "=" + inspection1.reading

assert(inspection1.line, "G1=0.02")

# inspection1's reading is deleted
delete inspection1.reading

assert(inspection1.line, null)
assert(inspection1.gauge, "G1")

---

# Nucleoid computes a throughput in a block

# daily is 960
daily = 960

# throughput is null
throughput = null

# while in the block, hourly is a local variable that is daily divided by 24,
# and throughput is hourly times 24
{
    hourly = daily / 24
    throughput = hourly * 24
}

assert(throughput, 960)

# daily is 1200
daily = 1200

assert(throughput, 1200)

---

# Nucleoid computes a die area in a nested block

# edge is 6
edge = 6

# while in the block, face is a local variable that is edge squared,
# and in a nested block, area is face times 4
{
    face = Math.pow(edge, 2)
    {
        area = face * 4
    }
}

assert(area, 144)

---

# Nucleoid raises a scrap alert in a nested if inside a block

# produced is 500
produced = 500

# good is 420
good = 420

# notify is true
notify = true

# while in the block, scrap is a local variable that is produced minus good,
# and if scrap is greater than 50, then alert is notify
{
    scrap = produced - good
    if scrap > 50:
        alert = notify
}

assert(alert, true)

# notify is false
notify = false

assert(alert, false)

---

# Nucleoid picks a maintenance level in a nested else inside a block

# hours is 4
hours = 4

# load is 3
load = 3

# overhaul is "OVERHAUL"
overhaul = "OVERHAUL"

# routine is "ROUTINE"
routine = "ROUTINE"

# while in the block, wear is a local variable that is hours times load,
# and if wear is greater than 20, then service is overhaul,
# else service is routine
{
    wear = hours * load
    if wear > 20:
        service = overhaul
    else:
        service = routine
}

assert(service, "ROUTINE")

# routine is "STD"
routine = "STD"

assert(service, "STD")

---

# Nucleoid computes a unit cost in a class-level block

# There is a Product type
class Product:
    pass

# while in the block, overhead is a local variable that is any product's cost times 25 divided by 100,
# and the product's price is the product's cost plus overhead
{
    overhead = $Product.cost * 25 / 100
    $Product.price = $Product.cost + overhead
}

# product1 is a Product
product1 = Product()

assert(product1.price, null)

# product1's cost is 80
product1.cost = 80

assert(product1.price, 100)

---

# Nucleoid computes a load index in a nested class-level block

# There is a Press type
class Press:
    pass

# while in the block, share is a local variable that is 500 divided by any press's tools,
# and in a nested block, the press's index is the floor of share times the press's strokes
{
    share = 500 / $Press.tools
    {
        $Press.index = Math.floor(share * $Press.strokes)
    }
}

# press1 is a Press
press1 = Press()

# press1's tools is 3
press1.tools = 3

# press1's strokes is 7
press1.strokes = 7

assert(press1.index, 1166)

---

# Nucleoid flags a quarantine with an if statement on a property

# There is a Lot type
class Lot:
    pass

# lot1 is a Lot whose state is "RELEASED"
lot1 = Lot()
lot1.state = "RELEASED"

# if lot1's state is "HELD", then lot1's quarantine is true
if lot1.state == "HELD":
    lot1.quarantine = true

assert(lot1.quarantine, null)

# lot1's state is "HELD"
lot1.state = "HELD"

assert(lot1.quarantine, true)

---

# Nucleoid chooses a packing note with an else statement on a property

# There is a Crate type
class Crate:
    pass

# crate1 is a Crate whose export is false
crate1 = Crate()
crate1.export = false

# treated is "HEAT TREATED"
treated = "HEAT TREATED"

# plain is "STANDARD"
plain = "STANDARD"

# if crate1's export is true, then crate1's note is treated,
# else crate1's note is plain
if crate1.export == true:
    crate1.note = treated
else:
    crate1.note = plain

assert(crate1.note, "STANDARD")

# crate1's export is true
crate1.export = true

assert(crate1.note, "HEAT TREATED")

---

# Nucleoid bands a tooling charge with multiple else if statements on a property

# There is a Tool type
class Tool:
    pass

# tool1 is a Tool whose strokes is 500
tool1 = Tool()
tool1.strokes = 500

# unit is 2
unit = 2

# if tool1's strokes is greater than 5000, then tool1's charge is tool1's strokes times unit plus 500,
# else if tool1's strokes is greater than 1000, then tool1's charge is tool1's strokes times unit plus 100,
# else tool1's charge is tool1's strokes times unit
if tool1.strokes > 5000:
    tool1.charge = tool1.strokes * unit + 500
else if tool1.strokes > 1000:
    tool1.charge = tool1.strokes * unit + 100
else:
    tool1.charge = tool1.strokes * unit

assert(tool1.charge, 1000)

# tool1's strokes is 2000
tool1.strokes = 2000

assert(tool1.charge, 4100)

# tool1's strokes is 6000
tool1.strokes = 6000

assert(tool1.charge, 12500)

---

# Nucleoid calls an efficiency function in an assignment

# efficiency returns the good divided by the total times 100
def efficiency(good, total):
    return good / total * 100

# passed is 90
passed = 90

# made is 100
made = 100

# rate is the result of the efficiency function call
rate = efficiency(passed, made)

assert(rate, 90)

# passed is 80
passed = 80

assert(rate, 80)

---

# Nucleoid updates a margin when its function is redefined

# margin returns the cost times 20 divided by 100
def margin(cost):
    return cost * 20 / 100

# unitCost is 50
unitCost = 50

# uplift is the result of the margin function call with unitCost
uplift = margin(unitCost)

assert(uplift, 10)

# margin returns the cost times 40 divided by 100
def margin(cost):
    return cost * 40 / 100

assert(uplift, 20)

---

# Nucleoid nests a grade lookup inside a cost function

# grade returns 3 times the tier
def grade(tier):
    return tier * 3

# cost returns the mass times the grade of the tier
def cost(mass, tier):
    return mass * grade(tier)

# billet is 5
billet = 5

# steelTier is 4
steelTier = 4

# spend is the result of the cost function call
spend = cost(billet, steelTier)

assert(spend, 60)

---

# Nucleoid finds a tolerance with the three lambda forms

# tolerances is a list of 1, 2 and 3
tolerances = [1, 2, 3]

assert(tolerances.find(function(tolerance) { return tolerance == 3 }), 3)
assert(tolerances.find(tolerance => { return tolerance == 2 }), 2)
assert(tolerances.find(tolerance => tolerance == 1), 1)

---

# Nucleoid filters a tool crib by two thresholds

# There is a Die type,
# which has a life as a number
class Die(life: int):
    this.life = life

# There are Dies whose lives are 1000, 3000 and 5000
Die(1000); Die(3000); Die(5000)

# ceiling is 4000
ceiling = 4000

# floorLife is 2000
floorLife = 2000

# usable is Dies whose life is above floorLife and below ceiling
usable = Die.filter(d => d.life > floorLife).filter(d => d.life < ceiling)

assert(usable.length, 1)
assert(usable[0].life, 3000)

# floorLife is 500
floorLife = 500

assert(usable.length, 2)
assert(usable[0].life, 1000)

---

# Nucleoid maps run lengths into hours

# runs is a list of 60, 120 and 180
runs = [60, 120, 180]

# perMinute is 2
perMinute = 2

# minutes is runs mapped to the run times perMinute
minutes = runs.map(r => r * perMinute)

assert(minutes[0], 120)
assert(minutes[2], 360)

# perMinute is 3
perMinute = 3

assert(minutes[2], 540)

---

# Nucleoid reduces a shift log into a total

# outputs is a list of 200, 300 and 400
outputs = [200, 300, 400]

# total is the sum of outputs
total = outputs.reduce((sum, output) => sum + output, 0)

assert(total, 900)

# Add 100 to outputs
outputs.push(100)

assert(total, 1000)

---

# Nucleoid checks whether every machine is calibrated

# There is a Machine type
class Machine:
    pass

# machine1 is a Machine that is calibrated
machine1 = Machine()
machine1.calibrated = true

# machine2 is a Machine that is calibrated
machine2 = Machine()
machine2.calibrated = true

# ready is whether every machine is calibrated
ready = Machine.every(m => m.calibrated == true)

assert(ready, true)

# machine2 is not calibrated
machine2.calibrated = false

assert(ready, false)

---

# Nucleoid checks whether any line is stopped

# There is a Line type
class Line:
    pass

# line1 is a Line that is running
line1 = Line()
line1.stopped = false

# line2 is a Line that is running
line2 = Line()
line2.stopped = false

# halted is whether any line is stopped
halted = Line.some(l => l.stopped == true)

assert(halted, false)

# line2 is stopped
line2.stopped = true

assert(halted, true)

---

# Nucleoid joins a routing into a single string

# steps is a list of "CUT", "BEND" and "WELD"
steps = ["CUT", "BEND", "WELD"]

# routing is steps joined with "/"
routing = steps.join("/")

assert(routing, "CUT/BEND/WELD")

# Add "PAINT" to steps
steps.push("PAINT")

assert(routing, "CUT/BEND/WELD/PAINT")

---

# Nucleoid creates a ticket for every open job

# There is a Job type
class Job:
    pass

# job1 is a Job
job1 = Job()

# job2 is a Job that is closed
job2 = Job()
job2.closed = true

# job3 is a Job
job3 = Job()

# There is a Ticket type,
# which has a job as a Job
class Ticket(job):
    this.job = job

# Any ticket's kind is "WORK"
$Ticket.kind = "WORK"

# For each job of Job, if the job is not closed,
# then there is a Ticket whose job is the job
for job of Job:
    if not job.closed:
        Ticket(job)

assert(Ticket.length, 2)
assert(Ticket[0].job.id, "job1")
assert(Ticket[1].job.id, "job3")
assert(Ticket[0].kind, "WORK")

---

# Nucleoid rolls back a batch if a rule throws

# There is a Batch type
class Batch:
    pass

# If any batch's size is greater than 5000, then throw 'BATCH_TOO_LARGE'
if $Batch.size > 5000:
    throw 'BATCH_TOO_LARGE'

# batch1 is a Batch
batch1 = Batch()

try:
    # batch1's size is 9000
    batch1.size = 9000
catch error:
    assert(error, "BATCH_TOO_LARGE")

assert(batch1.size, null)

---

# Nucleoid refuses a cycle between a rate and an output

# rate is 25
rate = 25

# output is rate times 8
output = rate * 8

assert(output, 200)

try:
    # rate is output times 8
    rate = output * 8
catch error:
    assert(error, TypeError("Circular Dependency"))

---

# Nucleoid validates a die code with a regular expression

# There is a Die type
class Die:
    pass

# If any die's code does not match /[A-Z]{2}/, then throw 'INVALID_DIE'
if not /[A-Z]{2}/.test($Die.code):
    throw 'INVALID_DIE'

# die1 is a Die
die1 = Die()

assert(die1.code, null)

try:
    # die1's code is 'a'
    die1.code = 'a'
catch error:
    assert(error, "INVALID_DIE")
```
