# Nucleoid Language Reference - Synthesized Use Cases 17

```

# Nucleoid converts a bandwidth through a chain of variables

# bits is 8000000
bits = 8000000

# bytes is bits divided by 8
bytes = bits / 8

# megabytes is bytes divided by 1000000
megabytes = bytes / 1000000

assert(bytes, 1000000)
assert(megabytes, 1)

# bits is 16000000
bits = 16000000

assert(bytes, 2000000)
assert(megabytes, 2)

---

# Nucleoid replaces the source of a tariff

# business is 80
business = 80

# consumer is 40
consumer = 40

# monthly is business divided by 2
monthly = business / 2

assert(monthly, 40)

# monthly is consumer divided by 2
monthly = consumer / 2

assert(monthly, 20)

# consumer is 60
consumer = 60

assert(monthly, 30)

---

# Nucleoid raises a channel count using its own value

# channels is 64
channels = 64

# channels is channels plus 64
channels = channels + 64

assert(channels, 128)

---

# Nucleoid fixes a call rate with the value property

# rate is 3
rate = 3

# minutes is 40
minutes = 40

# charged is rate's value times minutes
charged = rate.value * minutes

assert(charged, 120)

# rate is 5
rate = 5

assert(charged, 120)

# minutes is 80
minutes = 80

assert(charged, 240)

---

# Nucleoid clears a bill when its usage is deleted

# usage is 600
usage = 600

# excess is usage divided by 4
excess = usage / 4

# bill is usage plus excess
bill = usage + excess

assert(bill, 750)

# usage is deleted
delete usage

assert(excess, null)
assert(bill, null)

try:
    # usage
    usage
catch error:
    assert(error, ReferenceError("usage is not defined"))

---

# Nucleoid assigns a congestion comparison to a variable

# load is 900
load = 900

# capacity is 800
capacity = 800

# congested is whether load is greater than capacity
congested = load > capacity

assert(congested, true)

# load is 500
load = 500

assert(congested, false)

---

# Nucleoid builds a circuit reference from a template literal

# exchange is "LDN"
exchange = "LDN"

# circuit is 3308
circuit = 3308

# reference is the exchange and circuit in a template
reference = `${exchange}-CCT-${circuit}`

assert(reference, "LDN-CCT-3308")

# circuit is 3309
circuit = 3309

assert(reference, "LDN-CCT-3309")

---

# Nucleoid combines line flags with logical operators

# active is true
active = true

# barred is false
barred = false

assert(active and not barred, true)
assert(active && !barred, true)
assert(barred or active, true)
assert(barred || active, true)

---

# Nucleoid respects parentheses in a latency formula

# hops is 2
hops = 2

# delay is 3
delay = 3

# links is 4
links = 4

assert(hops + delay * links, 14)
assert((hops + delay) * links, 20)

---

# Nucleoid allocates addresses with the remainder operator

# hosts is 1027
hosts = 1027

# perSubnet is 256
perSubnet = 256

# spare is hosts modulo perSubnet
spare = hosts % perSubnet

assert(spare, 3)

# hosts is 1024
hosts = 1024

assert(spare, 0)

---

# Nucleoid appends a port number to a switch code

# switchCode is "SW-"
switchCode = "SW-"

# port is 24
port = 24

# code is switchCode plus port
code = switchCode + port

assert(code, "SW-24")

# port is 48
port = 48

assert(code, "SW-48")

---

# Nucleoid counts the characters of a network identifier

# identifier is "NET1"
identifier = "NET1"

# width is identifier's length
width = identifier.length

assert(width, 4)

# identifier is "NET123456"
identifier = "NET123456"

assert(width, 9)

---

# Nucleoid lowercases a protocol name as a dependency

# protocol is "HTTPS"
protocol = "HTTPS"

# key is protocol lowercased
key = protocol.lower()

assert(key, "https")

# protocol is "SFTP"
protocol = "SFTP"

assert(key, "sftp")

---

# Nucleoid reads the region letter of an exchange code

# exchange is "N204"
exchange = "N204"

# region is the character of exchange at 0
region = exchange.charAt(0)

assert(region, "N")

# exchange is "S204"
exchange = "S204"

assert(region, "S")

---

# Nucleoid rewrites a route path with replace

# route is "core/old/edge"
route = "core/old/edge"

# revision is "new"
revision = "new"

# current is route with "old" replaced by revision
current = route.replace("old", revision)

assert(current, "core/new/edge")

# revision is "alt"
revision = "alt"

assert(current, "core/alt/edge")

---

# Nucleoid takes the suffix from the end of a subscriber number

# subscriber is "0800-123456"
subscriber = "0800-123456"

# suffix is the last six characters of subscriber
suffix = subscriber[-6:]

assert(suffix, "123456")

# subscriber is "0800-987654"
subscriber = "0800-987654"

assert(suffix, "987654")

---

# Nucleoid declares a line type with a constructor

# There is a Line type,
# which has a number as a string
class Line(number: str):
    this.number = number

# line1 is a Line whose number is "020"
line1 = Line("020")

assert(line1, { "id": "line1", "number": "020" })
assert(Line.length, 1)

---

# Nucleoid declares a trunk as a subtype of a line

# There is a Line type,
# which has a number as a string
class Line(number: str):
    this.number = number

# There is a Trunk type,
# which is a subtype of Line
# and has a channels as a number
class Trunk: Line
    def init(number, channels):
        super(number)
        this.number = number
        this.channels = channels

# trunk1 is a Trunk whose number is "030" and whose channels is 30
trunk1 = Trunk("030", 30)

assert(trunk1, { "id": "trunk1", "number": "030", "channels": 30 })

---

# Nucleoid numbers every ticket with a class-level rule

# There is a Ticket type,
# which has a number as a number
class Ticket(number: int):
    this.number = number

# Any ticket's reference is "INC-" plus the ticket's number
$Ticket.reference = "INC-" + $Ticket.number

# ticket1 is a Ticket whose number is 55
ticket1 = Ticket(55)

assert(ticket1.reference, "INC-55")

# ticket2 is a Ticket whose number is 56
ticket2 = Ticket(56)

assert(ticket2.reference, "INC-56")

---

# Nucleoid replaces a class-level rule on a rack

# There is a Rack type
class Rack:
    pass

# rack1 is a Rack whose number is 8
rack1 = Rack()
rack1.number = 8

# Any rack's sign is "R" plus the rack's number
$Rack.sign = "R" + $Rack.number

assert(rack1.sign, "R8")

# Any rack's sign is "RACK-" plus the rack's number
$Rack.sign = "RACK-" + $Rack.number

assert(rack1.sign, "RACK-8")

# rack1's number is 9
rack1.number = 9

assert(rack1.sign, "RACK-9")

---

# Nucleoid marks a saturated link with a class-level conditional

# There is a Link type
class Link:
    pass

# Any link above 90 percent is saturated
if $Link.utilisation > 90:
    $Link.saturated = true

# link1 is a Link whose utilisation is 40
link1 = Link()
link1.utilisation = 40

assert(link1.saturated, null)

# link1's utilisation is 95
link1.utilisation = 95

assert(link1.saturated, true)

---

# Nucleoid prioritises an incident with a class-level chain

# There is an Incident type
class Incident:
    pass

# p1 is "P1", p2 is "P2", and p3 is "P3"
p1 = "P1"; p2 = "P2"; p3 = "P3"

# If any incident's affected is greater than 1000, then the incident's priority is p1,
# else if the incident's affected is greater than 100, then the incident's priority is p2,
# else the incident's priority is p3
if $Incident.affected > 1000:
    $Incident.priority = p1
else if $Incident.affected > 100:
    $Incident.priority = p2
else:
    $Incident.priority = p3

# incident1 is an Incident whose affected is 400
incident1 = Incident()
incident1.affected = 400

assert(incident1.priority, "P2")

# p2 is "HIGH"
p2 = "HIGH"

assert(incident1.priority, "HIGH")

# incident1's affected is 5000
incident1.affected = 5000

assert(incident1.priority, "P1")

---

# Nucleoid counts the ports of a switch with a class-level aggregate

# There is a Switch type
class Switch:
    pass

# There is a Port type
class Port:
    pass

# switch1 is a Switch
switch1 = Switch()

# port1 is a Port whose switch is switch1
port1 = Port()
port1.switch = switch1

# port2 is a Port whose switch is switch1
port2 = Port()
port2.switch = switch1

# Any switch's ports is the number of ports whose switch is the switch
$Switch.ports = Port.filter(p => p.switch == $Switch).length

assert(switch1.ports, 2)

# port3 is a Port whose switch is switch1
port3 = Port()
port3.switch = switch1

assert(switch1.ports, 3)

---

# Nucleoid builds an order from a property that arrives later

# There is an Order type
class Order:
    pass

# order1 is an Order
order1 = Order()

# order1's full is "ORD" plus order1's serial
order1.full = "ORD" + order1.serial

assert(order1.full, null)

# order1's serial is "7742"
order1.serial = "7742"

assert(order1.full, "ORD7742")

---

# Nucleoid reads an engineer through a job reference

# There is a Job type
class Job:
    pass

# There is an Engineer type
class Engineer:
    pass

# job1 is a Job
job1 = Job()

# engineer1 is an Engineer whose name is "Lena"
engineer1 = Engineer()
engineer1.name = "Lena"

# job1's engineer is engineer1
job1.engineer = engineer1

# job1's assigned is "Assigned to " plus job1's engineer's name
job1.assigned = "Assigned to " + job1.engineer.name

assert(job1.assigned, "Assigned to Lena")

# engineer1's name is "Lena Ferro"
engineer1.name = "Lena Ferro"

assert(job1.assigned, "Assigned to Lena Ferro")

---

# Nucleoid clears an alarm line when a code is deleted

# There is an Alarm type
class Alarm:
    pass

# alarm1 is an Alarm
alarm1 = Alarm()

# alarm1's code is "LOS"
alarm1.code = "LOS"

# alarm1's node is "N12"
alarm1.node = "N12"

# alarm1's line is alarm1's code plus "@" plus alarm1's node
alarm1.line = alarm1.code + "@" + alarm1.node

assert(alarm1.line, "LOS@N12")

# alarm1's code is deleted
delete alarm1.code

assert(alarm1.line, null)
assert(alarm1.node, "N12")

---

# Nucleoid computes a throughput in a block

# daily is 2400
daily = 2400

# throughput is null
throughput = null

# while in the block, hourly is a local variable that is daily divided by 24,
# and throughput is hourly times 24
{
    hourly = daily / 24
    throughput = hourly * 24
}

assert(throughput, 2400)

# daily is 4800
daily = 4800

assert(throughput, 4800)

---

# Nucleoid computes a coverage area in a nested block

# radius is 9
radius = 9

# while in the block, square is a local variable that is radius squared,
# and in a nested block, area is square times 3
{
    square = Math.pow(radius, 2)
    {
        area = square * 3
    }
}

assert(area, 243)

---

# Nucleoid raises a packet loss alert in a nested if inside a block

# sent is 1000
sent = 1000

# received is 940
received = 940

# notify is true
notify = true

# while in the block, lost is a local variable that is sent minus received,
# and if lost is greater than 50, then alert is notify
{
    lost = sent - received
    if lost > 50:
        alert = notify
}

assert(alert, true)

# notify is false
notify = false

assert(alert, false)

---

# Nucleoid picks a routing mode in a nested else inside a block

# hops is 3
hops = 3

# cost is 2
cost = 2

# dynamic is "DYNAMIC"
dynamic = "DYNAMIC"

# static is "STATIC"
static = "STATIC"

# while in the block, weight is a local variable that is hops times cost,
# and if weight is greater than 20, then routing is dynamic,
# else routing is static
{
    weight = hops * cost
    if weight > 20:
        routing = dynamic
    else:
        routing = static
}

assert(routing, "STATIC")

# static is "FIXED"
static = "FIXED"

assert(routing, "FIXED")

---

# Nucleoid computes a monthly charge in a class-level block

# There is a Contract type
class Contract:
    pass

# while in the block, tax is a local variable that is any contract's rental times 20 divided by 100,
# and the contract's gross is the contract's rental plus tax
{
    tax = $Contract.rental * 20 / 100
    $Contract.gross = $Contract.rental + tax
}

# contract1 is a Contract
contract1 = Contract()

assert(contract1.gross, null)

# contract1's rental is 50
contract1.rental = 50

assert(contract1.gross, 60)

---

# Nucleoid computes a capacity index in a nested class-level block

# There is a Node type
class Node:
    pass

# while in the block, share is a local variable that is 1000 divided by any node's links,
# and in a nested block, the node's index is the floor of share times the node's streams
{
    share = 1000 / $Node.links
    {
        $Node.index = Math.floor(share * $Node.streams)
    }
}

# node1 is a Node
node1 = Node()

# node1's links is 7
node1.links = 7

# node1's streams is 3
node1.streams = 3

assert(node1.index, 428)

---

# Nucleoid flags a suspension with an if statement on a property

# There is an Account type
class Account:
    pass

# account1 is an Account whose state is "ACTIVE"
account1 = Account()
account1.state = "ACTIVE"

# if account1's state is "ARREARS", then account1's suspend is true
if account1.state == "ARREARS":
    account1.suspend = true

assert(account1.suspend, null)

# account1's state is "ARREARS"
account1.state = "ARREARS"

assert(account1.suspend, true)

---

# Nucleoid chooses an install note with an else statement on a property

# There is an Install type
class Install:
    pass

# install1 is an Install whose fibre is false
install1 = Install()
install1.fibre = false

# blown is "BLOWN FIBRE"
blown = "BLOWN FIBRE"

# copper is "COPPER PAIR"
copper = "COPPER PAIR"

# if install1's fibre is true, then install1's method is blown,
# else install1's method is copper
if install1.fibre == true:
    install1.method = blown
else:
    install1.method = copper

assert(install1.method, "COPPER PAIR")

# install1's fibre is true
install1.fibre = true

assert(install1.method, "BLOWN FIBRE")

---

# Nucleoid bands an excess charge with multiple else if statements on a property

# There is a Bill type
class Bill:
    pass

# bill1 is a Bill whose gigabytes is 20
bill1 = Bill()
bill1.gigabytes = 20

# unit is 2
unit = 2

# if bill1's gigabytes is greater than 500, then bill1's excess is bill1's gigabytes times unit plus 200,
# else if bill1's gigabytes is greater than 100, then bill1's excess is bill1's gigabytes times unit plus 50,
# else bill1's excess is bill1's gigabytes times unit
if bill1.gigabytes > 500:
    bill1.excess = bill1.gigabytes * unit + 200
else if bill1.gigabytes > 100:
    bill1.excess = bill1.gigabytes * unit + 50
else:
    bill1.excess = bill1.gigabytes * unit

assert(bill1.excess, 40)

# bill1's gigabytes is 200
bill1.gigabytes = 200

assert(bill1.excess, 450)

# bill1's gigabytes is 600
bill1.gigabytes = 600

assert(bill1.excess, 1400)

---

# Nucleoid calls a latency function in an assignment

# latency returns the distance divided by the speed
def latency(distance, speed):
    return distance / speed

# span is 600
span = 600

# rate is 200
rate = 200

# delay is the result of the latency function call
delay = latency(span, rate)

assert(delay, 3)

# rate is 300
rate = 300

assert(delay, 2)

---

# Nucleoid updates a levy when its function is redefined

# levy returns the rental times 5 divided by 100
def levy(rental):
    return rental * 5 / 100

# monthly is 400
monthly = 400

# charge is the result of the levy function call with monthly
charge = levy(monthly)

assert(charge, 20)

# levy returns the rental times 10 divided by 100
def levy(rental):
    return rental * 10 / 100

assert(charge, 40)

---

# Nucleoid nests a class lookup inside a cost function

# class returns 9 times the grade
def band(grade):
    return grade * 9

# cost returns the circuits times the band of the grade
def cost(circuits, grade):
    return circuits * band(grade)

# count is 5
count = 5

# serviceGrade is 2
serviceGrade = 2

# spend is the result of the cost function call
spend = cost(count, serviceGrade)

assert(spend, 90)

---

# Nucleoid finds a port with the three lambda forms

# ports is a list of 22, 80 and 443
ports = [22, 80, 443]

assert(ports.find(function(port) { return port == 443 }), 443)
assert(ports.find(port => { return port == 80 }), 80)
assert(ports.find(port => port == 22), 22)

---

# Nucleoid filters a link list by two thresholds

# There is a Link type,
# which has a speed as a number
class Link(speed: int):
    this.speed = speed

# There are Links whose speeds are 100, 500 and 1000
Link(100); Link(500); Link(1000)

# ceiling is 800
ceiling = 800

# floorSpeed is 200
floorSpeed = 200

# midrange is Links whose speed is above floorSpeed and below ceiling
midrange = Link.filter(l => l.speed > floorSpeed).filter(l => l.speed < ceiling)

assert(midrange.length, 1)
assert(midrange[0].speed, 500)

# floorSpeed is 50
floorSpeed = 50

assert(midrange.length, 2)
assert(midrange[0].speed, 100)

---

# Nucleoid maps usage into charges

# usages is a list of 10, 20 and 30
usages = [10, 20, 30]

# perGig is 3
perGig = 3

# charges is usages mapped to the usage times perGig
charges = usages.map(u => u * perGig)

assert(charges[0], 30)
assert(charges[2], 90)

# perGig is 5
perGig = 5

assert(charges[2], 150)

---

# Nucleoid reduces a set of usages into a total

# months is a list of 200, 300 and 400
months = [200, 300, 400]

# total is the sum of months
total = months.reduce((sum, month) => sum + month, 0)

assert(total, 900)

# Add 100 to months
months.push(100)

assert(total, 1000)

---

# Nucleoid checks whether every node is reachable

# There is a Node type
class Node:
    pass

# node1 is a Node that is reachable
node1 = Node()
node1.reachable = true

# node2 is a Node that is reachable
node2 = Node()
node2.reachable = true

# healthy is whether every node is reachable
healthy = Node.every(n => n.reachable == true)

assert(healthy, true)

# node2 is not reachable
node2.reachable = false

assert(healthy, false)

---

# Nucleoid checks whether any circuit is down

# There is a Circuit type
class Circuit:
    pass

# circuit1 is a Circuit that is up
circuit1 = Circuit()
circuit1.down = false

# circuit2 is a Circuit that is up
circuit2 = Circuit()
circuit2.down = false

# outage is whether any circuit is down
outage = Circuit.some(c => c.down == true)

assert(outage, false)

# circuit2 is down
circuit2.down = true

assert(outage, true)

---

# Nucleoid joins a network path into a single string

# nodes is a list of "CPE", "EDGE" and "CORE"
nodes = ["CPE", "EDGE", "CORE"]

# path is nodes joined with "::"
path = nodes.join("::")

assert(path, "CPE::EDGE::CORE")

# Add "PEER" to nodes
nodes.push("PEER")

assert(path, "CPE::EDGE::CORE::PEER")

---

# Nucleoid creates a task for every open incident

# There is an Incident type
class Incident:
    pass

# incident1 is an Incident
incident1 = Incident()

# incident2 is an Incident that is resolved
incident2 = Incident()
incident2.resolved = true

# incident3 is an Incident
incident3 = Incident()

# There is a Task type,
# which has an incident as an Incident
class Task(incident):
    this.incident = incident

# Any task's kind is "DIAGNOSE"
$Task.kind = "DIAGNOSE"

# For each incident of Incident, if the incident is not resolved,
# then there is a Task whose incident is the incident
for incident of Incident:
    if not incident.resolved:
        Task(incident)

assert(Task.length, 2)
assert(Task[0].incident.id, "incident1")
assert(Task[1].incident.id, "incident3")
assert(Task[0].kind, "DIAGNOSE")

---

# Nucleoid rolls back a provision if a rule throws

# There is a Provision type
class Provision:
    pass

# If any provision's speed is greater than 1000, then throw 'SPEED_UNAVAILABLE'
if $Provision.speed > 1000:
    throw 'SPEED_UNAVAILABLE'

# provision1 is a Provision
provision1 = Provision()

try:
    # provision1's speed is 2000
    provision1.speed = 2000
catch error:
    assert(error, "SPEED_UNAVAILABLE")

assert(provision1.speed, null)

---

# Nucleoid refuses a cycle between a rate and a bill

# rate is 20
rate = 20

# bill is rate times 12
bill = rate * 12

assert(bill, 240)

try:
    # rate is bill times 12
    rate = bill * 12
catch error:
    assert(error, TypeError("Circular Dependency"))

---

# Nucleoid validates a circuit identifier with a regular expression

# There is a Circuit type
class Circuit:
    pass

# If any circuit's identifier does not match /[A-Z]{2}[0-9]{4}/, then throw 'INVALID_CIRCUIT'
if not /[A-Z]{2}[0-9]{4}/.test($Circuit.identifier):
    throw 'INVALID_CIRCUIT'

# circuit1 is a Circuit
circuit1 = Circuit()

assert(circuit1.identifier, null)

try:
    # circuit1's identifier is 'AB12'
    circuit1.identifier = 'AB12'
catch error:
    assert(error, "INVALID_CIRCUIT")
```
