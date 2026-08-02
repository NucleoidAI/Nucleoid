# Nucleoid Language Reference - Synthesized Use Cases 15

```

# Nucleoid converts a site area through a chain of variables

# squareMetres is 9000
squareMetres = 9000

# hectares is squareMetres divided by 10000
hectares = squareMetres / 10000

# plots is squareMetres divided by 300
plots = squareMetres / 300

assert(hectares, 0.9)
assert(plots, 30)

# squareMetres is 18000
squareMetres = 18000

assert(hectares, 1.8)
assert(plots, 60)

---

# Nucleoid replaces the source of a labour rate

# skilled is 60
skilled = 60

# general is 30
general = 30

# hourly is skilled divided by 2
hourly = skilled / 2

assert(hourly, 30)

# hourly is general divided by 2
hourly = general / 2

assert(hourly, 15)

# general is 40
general = 40

assert(hourly, 20)

---

# Nucleoid raises a storey count using its own value

# storeys is 12
storeys = 12

# storeys is storeys plus 8
storeys = storeys + 8

assert(storeys, 20)

---

# Nucleoid fixes a material price with the value property

# price is 25
price = 25

# tonnes is 8
tonnes = 8

# quoted is price's value times tonnes
quoted = price.value * tonnes

assert(quoted, 200)

# price is 30
price = 30

assert(quoted, 200)

# tonnes is 12
tonnes = 12

assert(quoted, 300)

---

# Nucleoid clears a estimate when its quantity is deleted

# quantity is 400
quantity = 400

# waste is quantity divided by 10
waste = quantity / 10

# ordered is quantity plus waste
ordered = quantity + waste

assert(ordered, 440)

# quantity is deleted
delete quantity

assert(waste, null)
assert(ordered, null)

try:
    # quantity
    quantity
catch error:
    assert(error, ReferenceError("quantity is not defined"))

---

# Nucleoid assigns an overrun comparison to a variable

# spent is 120
spent = 120

# budget is 100
budget = 100

# overrun is whether spent is greater than budget
overrun = spent > budget

assert(overrun, true)

# spent is 90
spent = 90

assert(overrun, false)

---

# Nucleoid builds a drawing reference from a template literal

# discipline is "STR"
discipline = "STR"

# sheet is 204
sheet = 204

# reference is the discipline and sheet in a template
reference = `${discipline}-DWG-${sheet}`

assert(reference, "STR-DWG-204")

# sheet is 205
sheet = 205

assert(reference, "STR-DWG-205")

---

# Nucleoid combines permit flags with logical operators

# approved is true
approved = true

# expired is false
expired = false

assert(approved and not expired, true)
assert(approved && !expired, true)
assert(expired or approved, true)
assert(expired || approved, true)

---

# Nucleoid respects parentheses in a volume formula

# length is 2
length = 2

# width is 3
width = 3

# depth is 4
depth = 4

assert(length + width * depth, 14)
assert((length + width) * depth, 20)

---

# Nucleoid orders bricks with the remainder operator

# bricks is 1015
bricks = 1015

# perPack is 100
perPack = 100

# loose is bricks modulo perPack
loose = bricks % perPack

assert(loose, 15)

# bricks is 1100
bricks = 1100

assert(loose, 0)

---

# Nucleoid appends a phase number to a project code

# project is "PRJ-"
project = "PRJ-"

# phase is 2
phase = 2

# code is project plus phase
code = project + phase

assert(code, "PRJ-2")

# phase is 3
phase = 3

assert(code, "PRJ-3")

---

# Nucleoid counts the characters of a grid reference

# grid is "A1"
grid = "A1"

# width is grid's length
width = grid.length

assert(width, 2)

# grid is "A1B2C3"
grid = "A1B2C3"

assert(width, 6)

---

# Nucleoid lowercases a trade name as a dependency

# trade is "GROUNDWORKS"
trade = "GROUNDWORKS"

# key is trade lowercased
key = trade.lower()

assert(key, "groundworks")

# trade is "ROOFING"
trade = "ROOFING"

assert(key, "roofing")

---

# Nucleoid reads the block letter of a plot code

# plot is "C14"
plot = "C14"

# block is the character of plot at 0
block = plot.charAt(0)

assert(block, "C")

# plot is "D14"
plot = "D14"

assert(block, "D")

---

# Nucleoid rewrites a specification path with replace

# specification is "spec/old/clause"
specification = "spec/old/clause"

# revision is "rev2"
revision = "rev2"

# current is specification with "old" replaced by revision
current = specification.replace("old", revision)

assert(current, "spec/rev2/clause")

# revision is "rev3"
revision = "rev3"

assert(current, "spec/rev3/clause")

---

# Nucleoid takes the revision from the end of a drawing number

# drawing is "DWG-1001-C"
drawing = "DWG-1001-C"

# revision is the last character of drawing
revision = drawing[-1:]

assert(revision, "C")

# drawing is "DWG-1001-D"
drawing = "DWG-1001-D"

assert(revision, "D")

---

# Nucleoid declares a plot type with a constructor

# There is a Plot type,
# which has a reference as a string
class Plot(reference: str):
    this.reference = reference

# plot1 is a Plot whose reference is "P12"
plot1 = Plot("P12")

assert(plot1, { "id": "plot1", "reference": "P12" })
assert(Plot.length, 1)

---

# Nucleoid declares a house as a subtype of a plot

# There is a Plot type,
# which has a reference as a string
class Plot(reference: str):
    this.reference = reference

# There is a House type,
# which is a subtype of Plot
# and has a bedrooms as a number
class House: Plot
    def init(reference, bedrooms):
        super(reference)
        this.reference = reference
        this.bedrooms = bedrooms

# house1 is a House whose reference is "P13" and whose bedrooms is 4
house1 = House("P13", 4)

assert(house1, { "id": "house1", "reference": "P13", "bedrooms": 4 })

---

# Nucleoid numbers every task with a class-level rule

# There is a Task type,
# which has a number as a number
class Task(number: int):
    this.number = number

# Any task's code is "TK-" plus the task's number
$Task.code = "TK-" + $Task.number

# task1 is a Task whose number is 4
task1 = Task(4)

assert(task1.code, "TK-4")

# task2 is a Task whose number is 5
task2 = Task(5)

assert(task2.code, "TK-5")

---

# Nucleoid replaces a class-level rule on a level

# There is a Level type
class Level:
    pass

# level1 is a Level whose number is 2
level1 = Level()
level1.number = 2

# Any level's sign is "L" plus the level's number
$Level.sign = "L" + $Level.number

assert(level1.sign, "L2")

# Any level's sign is "LEVEL-" plus the level's number
$Level.sign = "LEVEL-" + $Level.number

assert(level1.sign, "LEVEL-2")

# level1's number is 3
level1.number = 3

assert(level1.sign, "LEVEL-3")

---

# Nucleoid marks a deep foundation with a class-level conditional

# There is a Foundation type
class Foundation:
    pass

# Any foundation deeper than 3 metres is piled
if $Foundation.depth > 3:
    $Foundation.piled = true

# foundation1 is a Foundation whose depth is 2
foundation1 = Foundation()
foundation1.depth = 2

assert(foundation1.piled, null)

# foundation1's depth is 6
foundation1.depth = 6

assert(foundation1.piled, true)

---

# Nucleoid classifies a defect with a class-level chain

# There is a Defect type
class Defect:
    pass

# critical is "CRITICAL", major is "MAJOR", and minor is "MINOR"
critical = "CRITICAL"; major = "MAJOR"; minor = "MINOR"

# If any defect's severity is greater than 8, then the defect's grade is critical,
# else if the defect's severity is greater than 4, then the defect's grade is major,
# else the defect's grade is minor
if $Defect.severity > 8:
    $Defect.grade = critical
else if $Defect.severity > 4:
    $Defect.grade = major
else:
    $Defect.grade = minor

# defect1 is a Defect whose severity is 6
defect1 = Defect()
defect1.severity = 6

assert(defect1.grade, "MAJOR")

# major is "SIGNIFICANT"
major = "SIGNIFICANT"

assert(defect1.grade, "SIGNIFICANT")

# defect1's severity is 9
defect1.severity = 9

assert(defect1.grade, "CRITICAL")

---

# Nucleoid counts the tasks of a contractor with a class-level aggregate

# There is a Contractor type
class Contractor:
    pass

# There is a Task type
class Task:
    pass

# contractor1 is a Contractor
contractor1 = Contractor()

# task1 is a Task whose contractor is contractor1
task1 = Task()
task1.contractor = contractor1

# task2 is a Task whose contractor is contractor1
task2 = Task()
task2.contractor = contractor1

# Any contractor's workload is the number of tasks whose contractor is the contractor
$Contractor.workload = Task.filter(t => t.contractor == $Contractor).length

assert(contractor1.workload, 2)

# task3 is a Task whose contractor is contractor1
task3 = Task()
task3.contractor = contractor1

assert(contractor1.workload, 3)

---

# Nucleoid builds a certificate from a property that arrives later

# There is a Certificate type
class Certificate:
    pass

# certificate1 is a Certificate
certificate1 = Certificate()

# certificate1's full is "CC" plus certificate1's serial
certificate1.full = "CC" + certificate1.serial

assert(certificate1.full, null)

# certificate1's serial is "8814"
certificate1.serial = "8814"

assert(certificate1.full, "CC8814")

---

# Nucleoid reads an engineer through a package reference

# There is a Package type
class Package:
    pass

# There is an Engineer type
class Engineer:
    pass

# package1 is a Package
package1 = Package()

# engineer1 is an Engineer whose name is "Petra"
engineer1 = Engineer()
engineer1.name = "Petra"

# package1's engineer is engineer1
package1.engineer = engineer1

# package1's signed is "Signed by " plus package1's engineer's name
package1.signed = "Signed by " + package1.engineer.name

assert(package1.signed, "Signed by Petra")

# engineer1's name is "Petra Novak"
engineer1.name = "Petra Novak"

assert(package1.signed, "Signed by Petra Novak")

---

# Nucleoid clears a snag line when a location is deleted

# There is a Snag type
class Snag:
    pass

# snag1 is a Snag
snag1 = Snag()

# snag1's location is "L2"
snag1.location = "L2"

# snag1's issue is "CRACK"
snag1.issue = "CRACK"

# snag1's line is snag1's location plus " " plus snag1's issue
snag1.line = snag1.location + " " + snag1.issue

assert(snag1.line, "L2 CRACK")

# snag1's location is deleted
delete snag1.location

assert(snag1.line, null)
assert(snag1.issue, "CRACK")

---

# Nucleoid computes a pour volume in a block

# concrete is 480
concrete = 480

# volume is null
volume = null

# while in the block, loads is a local variable that is concrete divided by 8,
# and volume is loads times 8
{
    loads = concrete / 8
    volume = loads * 8
}

assert(volume, 480)

# concrete is 640
concrete = 640

assert(volume, 640)

---

# Nucleoid computes a slab area in a nested block

# side is 15
side = 15

# while in the block, face is a local variable that is side squared,
# and in a nested block, area is face times 2
{
    face = Math.pow(side, 2)
    {
        area = face * 2
    }
}

assert(area, 450)

---

# Nucleoid raises a programme alert in a nested if inside a block

# planned is 200
planned = 200

# complete is 120
complete = 120

# notify is true
notify = true

# while in the block, behind is a local variable that is planned minus complete,
# and if behind is greater than 50, then alert is notify
{
    behind = planned - complete
    if behind > 50:
        alert = notify
}

assert(alert, true)

# notify is false
notify = false

assert(alert, false)

---

# Nucleoid picks a lifting method in a nested else inside a block

# mass is 4
mass = 4

# reach is 3
reach = 3

# crane is "CRANE"
crane = "CRANE"

# manual is "MANUAL"
manual = "MANUAL"

# while in the block, effort is a local variable that is mass times reach,
# and if effort is greater than 20, then method is crane,
# else method is manual
{
    effort = mass * reach
    if effort > 20:
        method = crane
    else:
        method = manual
}

assert(method, "MANUAL")

# manual is "HAND"
manual = "HAND"

assert(method, "HAND")

---

# Nucleoid computes a valuation in a class-level block

# There is a Valuation type
class Valuation:
    pass

# while in the block, retention is a local variable that is any valuation's gross times 5 divided by 100,
# and the valuation's net is the valuation's gross minus retention
{
    retention = $Valuation.gross * 5 / 100
    $Valuation.net = $Valuation.gross - retention
}

# valuation1 is a Valuation
valuation1 = Valuation()

assert(valuation1.net, null)

# valuation1's gross is 20000
valuation1.gross = 20000

assert(valuation1.net, 19000)

---

# Nucleoid computes a crew index in a nested class-level block

# There is a Site type
class Site:
    pass

# while in the block, share is a local variable that is 800 divided by any site's gangs,
# and in a nested block, the site's index is the floor of share times the site's shifts
{
    share = 800 / $Site.gangs
    {
        $Site.index = Math.floor(share * $Site.shifts)
    }
}

# site1 is a Site
site1 = Site()

# site1's gangs is 7
site1.gangs = 7

# site1's shifts is 3
site1.shifts = 3

assert(site1.index, 342)

---

# Nucleoid flags a stop notice with an if statement on a property

# There is an Inspection type
class Inspection:
    pass

# inspection1 is an Inspection whose outcome is "PASS"
inspection1 = Inspection()
inspection1.outcome = "PASS"

# if inspection1's outcome is "FAIL", then inspection1's stopNotice is true
if inspection1.outcome == "FAIL":
    inspection1.stopNotice = true

assert(inspection1.stopNotice, null)

# inspection1's outcome is "FAIL"
inspection1.outcome = "FAIL"

assert(inspection1.stopNotice, true)

---

# Nucleoid chooses a hoarding note with an else statement on a property

# There is a Boundary type
class Boundary:
    pass

# boundary1 is a Boundary whose public is false
boundary1 = Boundary()
boundary1.public = false

# secured is "SOLID HOARDING"
secured = "SOLID HOARDING"

# basic is "MESH FENCE"
basic = "MESH FENCE"

# if boundary1's public is true, then boundary1's note is secured,
# else boundary1's note is basic
if boundary1.public == true:
    boundary1.note = secured
else:
    boundary1.note = basic

assert(boundary1.note, "MESH FENCE")

# boundary1's public is true
boundary1.public = true

assert(boundary1.note, "SOLID HOARDING")

---

# Nucleoid bands a preliminaries charge with multiple else if statements on a property

# There is a Contract type
class Contract:
    pass

# contract1 is a Contract whose weeks is 10
contract1 = Contract()
contract1.weeks = 10

# unit is 100
unit = 100

# if contract1's weeks is greater than 52, then contract1's prelims is contract1's weeks times unit plus 5000,
# else if contract1's weeks is greater than 26, then contract1's prelims is contract1's weeks times unit plus 2000,
# else contract1's prelims is contract1's weeks times unit
if contract1.weeks > 52:
    contract1.prelims = contract1.weeks * unit + 5000
else if contract1.weeks > 26:
    contract1.prelims = contract1.weeks * unit + 2000
else:
    contract1.prelims = contract1.weeks * unit

assert(contract1.prelims, 1000)

# contract1's weeks is 30
contract1.weeks = 30

assert(contract1.prelims, 5000)

# contract1's weeks is 60
contract1.weeks = 60

assert(contract1.prelims, 11000)

---

# Nucleoid calls a coverage function in an assignment

# coverage returns the area divided by the rate
def coverage(area, rate):
    return area / rate

# wall is 240
wall = 240

# perTin is 12
perTin = 12

# tins is the result of the coverage function call
tins = coverage(wall, perTin)

assert(tins, 20)

# perTin is 24
perTin = 24

assert(tins, 10)

---

# Nucleoid updates an overhead when its function is redefined

# overhead returns the cost times 15 divided by 100
def overhead(cost):
    return cost * 15 / 100

# works is 20000
works = 20000

# addition is the result of the overhead function call with works
addition = overhead(works)

assert(addition, 3000)

# overhead returns the cost times 20 divided by 100
def overhead(cost):
    return cost * 20 / 100

assert(addition, 4000)

---

# Nucleoid nests a grade lookup inside a cost function

# grade returns 8 times the class
def grade(band):
    return band * 8

# cost returns the volume times the grade of the band
def cost(volume, band):
    return volume * grade(band)

# pour is 5
pour = 5

# mixBand is 3
mixBand = 3

# spend is the result of the cost function call
spend = cost(pour, mixBand)

assert(spend, 120)

---

# Nucleoid finds a level with the three lambda forms

# levels is a list of 1, 2 and 3
levels = [1, 2, 3]

assert(levels.find(function(level) { return level == 3 }), 3)
assert(levels.find(level => { return level == 2 }), 2)
assert(levels.find(level => level == 1), 1)

---

# Nucleoid filters a plot list by two thresholds

# There is a Plot type,
# which has an area as a number
class Plot(area: int):
    this.area = area

# There are Plots whose areas are 200, 400 and 600
Plot(200); Plot(400); Plot(600)

# ceiling is 500
ceiling = 500

# floorArea is 300
floorArea = 300

# midrange is Plots whose area is above floorArea and below ceiling
midrange = Plot.filter(p => p.area > floorArea).filter(p => p.area < ceiling)

assert(midrange.length, 1)
assert(midrange[0].area, 400)

# floorArea is 100
floorArea = 100

assert(midrange.length, 2)
assert(midrange[0].area, 200)

---

# Nucleoid maps quantities into costs

# quantities is a list of 5, 10 and 15
quantities = [5, 10, 15]

# unitCost is 20
unitCost = 20

# costs is quantities mapped to the quantity times unitCost
costs = quantities.map(q => q * unitCost)

assert(costs[0], 100)
assert(costs[2], 300)

# unitCost is 40
unitCost = 40

assert(costs[2], 600)

---

# Nucleoid reduces a bill of quantities into a total

# items is a list of 500, 1500 and 2000
items = [500, 1500, 2000]

# total is the sum of items
total = items.reduce((sum, item) => sum + item, 0)

assert(total, 4000)

# Add 1000 to items
items.push(1000)

assert(total, 5000)

---

# Nucleoid checks whether every plot is signed off

# There is a Plot type
class Plot:
    pass

# plot1 is a Plot that is signed off
plot1 = Plot()
plot1.signedOff = true

# plot2 is a Plot that is signed off
plot2 = Plot()
plot2.signedOff = true

# handover is whether every plot is signed off
handover = Plot.every(p => p.signedOff == true)

assert(handover, true)

# plot2 is not signed off
plot2.signedOff = false

assert(handover, false)

---

# Nucleoid checks whether any permit has lapsed

# There is a Permit type
class Permit:
    pass

# permit1 is a Permit that is current
permit1 = Permit()
permit1.lapsed = false

# permit2 is a Permit that is current
permit2 = Permit()
permit2.lapsed = false

# blocked is whether any permit has lapsed
blocked = Permit.some(p => p.lapsed == true)

assert(blocked, false)

# permit2 has lapsed
permit2.lapsed = true

assert(blocked, true)

---

# Nucleoid joins a construction sequence into a single string

# stages is a list of "DIG", "POUR" and "FRAME"
stages = ["DIG", "POUR", "FRAME"]

# sequence is stages joined with " then "
sequence = stages.join(" then ")

assert(sequence, "DIG then POUR then FRAME")

# Add "ROOF" to stages
stages.push("ROOF")

assert(sequence, "DIG then POUR then FRAME then ROOF")

---

# Nucleoid creates a permit for every live task

# There is a Task type
class Task:
    pass

# task1 is a Task
task1 = Task()

# task2 is a Task that is suspended
task2 = Task()
task2.suspended = true

# task3 is a Task
task3 = Task()

# There is a Permit type,
# which has a task as a Task
class Permit(task):
    this.task = task

# Any permit's kind is "HOTWORK"
$Permit.kind = "HOTWORK"

# For each task of Task, if the task is not suspended,
# then there is a Permit whose task is the task
for task of Task:
    if not task.suspended:
        Permit(task)

assert(Permit.length, 2)
assert(Permit[0].task.id, "task1")
assert(Permit[1].task.id, "task3")
assert(Permit[0].kind, "HOTWORK")

---

# Nucleoid rolls back a lift if a rule throws

# There is a Lift type
class Lift:
    pass

# If any lift's mass is greater than 5000, then throw 'OVER_CAPACITY'
if $Lift.mass > 5000:
    throw 'OVER_CAPACITY'

# lift1 is a Lift
lift1 = Lift()

try:
    # lift1's mass is 8000
    lift1.mass = 8000
catch error:
    assert(error, "OVER_CAPACITY")

assert(lift1.mass, null)

---

# Nucleoid refuses a cycle between a rate and a cost

# rate is 45
rate = 45

# cost is rate times 8
cost = rate * 8

assert(cost, 360)

try:
    # rate is cost times 8
    rate = cost * 8
catch error:
    assert(error, TypeError("Circular Dependency"))

---

# Nucleoid validates a plot reference with a regular expression

# There is a Record type
class Record:
    pass

# If any record's plot does not match /[A-Z][0-9]{2}/, then throw 'INVALID_PLOT'
if not /[A-Z][0-9]{2}/.test($Record.plot):
    throw 'INVALID_PLOT'

# record1 is a Record
record1 = Record()

assert(record1.plot, null)

try:
    # record1's plot is 'a1'
    record1.plot = 'a1'
catch error:
    assert(error, "INVALID_PLOT")
```
