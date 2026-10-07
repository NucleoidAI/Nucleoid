# Nucleoid Language Reference - Synthesized Use Cases 11

```

# Nucleoid converts an energy reading through a chain of variables

# watts is 9000
watts = 9000

# kilowatts is watts divided by 1000
kilowatts = watts / 1000

# units is kilowatts times 3
units = kilowatts * 3

assert(kilowatts, 9)
assert(units, 27)

# watts is 12000
watts = 12000

assert(kilowatts, 12)
assert(units, 36)

---

# Nucleoid replaces the source of a tariff band

# peak is 48
peak = 48

# offPeak is 24
offPeak = 24

# unitPrice is peak divided by 2
unitPrice = peak / 2

assert(unitPrice, 24)

# unitPrice is offPeak divided by 2
unitPrice = offPeak / 2

assert(unitPrice, 12)

# offPeak is 30
offPeak = 30

assert(unitPrice, 15)

---

# Nucleoid raises a generation total using its own value

# generated = 400
generated = 400

# generated is generated plus 150
generated = generated + 150

assert(generated, 550)

---

# Nucleoid fixes a standing charge with the value property

# standing is 30
standing = 30

# days is 20
days = 20

# fixed is standing's value times days
fixed = standing.value * days

assert(fixed, 600)

# standing is 40
standing = 40

assert(fixed, 600)

# days is 30
days = 30

assert(fixed, 900)

---

# Nucleoid clears a bill when its reading is deleted

# reading is 600
reading = 600

# usage is reading divided by 2
usage = reading / 2

# bill is usage plus 50
bill = usage + 50

assert(bill, 350)

# reading is deleted
delete reading

assert(usage, null)
assert(bill, null)

try:
    # reading
    reading
catch error:
    assert(error, ReferenceError("reading is not defined"))

---

# Nucleoid assigns a demand comparison to a variable

# demand is 900
demand = 900

# supply is 800
supply = 800

# shortfall is whether demand is greater than supply
shortfall = demand > supply

assert(shortfall, true)

# demand is 700
demand = 700

assert(shortfall, false)

---

# Nucleoid builds a meter reference from a template literal

# region is "NW"
region = "NW"

# meter is 4471
meter = 4471

# reference is the region and meter in a template
reference = `${region}-MTR-${meter}`

assert(reference, "NW-MTR-4471")

# meter is 4472
meter = 4472

assert(reference, "NW-MTR-4472")

---

# Nucleoid combines supply flags with logical operators

# connected is true
connected = true

# isolated is false
isolated = false

assert(connected and not isolated, true)
assert(connected && !isolated, true)
assert(isolated or connected, true)
assert(isolated || connected, true)

---

# Nucleoid respects parentheses in a load formula

# base is 2
base = 2

# peakLoad is 3
peakLoad = 3

# factor is 4
factor = 4

assert(base + peakLoad * factor, 14)
assert((base + peakLoad) * factor, 20)

---

# Nucleoid splits a reading with the remainder operator

# units is 1234
units = 1234

# perBlock is 100
perBlock = 100

# partial is units modulo perBlock
partial = units % perBlock

assert(partial, 34)

# units is 1250
units = 1250

assert(partial, 50)

---

# Nucleoid appends a phase number to a circuit code

# circuit is "CIR-"
circuit = "CIR-"

# phase is 3
phase = 3

# code is circuit plus phase
code = circuit + phase

assert(code, "CIR-3")

# phase is 1
phase = 1

assert(code, "CIR-1")

---

# Nucleoid counts the characters of a supply number

# supply is "MPAN"
supply = "MPAN"

# width is supply's length
width = supply.length

assert(width, 4)

# supply is "MPAN123456"
supply = "MPAN123456"

assert(width, 10)

---

# Nucleoid lowercases a fuel name as a dependency

# fuel is "SOLAR"
fuel = "SOLAR"

# key is fuel lowercased
key = fuel.lower()

assert(key, "solar")

# fuel is "WIND"
fuel = "WIND"

assert(key, "wind")

---

# Nucleoid reads the grid letter of a substation code

# substation is "N44"
substation = "N44"

# grid is the character of substation at 0
grid = substation.charAt(0)

assert(grid, "N")

# substation is "S44"
substation = "S44"

assert(grid, "S")

---

# Nucleoid rewrites a network path with replace

# network is "grid/old/feeder"
network = "grid/old/feeder"

# stage is "new"
stage = "new"

# current is network with "old" replaced by stage
current = network.replace("old", stage)

assert(current, "grid/new/feeder")

# stage is "spare"
stage = "spare"

assert(current, "grid/spare/feeder")

---

# Nucleoid takes the period from the end of a bill code

# billCode is "BILL-2022-Q3"
billCode = "BILL-2022-Q3"

# period is the last two characters of billCode
period = billCode[-2:]

assert(period, "Q3")

# billCode is "BILL-2022-Q4"
billCode = "BILL-2022-Q4"

assert(period, "Q4")

---

# Nucleoid declares a meter type with a constructor

# There is a Meter type,
# which has a serial as a string
class Meter(serial: str):
    this.serial = serial

# meter1 is a Meter whose serial is "M100"
meter1 = Meter("M100")

assert(meter1, { "id": "meter1", "serial": "M100" })
assert(Meter.length, 1)

---

# Nucleoid declares a smart meter as a subtype of a meter

# There is a Meter type,
# which has a serial as a string
class Meter(serial: str):
    this.serial = serial

# There is a Smart type,
# which is a subtype of Meter
# and has an interval as a number
class Smart: Meter
    def init(serial, interval):
        super(serial)
        this.serial = serial
        this.interval = interval

# smart1 is a Smart whose serial is "S200" and whose interval is 30
smart1 = Smart("S200", 30)

assert(smart1, { "id": "smart1", "serial": "S200", "interval": 30 })

---

# Nucleoid labels every feeder with a class-level rule

# There is a Feeder type,
# which has a number as a number
class Feeder(number: int):
    this.number = number

# Any feeder's label is "FD-" plus the feeder's number
$Feeder.label = "FD-" + $Feeder.number

# feeder1 is a Feeder whose number is 5
feeder1 = Feeder(5)

assert(feeder1.label, "FD-5")

# feeder2 is a Feeder whose number is 6
feeder2 = Feeder(6)

assert(feeder2.label, "FD-6")

---

# Nucleoid replaces a class-level rule on a substation

# There is a Substation type
class Substation:
    pass

# substation1 is a Substation whose number is 7
substation1 = Substation()
substation1.number = 7

# Any substation's sign is "S" plus the substation's number
$Substation.sign = "S" + $Substation.number

assert(substation1.sign, "S7")

# Any substation's sign is "SUB-" plus the substation's number
$Substation.sign = "SUB-" + $Substation.number

assert(substation1.sign, "SUB-7")

# substation1's number is 8
substation1.number = 8

assert(substation1.sign, "SUB-8")

---

# Nucleoid marks an overload with a class-level conditional

# There is a Circuit type
class Circuit:
    pass

# Any circuit above 100 amps is overloaded
if $Circuit.amps > 100:
    $Circuit.overloaded = true

# circuit1 is a Circuit whose amps is 60
circuit1 = Circuit()
circuit1.amps = 60

assert(circuit1.overloaded, null)

# circuit1's amps is 140
circuit1.amps = 140

assert(circuit1.overloaded, true)

---

# Nucleoid rates a property with a class-level chain

# There is a Building type
class Building:
    pass

# good is "A", middling is "C", and poor is "E"
good = "A"; middling = "C"; poor = "E"

# If any building's rating is greater than 80, then the building's band is good,
# else if the building's rating is greater than 50, then the building's band is middling,
# else the building's band is poor
if $Building.rating > 80:
    $Building.band = good
else if $Building.rating > 50:
    $Building.band = middling
else:
    $Building.band = poor

# building1 is a Building whose rating is 65
building1 = Building()
building1.rating = 65

assert(building1.band, "C")

# middling is "AVERAGE"
middling = "AVERAGE"

assert(building1.band, "AVERAGE")

# building1's rating is 90
building1.rating = 90

assert(building1.band, "A")

---

# Nucleoid counts the meters of a site with a class-level aggregate

# There is a Site type
class Site:
    pass

# There is a Meter type
class Meter:
    pass

# site1 is a Site
site1 = Site()

# meter1 is a Meter whose site is site1
meter1 = Meter()
meter1.site = site1

# meter2 is a Meter whose site is site1
meter2 = Meter()
meter2.site = site1

# Any site's meters is the number of meters whose site is the site
$Site.meters = Meter.filter(m => m.site == $Site).length

assert(site1.meters, 2)

# meter3 is a Meter whose site is site1
meter3 = Meter()
meter3.site = site1

assert(site1.meters, 3)

---

# Nucleoid builds a connection from a property that arrives later

# There is a Connection type
class Connection:
    pass

# connection1 is a Connection
connection1 = Connection()

# connection1's full is "CN" plus connection1's serial
connection1.full = "CN" + connection1.serial

assert(connection1.full, null)

# connection1's serial is "7781"
connection1.serial = "7781"

assert(connection1.full, "CN7781")

---

# Nucleoid reads a supplier through a contract reference

# There is a Contract type
class Contract:
    pass

# There is a Supplier type
class Supplier:
    pass

# contract1 is a Contract
contract1 = Contract()

# supplier1 is a Supplier whose name is "Helios"
supplier1 = Supplier()
supplier1.name = "Helios"

# contract1's supplier is supplier1
contract1.supplier = supplier1

# contract1's held is "Supplied by " plus contract1's supplier's name
contract1.held = "Supplied by " + contract1.supplier.name

assert(contract1.held, "Supplied by Helios")

# supplier1's name is "Aurora"
supplier1.name = "Aurora"

assert(contract1.held, "Supplied by Aurora")

---

# Nucleoid clears a reading line when a value is deleted

# There is a Reading type
class Reading:
    pass

# reading1 is a Reading
reading1 = Reading()

# reading1's register is "R1"
reading1.register = "R1"

# reading1's units is "3400"
reading1.units = "3400"

# reading1's line is reading1's register plus ":" plus reading1's units
reading1.line = reading1.register + ":" + reading1.units

assert(reading1.line, "R1:3400")

# reading1's units is deleted
delete reading1.units

assert(reading1.line, null)
assert(reading1.register, "R1")

---

# Nucleoid computes a daily average in a block

# annual = 7300
annual = 7300

# daily is null
daily = null

# while in the block, weekly is a local variable that is annual divided by 365,
# and daily is weekly times 1
{
    weekly = annual / 365
    daily = weekly * 1
}

assert(daily, 20)

# annual is 3650
annual = 3650

assert(daily, 10)

---

# Nucleoid computes a panel area in a nested block

# side is 5
side = 5

# while in the block, face is a local variable that is side squared,
# and in a nested block, array is face times 8
{
    face = Math.pow(side, 2)
    {
        array = face * 8
    }
}

assert(array, 200)

---

# Nucleoid raises a leak alert in a nested if inside a block

# metered is 800
metered = 800

# billed is 700
billed = 700

# notify is true
notify = true

# while in the block, gap is a local variable that is metered minus billed,
# and if gap is greater than 50, then alert is notify
{
    gap = metered - billed
    if gap > 50:
        alert = notify
}

assert(alert, true)

# notify is false
notify = false

assert(alert, false)

---

# Nucleoid picks an inspection level in a nested else inside a block

# age is 3
age = 3

# risk is 3
risk = 3

# full is "FULL"
full = "FULL"

# visual is "VISUAL"
visual = "VISUAL"

# while in the block, score is a local variable that is age times risk,
# and if score is greater than 20, then inspection is full,
# else inspection is visual
{
    score = age * risk
    if score > 20:
        inspection = full
    else:
        inspection = visual
}

assert(inspection, "VISUAL")

# visual is "V"
visual = "V"

assert(inspection, "V")

---

# Nucleoid computes a bill in a class-level block

# There is an Account type
class Account:
    pass

# while in the block, levy is a local variable that is any account's usage times 5 divided by 100,
# and the account's bill is the account's usage plus levy
{
    levy = $Account.usage * 5 / 100
    $Account.bill = $Account.usage + levy
}

# account1 is an Account
account1 = Account()

assert(account1.bill, null)

# account1's usage is 400
account1.usage = 400

assert(account1.bill, 420)

---

# Nucleoid computes a capacity index in a nested class-level block

# There is a Plant type
class Plant:
    pass

# while in the block, share is a local variable that is 700 divided by any plant's turbines,
# and in a nested block, the plant's index is the floor of share times the plant's hours
{
    share = 700 / $Plant.turbines
    {
        $Plant.index = Math.floor(share * $Plant.hours)
    }
}

# plant1 is a Plant
plant1 = Plant()

# plant1's turbines is 3
plant1.turbines = 3

# plant1's hours is 5
plant1.hours = 5

assert(plant1.index, 1166)

---

# Nucleoid flags an outage with an if statement on a property

# There is a Feeder type
class Feeder:
    pass

# feeder1 is a Feeder whose state is "LIVE"
feeder1 = Feeder()
feeder1.state = "LIVE"

# if feeder1's state is "DEAD", then feeder1's outage is true
if feeder1.state == "DEAD":
    feeder1.outage = true

assert(feeder1.outage, null)

# feeder1's state is "DEAD"
feeder1.state = "DEAD"

assert(feeder1.outage, true)

---

# Nucleoid chooses a metering note with an else statement on a property

# There is a Site type
class Site:
    pass

# site1 is a Site whose halfHourly is false
site1 = Site()
site1.halfHourly = false

# detailed is "HALF HOURLY"
detailed = "HALF HOURLY"

# simple is "MONTHLY"
simple = "MONTHLY"

# if site1's halfHourly is true, then site1's metering is detailed,
# else site1's metering is simple
if site1.halfHourly == true:
    site1.metering = detailed
else:
    site1.metering = simple

assert(site1.metering, "MONTHLY")

# site1's halfHourly is true
site1.halfHourly = true

assert(site1.metering, "HALF HOURLY")

---

# Nucleoid bands a connection charge with multiple else if statements on a property

# There is a Job type
class Job:
    pass

# job1 is a Job whose metres is 20
job1 = Job()
job1.metres = 20

# unit is 5
unit = 5

# if job1's metres is greater than 200, then job1's charge is job1's metres times unit plus 400,
# else if job1's metres is greater than 50, then job1's charge is job1's metres times unit plus 100,
# else job1's charge is job1's metres times unit
if job1.metres > 200:
    job1.charge = job1.metres * unit + 400
else if job1.metres > 50:
    job1.charge = job1.metres * unit + 100
else:
    job1.charge = job1.metres * unit

assert(job1.charge, 100)

# job1's metres is 100
job1.metres = 100

assert(job1.charge, 600)

# job1's metres is 300
job1.metres = 300

assert(job1.charge, 1900)

---

# Nucleoid calls a load factor function in an assignment

# factor returns the used divided by the peak times 100
def factor(used, peak):
    return used / peak * 100

# consumed is 60
consumed = 60

# maximum is 100
maximum = 100

# loadFactor is the result of the factor function call
loadFactor = factor(consumed, maximum)

assert(loadFactor, 60)

# consumed is 80
consumed = 80

assert(loadFactor, 80)

---

# Nucleoid updates a levy when its function is redefined

# levy returns the units times 3 divided by 100
def levy(units):
    return units * 3 / 100

# consumption is 2000
consumption = 2000

# green is the result of the levy function call with consumption
green = levy(consumption)

assert(green, 60)

# levy returns the units times 6 divided by 100
def levy(units):
    return units * 6 / 100

assert(green, 120)

---

# Nucleoid nests a rate lookup inside a bill function

# rate returns 6 times the band
def rate(band):
    return band * 6

# bill returns the units times the rate of the band
def bill(units, band):
    return units * rate(band)

# consumed is 5
consumed = 5

# tariffBand is 2
tariffBand = 2

# amount is the result of the bill function call
amount = bill(consumed, tariffBand)

assert(amount, 60)

---

# Nucleoid finds a reading with the three lambda forms

# registers is a list of 100, 200 and 300
registers = [100, 200, 300]

assert(registers.find(function(register) { return register == 300 }), 300)
assert(registers.find(register => { return register == 200 }), 200)
assert(registers.find(register => register == 100), 100)

---

# Nucleoid filters a fleet of turbines by two thresholds

# There is a Turbine type,
# which has an output as a number
class Turbine(output: int):
    this.output = output

# There are Turbines whose outputs are 200, 500 and 800
Turbine(200); Turbine(500); Turbine(800)

# ceiling is 700
ceiling = 700

# floorOutput is 300
floorOutput = 300

# midrange is Turbines whose output is above floorOutput and below ceiling
midrange = Turbine.filter(t => t.output > floorOutput).filter(t => t.output < ceiling)

assert(midrange.length, 1)
assert(midrange[0].output, 500)

# floorOutput is 100
floorOutput = 100

assert(midrange.length, 2)
assert(midrange[0].output, 200)

---

# Nucleoid maps readings into costs

# readings is a list of 10, 20 and 30
readings = [10, 20, 30]

# unitCost is 4
unitCost = 4

# costs is readings mapped to the reading times unitCost
costs = readings.map(r => r * unitCost)

assert(costs[0], 40)
assert(costs[2], 120)

# unitCost is 6
unitCost = 6

assert(costs[2], 180)

---

# Nucleoid reduces a set of readings into a total

# quarters is a list of 300, 400 and 500
quarters = [300, 400, 500]

# annual is the sum of quarters
annual = quarters.reduce((sum, quarter) => sum + quarter, 0)

assert(annual, 1200)

# Add 300 to quarters
quarters.push(300)

assert(annual, 1500)

---

# Nucleoid checks whether every meter is read

# There is a Meter type
class Meter:
    pass

# meter1 is a Meter that is read
meter1 = Meter()
meter1.read = true

# meter2 is a Meter that is read
meter2 = Meter()
meter2.read = true

# complete is whether every meter is read
complete = Meter.every(m => m.read == true)

assert(complete, true)

# meter2 is not read
meter2.read = false

assert(complete, false)

---

# Nucleoid checks whether any feeder is faulted

# There is a Feeder type
class Feeder:
    pass

# feeder1 is a Feeder that is healthy
feeder1 = Feeder()
feeder1.faulted = false

# feeder2 is a Feeder that is healthy
feeder2 = Feeder()
feeder2.faulted = false

# tripped is whether any feeder is faulted
tripped = Feeder.some(f => f.faulted == true)

assert(tripped, false)

# feeder2 is faulted
feeder2.faulted = true

assert(tripped, true)

---

# Nucleoid joins a network route into a single string

# nodes is a list of "GEN", "SUB" and "LOAD"
nodes = ["GEN", "SUB", "LOAD"]

# route is nodes joined with "->"
route = nodes.join("->")

assert(route, "GEN->SUB->LOAD")

# Add "METER" to nodes
nodes.push("METER")

assert(route, "GEN->SUB->LOAD->METER")

---

# Nucleoid creates a visit for every live connection

# There is a Connection type
class Connection:
    pass

# connection1 is a Connection
connection1 = Connection()

# connection2 is a Connection that is disconnected
connection2 = Connection()
connection2.disconnected = true

# connection3 is a Connection
connection3 = Connection()

# There is a Visit type,
# which has a connection as a Connection
class Visit(connection):
    this.connection = connection

# Any visit's kind is "SURVEY"
$Visit.kind = "SURVEY"

# For each connection of Connection, if the connection is not disconnected,
# then there is a Visit whose connection is the connection
for connection of Connection:
    if not connection.disconnected:
        Visit(connection)

assert(Visit.length, 2)
assert(Visit[0].connection.id, "connection1")
assert(Visit[1].connection.id, "connection3")
assert(Visit[0].kind, "SURVEY")

---

# Nucleoid rolls back a switch if a rule throws

# There is a Switch type
class Switch:
    pass

# If any switch's amps is greater than 400, then throw 'RATING_EXCEEDED'
if $Switch.amps > 400:
    throw 'RATING_EXCEEDED'

# switch1 is a Switch
switch1 = Switch()

try:
    # switch1's amps is 600
    switch1.amps = 600
catch error:
    assert(error, "RATING_EXCEEDED")

assert(switch1.amps, null)

---

# Nucleoid refuses a cycle between a load and a demand

# load is 30
load = 30

# demand is load times 6
demand = load * 6

assert(demand, 180)

try:
    # load is demand times 6
    load = demand * 6
catch error:
    assert(error, TypeError("Circular Dependency"))

---

# Nucleoid validates a meter point with a regular expression

# There is a Point type
class Point:
    pass

# If any point's mpan does not match /[0-9]{8}/, then throw 'INVALID_MPAN'
if not /[0-9]{8}/.test($Point.mpan):
    throw 'INVALID_MPAN'

# point1 is a Point
point1 = Point()

assert(point1.mpan, null)

try:
    # point1's mpan is '1234'
    point1.mpan = '1234'
catch error:
    assert(error, "INVALID_MPAN")
```
