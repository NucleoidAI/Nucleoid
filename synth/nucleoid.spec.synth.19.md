# Nucleoid Language Reference - Synthesized Use Cases 19

```

# Nucleoid converts a shelf run through a chain of variables

# millimetres = 90000
millimetres = 90000

# metres is millimetres divided by 1000
metres = millimetres / 1000

# bays is metres divided by 3
bays = metres / 3

assert(metres, 90)
assert(bays, 30)

# millimetres is 120000
millimetres = 120000

assert(metres, 120)
assert(bays, 40)

---

# Nucleoid replaces the source of a loan period

# standard is 28
standard = 28

# extended is 56
extended = 56

# period is standard divided by 2
period = standard / 2

assert(period, 14)

# period is extended divided by 2
period = extended / 2

assert(period, 28)

# extended is 84
extended = 84

assert(period, 42)

---

# Nucleoid raises a holdings count using its own value

# holdings is 40000
holdings = 40000

# holdings is holdings plus 5000
holdings = holdings + 5000

assert(holdings, 45000)

---

# Nucleoid fixes a fine rate with the value property

# rate is 5
rate = 5

# days is 12
days = 12

# owed is rate's value times days
owed = rate.value * days

assert(owed, 60)

# rate is 10
rate = 10

assert(owed, 60)

# days is 24
days = 24

assert(owed, 120)

---

# Nucleoid clears a total when its count is deleted

# count is 900
count = 900

# damaged is count divided by 9
damaged = count / 9

# usable is count minus damaged
usable = count - damaged

assert(usable, 800)

# count is deleted
delete count

assert(damaged, null)
assert(usable, null)

try:
    # count
    count
catch error:
    assert(error, ReferenceError("count is not defined"))

---

# Nucleoid assigns an overdue comparison to a variable

# elapsed is 30
elapsed = 30

# allowed is 21
allowed = 21

# overdue is whether elapsed is greater than allowed
overdue = elapsed > allowed

assert(overdue, true)

# elapsed is 14
elapsed = 14

assert(overdue, false)

---

# Nucleoid builds a shelfmark from a template literal

# classification is "823"
classification = "823"

# cutter = "WOO"
cutter = "WOO"

# shelfmark is the classification and cutter in a template
shelfmark = `${classification}.${cutter}`

assert(shelfmark, "823.WOO")

# cutter is "AUS"
cutter = "AUS"

assert(shelfmark, "823.AUS")

---

# Nucleoid combines membership flags with logical operators

# current is true
current = true

# barred is false
barred = false

assert(current and not barred, true)
assert(current && !barred, true)
assert(barred or current, true)
assert(barred || current, true)

---

# Nucleoid respects parentheses in a capacity formula

# shelves is 2
shelves = 2

# bays is 3
bays = 3

# volumes is 4
volumes = 4

assert(shelves + bays * volumes, 14)
assert((shelves + bays) * volumes, 20)

---

# Nucleoid boxes an archive with the remainder operator

# files is 137
files = 137

# perBox is 25
perBox = 25

# loose is files modulo perBox
loose = files % perBox

assert(loose, 12)

# files is 150
files = 150

assert(loose, 0)

---

# Nucleoid appends a sequence to an accession code

# accession is "ACC-"
accession = "ACC-"

# sequence is 44
sequence = 44

# code is accession plus sequence
code = accession + sequence

assert(code, "ACC-44")

# sequence is 45
sequence = 45

assert(code, "ACC-45")

---

# Nucleoid counts the characters of a catalogue key

# key is "MSS1"
key = "MSS1"

# width is key's length
width = key.length

assert(width, 4)

# key is "MSS123456"
key = "MSS123456"

assert(width, 9)

---

# Nucleoid lowercases a collection name as a dependency

# collection is "RAREBOOKS"
collection = "RAREBOOKS"

# slug is collection lowercased
slug = collection.lower()

assert(slug, "rarebooks")

# collection is "MANUSCRIPTS"
collection = "MANUSCRIPTS"

assert(slug, "manuscripts")

---

# Nucleoid reads the floor letter of a store code

# store is "B12"
store = "B12"

# floor is the character of store at 0
floor = store.charAt(0)

assert(floor, "B")

# store is "C12"
store = "C12"

assert(floor, "C")

---

# Nucleoid rewrites a finding aid path with replace

# aid is "fonds/old/series"
aid = "fonds/old/series"

# revision is "new"
revision = "new"

# current is aid with "old" replaced by revision
current = aid.replace("old", revision)

assert(current, "fonds/new/series")

# revision is "rev2"
revision = "rev2"

assert(current, "fonds/rev2/series")

---

# Nucleoid takes the year from the end of an accession number

# accession is "ACC-1987"
accession = "ACC-1987"

# year is the last four characters of accession
year = accession[-4:]

assert(year, "1987")

# accession is "ACC-1988"
accession = "ACC-1988"

assert(year, "1988")

---

# Nucleoid declares an item type with a constructor

# There is an Item type,
# which has a title as a string
class Item(title: str):
    this.title = title

# item1 is an Item whose title is "Atlas"
item1 = Item("Atlas")

assert(item1, { "id": "item1", "title": "Atlas" })
assert(Item.length, 1)

---

# Nucleoid declares a manuscript as a subtype of an item

# There is an Item type,
# which has a title as a string
class Item(title: str):
    this.title = title

# There is a Manuscript type,
# which is a subtype of Item
# and has a folios as a number
class Manuscript: Item
    def init(title, folios):
        super(title)
        this.title = title
        this.folios = folios

# manuscript1 is a Manuscript whose title is "Hours" and whose folios is 120
manuscript1 = Manuscript("Hours", 120)

assert(manuscript1, { "id": "manuscript1", "title": "Hours", "folios": 120 })

---

# Nucleoid numbers every loan with a class-level rule

# There is a Loan type,
# which has a number as a number
class Loan(number: int):
    this.number = number

# Any loan's reference is "LN-" plus the loan's number
$Loan.reference = "LN-" + $Loan.number

# loan1 is a Loan whose number is 8
loan1 = Loan(8)

assert(loan1.reference, "LN-8")

# loan2 is a Loan whose number is 9
loan2 = Loan(9)

assert(loan2.reference, "LN-9")

---

# Nucleoid replaces a class-level rule on a store

# There is a Store type
class Store:
    pass

# store1 is a Store whose number is 6
store1 = Store()
store1.number = 6

# Any store's sign is "S" plus the store's number
$Store.sign = "S" + $Store.number

assert(store1.sign, "S6")

# Any store's sign is "STORE-" plus the store's number
$Store.sign = "STORE-" + $Store.number

assert(store1.sign, "STORE-6")

# store1's number is 7
store1.number = 7

assert(store1.sign, "STORE-7")

---

# Nucleoid marks a fragile item with a class-level conditional

# There is a Volume type
class Volume:
    pass

# Any volume older than 200 years is fragile
if $Volume.age > 200:
    $Volume.fragile = true

# volume1 is a Volume whose age is 40
volume1 = Volume()
volume1.age = 40

assert(volume1.fragile, null)

# volume1's age is 300
volume1.age = 300

assert(volume1.fragile, true)

---

# Nucleoid grades a condition with a class-level chain

# There is a Record type
class Record:
    pass

# good is "GOOD", fair is "FAIR", and poor is "POOR"
good = "GOOD"; fair = "FAIR"; poor = "POOR"

# If any record's score is greater than 70, then the record's condition is good,
# else if the record's score is greater than 40, then the record's condition is fair,
# else the record's condition is poor
if $Record.score > 70:
    $Record.condition = good
else if $Record.score > 40:
    $Record.condition = fair
else:
    $Record.condition = poor

# record1 is a Record whose score is 55
record1 = Record()
record1.score = 55

assert(record1.condition, "FAIR")

# fair is "STABLE"
fair = "STABLE"

assert(record1.condition, "STABLE")

# record1's score is 85
record1.score = 85

assert(record1.condition, "GOOD")

---

# Nucleoid counts the items of a collection with a class-level aggregate

# There is a Collection type
class Collection:
    pass

# There is an Item type
class Item:
    pass

# collection1 is a Collection
collection1 = Collection()

# item1 is an Item whose collection is collection1
item1 = Item()
item1.collection = collection1

# item2 is an Item whose collection is collection1
item2 = Item()
item2.collection = collection1

# Any collection's holdings is the number of items whose collection is the collection
$Collection.holdings = Item.filter(i => i.collection == $Collection).length

assert(collection1.holdings, 2)

# item3 is an Item whose collection is collection1
item3 = Item()
item3.collection = collection1

assert(collection1.holdings, 3)

---

# Nucleoid builds a barcode from a property that arrives later

# There is a Copy type
class Copy:
    pass

# copy1 is a Copy
copy1 = Copy()

# copy1's barcode is "LIB" plus copy1's serial
copy1.barcode = "LIB" + copy1.serial

assert(copy1.barcode, null)

# copy1's serial is "9911"
copy1.serial = "9911"

assert(copy1.barcode, "LIB9911")

---

# Nucleoid reads a donor through a collection reference

# There is a Collection type
class Collection:
    pass

# There is a Donor type
class Donor:
    pass

# collection1 is a Collection
collection1 = Collection()

# donor1 is a Donor whose name is "Hale"
donor1 = Donor()
donor1.name = "Hale"

# collection1's donor is donor1
collection1.donor = donor1

# collection1's credit is "Gift of " plus collection1's donor's name
collection1.credit = "Gift of " + collection1.donor.name

assert(collection1.credit, "Gift of Hale")

# donor1's name is "Hale Estate"
donor1.name = "Hale Estate"

assert(collection1.credit, "Gift of Hale Estate")

---

# Nucleoid clears a label when a shelfmark is deleted

# There is a Label type
class Label:
    pass

# label1 is a Label
label1 = Label()

# label1's mark is "823"
label1.mark = "823"

# label1's author is "WOO"
label1.author = "WOO"

# label1's text is label1's mark plus " " plus label1's author
label1.text = label1.mark + " " + label1.author

assert(label1.text, "823 WOO")

# label1's mark is deleted
delete label1.mark

assert(label1.text, null)
assert(label1.author, "WOO")

---

# Nucleoid computes a digitisation rate in a block

# pages is 7300
pages = 7300

# rate is null
rate = null

# while in the block, daily is a local variable that is pages divided by 365,
# and rate is daily times 365
{
    daily = pages / 365
    rate = daily * 365
}

assert(rate, 7300)

# pages is 3650
pages = 3650

assert(rate, 3650)

---

# Nucleoid computes a reading room area in a nested block

# side is 14
side = 14

# while in the block, face is a local variable that is side squared,
# and in a nested block, area is face times 2
{
    face = Math.pow(side, 2)
    {
        area = face * 2
    }
}

assert(area, 392)

---

# Nucleoid raises a backlog alert in a nested if inside a block

# received is 500
received = 500

# catalogued is 380
catalogued = 380

# notify is true
notify = true

# while in the block, backlog is a local variable that is received minus catalogued,
# and if backlog is greater than 100, then alert is notify
{
    backlog = received - catalogued
    if backlog > 100:
        alert = notify
}

assert(alert, true)

# notify is false
notify = false

assert(alert, false)

---

# Nucleoid picks a storage level in a nested else inside a block

# rarity is 3
rarity = 3

# fragility is 3
fragility = 3

# vault is "VAULT"
vault = "VAULT"

# open is "OPEN SHELF"
open = "OPEN SHELF"

# while in the block, risk is a local variable that is rarity times fragility,
# and if risk is greater than 20, then storage is vault,
# else storage is open
{
    risk = rarity * fragility
    if risk > 20:
        storage = vault
    else:
        storage = open
}

assert(storage, "OPEN SHELF")

# open is "OPEN"
open = "OPEN"

assert(storage, "OPEN")

---

# Nucleoid computes a conservation quote in a class-level block

# There is a Treatment type
class Treatment:
    pass

# while in the block, materials is a local variable that is any treatment's hours times 20 divided by 100,
# and the treatment's quote is the treatment's hours plus materials
{
    materials = $Treatment.hours * 20 / 100
    $Treatment.quote = $Treatment.hours + materials
}

# treatment1 is a Treatment
treatment1 = Treatment()

assert(treatment1.quote, null)

# treatment1's hours is 50
treatment1.hours = 50

assert(treatment1.quote, 60)

---

# Nucleoid computes a store index in a nested class-level block

# There is a Repository type
class Repository:
    pass

# while in the block, share is a local variable that is 900 divided by any repository's rooms,
# and in a nested block, the repository's index is the floor of share times the repository's ranges
{
    share = 900 / $Repository.rooms
    {
        $Repository.index = Math.floor(share * $Repository.ranges)
    }
}

# repository1 is a Repository
repository1 = Repository()

# repository1's rooms is 7
repository1.rooms = 7

# repository1's ranges is 4
repository1.ranges = 4

assert(repository1.index, 514)

---

# Nucleoid flags a recall with an if statement on a property

# There is a Loan type
class Loan:
    pass

# loan1 is a Loan whose state is "CURRENT"
loan1 = Loan()
loan1.state = "CURRENT"

# if loan1's state is "REQUESTED", then loan1's recall is true
if loan1.state == "REQUESTED":
    loan1.recall = true

assert(loan1.recall, null)

# loan1's state is "REQUESTED"
loan1.state = "REQUESTED"

assert(loan1.recall, true)

---

# Nucleoid chooses a handling note with an else statement on a property

# There is an Item type
class Item:
    pass

# item1 is an Item whose rare is false
item1 = Item()
item1.rare = false

# supervised is "SUPERVISED ACCESS"
supervised = "SUPERVISED ACCESS"

# usual is "OPEN ACCESS"
usual = "OPEN ACCESS"

# if item1's rare is true, then item1's access is supervised,
# else item1's access is usual
if item1.rare == true:
    item1.access = supervised
else:
    item1.access = usual

assert(item1.access, "OPEN ACCESS")

# item1's rare is true
item1.rare = true

assert(item1.access, "SUPERVISED ACCESS")

---

# Nucleoid bands a fine with multiple else if statements on a property

# There is a Fine type
class Fine:
    pass

# fine1 is a Fine whose days is 5
fine1 = Fine()
fine1.days = 5

# unit is 2
unit = 2

# if fine1's days is greater than 60, then fine1's amount is fine1's days times unit plus 50,
# else if fine1's days is greater than 30, then fine1's amount is fine1's days times unit plus 20,
# else fine1's amount is fine1's days times unit
if fine1.days > 60:
    fine1.amount = fine1.days * unit + 50
else if fine1.days > 30:
    fine1.amount = fine1.days * unit + 20
else:
    fine1.amount = fine1.days * unit

assert(fine1.amount, 10)

# fine1's days is 40
fine1.days = 40

assert(fine1.amount, 100)

# fine1's days is 70
fine1.days = 70

assert(fine1.amount, 190)

---

# Nucleoid calls a throughput function in an assignment

# throughput returns the items divided by the hours
def throughput(items, hours):
    return items / hours

# processed is 600
processed = 600

# spent is 6
spent = 6

# perHour is the result of the throughput function call
perHour = throughput(processed, spent)

assert(perHour, 100)

# spent is 12
spent = 12

assert(perHour, 50)

---

# Nucleoid updates a levy when its function is redefined

# levy returns the grant times 5 divided by 100
def levy(grant):
    return grant * 5 / 100

# award is 20000
award = 20000

# admin is the result of the levy function call with award
admin = levy(award)

assert(admin, 1000)

# levy returns the grant times 8 divided by 100
def levy(grant):
    return grant * 8 / 100

assert(admin, 1600)

---

# Nucleoid nests a band lookup inside a cost function

# band returns 5 times the grade
def band(grade):
    return grade * 5

# cost returns the folios times the band of the grade
def cost(folios, grade):
    return folios * band(grade)

# extent is 6
extent = 6

# conditionGrade is 3
conditionGrade = 3

# spend is the result of the cost function call
spend = cost(extent, conditionGrade)

assert(spend, 90)

---

# Nucleoid finds a shelfmark with the three lambda forms

# marks is a list of 100, 200 and 300
marks = [100, 200, 300]

assert(marks.find(function(mark) { return mark == 300 }), 300)
assert(marks.find(mark => { return mark == 200 }), 200)
assert(marks.find(mark => mark == 100), 100)

---

# Nucleoid filters a catalogue by two thresholds

# There is a Volume type,
# which has a folios as a number
class Volume(folios: int):
    this.folios = folios

# There are Volumes whose folios are 50, 150 and 250
Volume(50); Volume(150); Volume(250)

# ceiling is 200
ceiling = 200

# floorFolios is 100
floorFolios = 100

# midrange is Volumes whose folios is above floorFolios and below ceiling
midrange = Volume.filter(v => v.folios > floorFolios).filter(v => v.folios < ceiling)

assert(midrange.length, 1)
assert(midrange[0].folios, 150)

# floorFolios is 20
floorFolios = 20

assert(midrange.length, 2)
assert(midrange[0].folios, 50)

---

# Nucleoid maps folios into scanning times

# folios is a list of 10, 20 and 30
folios = [10, 20, 30]

# perFolio is 2
perFolio = 2

# times is folios mapped to the folio times perFolio
times = folios.map(f => f * perFolio)

assert(times[0], 20)
assert(times[2], 60)

# perFolio is 4
perFolio = 4

assert(times[2], 120)

---

# Nucleoid reduces a set of accessions into a total

# years is a list of 300, 400 and 500
years = [300, 400, 500]

# total is the sum of years
total = years.reduce((sum, year) => sum + year, 0)

assert(total, 1200)

# Add 300 to years
years.push(300)

assert(total, 1500)

---

# Nucleoid checks whether every item is catalogued

# There is an Item type
class Item:
    pass

# item1 is an Item that is catalogued
item1 = Item()
item1.catalogued = true

# item2 is an Item that is catalogued
item2 = Item()
item2.catalogued = true

# complete is whether every item is catalogued
complete = Item.every(i => i.catalogued == true)

assert(complete, true)

# item2 is not catalogued
item2.catalogued = false

assert(complete, false)

---

# Nucleoid checks whether any copy is missing

# There is a Copy type
class Copy:
    pass

# copy1 is a Copy that is present
copy1 = Copy()
copy1.missing = false

# copy2 is a Copy that is present
copy2 = Copy()
copy2.missing = false

# search is whether any copy is missing
search = Copy.some(c => c.missing == true)

assert(search, false)

# copy2 is missing
copy2.missing = true

assert(search, true)

---

# Nucleoid joins a provenance chain into a single string

# owners is a list of "HALE", "REID" and "LIBRARY"
owners = ["HALE", "REID", "LIBRARY"]

# provenance is owners joined with " > "
provenance = owners.join(" > ")

assert(provenance, "HALE > REID > LIBRARY")

# Add "ARCHIVE" to owners
owners.push("ARCHIVE")

assert(provenance, "HALE > REID > LIBRARY > ARCHIVE")

---

# Nucleoid creates a record for every held item

# There is an Item type
class Item:
    pass

# item1 is an Item
item1 = Item()

# item2 is an Item that is withdrawn
item2 = Item()
item2.withdrawn = true

# item3 is an Item
item3 = Item()

# There is a Record type,
# which has an item as an Item
class Record(item):
    this.item = item

# Any record's kind is "MARC"
$Record.kind = "MARC"

# For each item of Item, if the item is not withdrawn,
# then there is a Record whose item is the item
for item of Item:
    if not item.withdrawn:
        Record(item)

assert(Record.length, 2)
assert(Record[0].item.id, "item1")
assert(Record[1].item.id, "item3")
assert(Record[0].kind, "MARC")

---

# Nucleoid rolls back a request if a rule throws

# There is a Request type
class Request:
    pass

# If any request's items is greater than 10, then throw 'TOO_MANY_ITEMS'
if $Request.items > 10:
    throw 'TOO_MANY_ITEMS'

# request1 is a Request
request1 = Request()

try:
    # request1's items is 25
    request1.items = 25
catch error:
    assert(error, "TOO_MANY_ITEMS")

assert(request1.items, null)

---

# Nucleoid refuses a cycle between a rate and a fine

# rate is 3
rate = 3

# fine is rate times 7
fine = rate * 7

assert(fine, 21)

try:
    # rate is fine times 7
    rate = fine * 7
catch error:
    assert(error, TypeError("Circular Dependency"))
```
