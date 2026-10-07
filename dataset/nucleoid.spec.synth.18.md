# Nucleoid Language Reference - Synthesized Use Cases 18

```

# Nucleoid converts a race distance through a chain of variables

# metres is 42195
metres = 42195

# kilometres is metres divided by 1000
kilometres = metres / 1000

# laps is metres divided by 400
laps = metres / 400

assert(kilometres, 42.195)
assert(laps, 105.4875)

# metres is 21000
metres = 21000

assert(kilometres, 21)
assert(laps, 52.5)

---

# Nucleoid replaces the source of a squad size

# senior is 30
senior = 30

# youth is 20
youth = 20

# squad is senior divided by 2
squad = senior / 2

assert(squad, 15)

# squad is youth divided by 2
squad = youth / 2

assert(squad, 10)

# youth is 24
youth = 24

assert(squad, 12)

---

# Nucleoid raises a points total using its own value

# points is 48
points = 48

# points is points plus 3
points = points + 3

assert(points, 51)

---

# Nucleoid fixes a ticket price with the value property

# price is 25
price = 25

# seats is 4
seats = 4

# quoted is price's value times seats
quoted = price.value * seats

assert(quoted, 100)

# price is 30
price = 30

assert(quoted, 100)

# seats is 8
seats = 8

assert(quoted, 200)

---

# Nucleoid clears a rating when its score is deleted

# score is 80
score = 80

# bonus is score divided by 8
bonus = score / 8

# rating is score plus bonus
rating = score + bonus

assert(rating, 90)

# score is deleted
delete score

assert(bonus, null)
assert(rating, null)

try:
    # score
    score
catch error:
    assert(error, ReferenceError("score is not defined"))

---

# Nucleoid assigns a record comparison to a variable

# time is 9
time = 9

# record is 10
record = 10

# broken is whether time is less than record
broken = time < record

assert(broken, true)

# time is 11
time = 11

assert(broken, false)

---

# Nucleoid builds a fixture reference from a template literal

# league is "PREM"
league = "PREM"

# round is 12
round = 12

# fixture is the league and round in a template
fixture = `${league}-R${round}`

assert(fixture, "PREM-R12")

# round is 13
round = 13

assert(fixture, "PREM-R13")

---

# Nucleoid combines eligibility flags with logical operators

# registered is true
registered = true

# suspended is false
suspended = false

assert(registered and not suspended, true)
assert(registered && !suspended, true)
assert(suspended or registered, true)
assert(suspended || registered, true)

---

# Nucleoid respects parentheses in a scoring formula

# tries is 2
tries = 2

# goals is 3
goals = 3

# weight is 4
weight = 4

assert(tries + goals * weight, 14)
assert((tries + goals) * weight, 20)

---

# Nucleoid draws heats with the remainder operator

# entrants is 53
entrants = 53

# perHeat is 8
perHeat = 8

# spare is entrants modulo perHeat
spare = entrants % perHeat

assert(spare, 5)

# entrants is 56
entrants = 56

assert(spare, 0)

---

# Nucleoid appends a squad number to a club code

# club is "CLB-"
club = "CLB-"

# number is 9
number = 9

# code is club plus number
code = club + number

assert(code, "CLB-9")

# number is 10
number = 10

assert(code, "CLB-10")

---

# Nucleoid counts the characters of a competition code

# competition is "UCL"
competition = "UCL"

# width is competition's length
width = competition.length

assert(width, 3)

# competition is "UCLQUAL"
competition = "UCLQUAL"

assert(width, 7)

---

# Nucleoid lowercases a discipline name as a dependency

# discipline is "SPRINT"
discipline = "SPRINT"

# key is discipline lowercased
key = discipline.lower()

assert(key, "sprint")

# discipline is "HURDLES"
discipline = "HURDLES"

assert(key, "hurdles")

---

# Nucleoid reads the group letter of a draw code

# draw is "A12"
draw = "A12"

# group is the character of draw at 0
group = draw.charAt(0)

assert(group, "A")

# draw is "B12"
draw = "B12"

assert(group, "B")

---

# Nucleoid rewrites a fixture path with replace

# fixture is "season/old/week"
fixture = "season/old/week"

# revision is "new"
revision = "new"

# current is fixture with "old" replaced by revision
current = fixture.replace("old", revision)

assert(current, "season/new/week")

# revision is "cup"
revision = "cup"

assert(current, "season/cup/week")

---

# Nucleoid takes the season from the end of a squad code

# squadCode is "SQ-2023"
squadCode = "SQ-2023"

# season is the last four characters of squadCode
season = squadCode[-4:]

assert(season, "2023")

# squadCode is "SQ-2024"
squadCode = "SQ-2024"

assert(season, "2024")

---

# Nucleoid declares a player type with a constructor

# There is a Player type,
# which has a name as a string
class Player(name: str):
    this.name = name

# player1 is a Player whose name is "Rio"
player1 = Player("Rio")

assert(player1, { "id": "player1", "name": "Rio" })
assert(Player.length, 1)

---

# Nucleoid declares a keeper as a subtype of a player

# There is a Player type,
# which has a name as a string
class Player(name: str):
    this.name = name

# There is a Keeper type,
# which is a subtype of Player
# and has a saves as a number
class Keeper: Player
    def init(name, saves):
        super(name)
        this.name = name
        this.saves = saves

# keeper1 is a Keeper whose name is "Ana" and whose saves is 40
keeper1 = Keeper("Ana", 40)

assert(keeper1, { "id": "keeper1", "name": "Ana", "saves": 40 })

---

# Nucleoid numbers every match with a class-level rule

# There is a Match type,
# which has a number as a number
class Match(number: int):
    this.number = number

# Any match's reference is "MT-" plus the match's number
$Match.reference = "MT-" + $Match.number

# match1 is a Match whose number is 3
match1 = Match(3)

assert(match1.reference, "MT-3")

# match2 is a Match whose number is 4
match2 = Match(4)

assert(match2.reference, "MT-4")

---

# Nucleoid replaces a class-level rule on a stand

# There is a Stand type
class Stand:
    pass

# stand1 is a Stand whose number is 2
stand1 = Stand()
stand1.number = 2

# Any stand's sign is "T" plus the stand's number
$Stand.sign = "T" + $Stand.number

assert(stand1.sign, "T2")

# Any stand's sign is "STAND-" plus the stand's number
$Stand.sign = "STAND-" + $Stand.number

assert(stand1.sign, "STAND-2")

# stand1's number is 3
stand1.number = 3

assert(stand1.sign, "STAND-3")

---

# Nucleoid marks a suspension with a class-level conditional

# There is a Player type
class Player:
    pass

# Any player with more than 4 cards is suspended
if $Player.cards > 4:
    $Player.suspended = true

# player1 is a Player whose cards is 2
player1 = Player()
player1.cards = 2

assert(player1.suspended, null)

# player1's cards is 6
player1.cards = 6

assert(player1.suspended, true)

---

# Nucleoid seeds an entrant with a class-level chain

# There is an Entrant type
class Entrant:
    pass

# top is "SEEDED", middle is "QUALIFIER", and open is "WILDCARD"
top = "SEEDED"; middle = "QUALIFIER"; open = "WILDCARD"

# If any entrant's ranking is greater than 800, then the entrant's status is top,
# else if the entrant's ranking is greater than 400, then the entrant's status is middle,
# else the entrant's status is open
if $Entrant.ranking > 800:
    $Entrant.status = top
else if $Entrant.ranking > 400:
    $Entrant.status = middle
else:
    $Entrant.status = open

# entrant1 is an Entrant whose ranking is 600
entrant1 = Entrant()
entrant1.ranking = 600

assert(entrant1.status, "QUALIFIER")

# middle is "PRELIM"
middle = "PRELIM"

assert(entrant1.status, "PRELIM")

# entrant1's ranking is 900
entrant1.ranking = 900

assert(entrant1.status, "SEEDED")

---

# Nucleoid counts the players of a club with a class-level aggregate

# There is a Club type
class Club:
    pass

# There is a Player type
class Player:
    pass

# club1 is a Club
club1 = Club()

# player1 is a Player whose club is club1
player1 = Player()
player1.club = club1

# player2 is a Player whose club is club1
player2 = Player()
player2.club = club1

# Any club's squad is the number of players whose club is the club
$Club.squad = Player.filter(p => p.club == $Club).length

assert(club1.squad, 2)

# player3 is a Player whose club is club1
player3 = Player()
player3.club = club1

assert(club1.squad, 3)

---

# Nucleoid builds a licence from a property that arrives later

# There is a Licence type
class Licence:
    pass

# licence1 is a Licence
licence1 = Licence()

# licence1's full is "LC" plus licence1's serial
licence1.full = "LC" + licence1.serial

assert(licence1.full, null)

# licence1's serial is "3320"
licence1.serial = "3320"

assert(licence1.full, "LC3320")

---

# Nucleoid reads a captain through a team reference

# There is a Team type
class Team:
    pass

# There is a Captain type
class Captain:
    pass

# team1 is a Team
team1 = Team()

# captain1 is a Captain whose name is "Nuno"
captain1 = Captain()
captain1.name = "Nuno"

# team1's captain is captain1
team1.captain = captain1

# team1's led is "Led by " plus team1's captain's name
team1.led = "Led by " + team1.captain.name

assert(team1.led, "Led by Nuno")

# captain1's name is "Nuno Reis"
captain1.name = "Nuno Reis"

assert(team1.led, "Led by Nuno Reis")

---

# Nucleoid clears a result line when a score is deleted

# There is a Result type
class Result:
    pass

# result1 is a Result
result1 = Result()

# result1's home is "2"
result1.home = "2"

# result1's away is "1"
result1.away = "1"

# result1's line is result1's home plus "-" plus result1's away
result1.line = result1.home + "-" + result1.away

assert(result1.line, "2-1")

# result1's home is deleted
delete result1.home

assert(result1.line, null)
assert(result1.away, "1")

---

# Nucleoid computes a training load in a block

# weekly is 840
weekly = 840

# load is null
load = null

# while in the block, daily is a local variable that is weekly divided by 7,
# and load is daily times 7
{
    daily = weekly / 7
    load = daily * 7
}

assert(load, 840)

# weekly is 1400
weekly = 1400

assert(load, 1400)

---

# Nucleoid computes a pitch area in a nested block

# side is 11
side = 11

# while in the block, face is a local variable that is side squared,
# and in a nested block, area is face times 6
{
    face = Math.pow(side, 2)
    {
        area = face * 6
    }
}

assert(area, 726)

---

# Nucleoid raises a fitness alert in a nested if inside a block

# target is 300
target = 300

# achieved is 180
achieved = 180

# notify is true
notify = true

# while in the block, shortfall is a local variable that is target minus achieved,
# and if shortfall is greater than 100, then alert is notify
{
    shortfall = target - achieved
    if shortfall > 100:
        alert = notify
}

assert(alert, true)

# notify is false
notify = false

assert(alert, false)

---

# Nucleoid picks a recovery plan in a nested else inside a block

# minutes is 4
minutes = 4

# intensity is 3
intensity = 3

# full is "FULL REST"
full = "FULL REST"

# light is "LIGHT SESSION"
light = "LIGHT SESSION"

# while in the block, strain is a local variable that is minutes times intensity,
# and if strain is greater than 20, then recovery is full,
# else recovery is light
{
    strain = minutes * intensity
    if strain > 20:
        recovery = full
    else:
        recovery = light
}

assert(recovery, "LIGHT SESSION")

# light is "EASY"
light = "EASY"

assert(recovery, "EASY")

---

# Nucleoid computes a gate receipt in a class-level block

# There is a Fixture type
class Fixture:
    pass

# while in the block, levy is a local variable that is any fixture's gate times 10 divided by 100,
# and the fixture's net is the fixture's gate minus levy
{
    levy = $Fixture.gate * 10 / 100
    $Fixture.net = $Fixture.gate - levy
}

# fixture1 is a Fixture
fixture1 = Fixture()

assert(fixture1.net, null)

# fixture1's gate is 50000
fixture1.gate = 50000

assert(fixture1.net, 45000)

---

# Nucleoid computes a schedule index in a nested class-level block

# There is a Season type
class Season:
    pass

# while in the block, share is a local variable that is 380 divided by any season's clubs,
# and in a nested block, the season's index is the floor of share times the season's rounds
{
    share = 380 / $Season.clubs
    {
        $Season.index = Math.floor(share * $Season.rounds)
    }
}

# season1 is a Season
season1 = Season()

# season1's clubs is 7
season1.clubs = 7

# season1's rounds is 3
season1.rounds = 3

assert(season1.index, 162)

---

# Nucleoid flags a review with an if statement on a property

# There is a Decision type
class Decision:
    pass

# decision1 is a Decision whose state is "GIVEN"
decision1 = Decision()
decision1.state = "GIVEN"

# if decision1's state is "CHALLENGED", then decision1's review is true
if decision1.state == "CHALLENGED":
    decision1.review = true

assert(decision1.review, null)

# decision1's state is "CHALLENGED"
decision1.state = "CHALLENGED"

assert(decision1.review, true)

---

# Nucleoid chooses a kit note with an else statement on a property

# There is a Fixture type
class Fixture:
    pass

# fixture1 is a Fixture whose clash is false
fixture1 = Fixture()
fixture1.clash = false

# away is "AWAY KIT"
away = "AWAY KIT"

# home is "HOME KIT"
home = "HOME KIT"

# if fixture1's clash is true, then fixture1's kit is away,
# else fixture1's kit is home
if fixture1.clash == true:
    fixture1.kit = away
else:
    fixture1.kit = home

assert(fixture1.kit, "HOME KIT")

# fixture1's clash is true
fixture1.clash = true

assert(fixture1.kit, "AWAY KIT")

---

# Nucleoid bands a bonus with multiple else if statements on a property

# There is a Contract type
class Contract:
    pass

# contract1 is a Contract whose goals is 5
contract1 = Contract()
contract1.goals = 5

# unit is 1000
unit = 1000

# if contract1's goals is greater than 30, then contract1's bonus is contract1's goals times unit plus 50000,
# else if contract1's goals is greater than 15, then contract1's bonus is contract1's goals times unit plus 10000,
# else contract1's bonus is contract1's goals times unit
if contract1.goals > 30:
    contract1.bonus = contract1.goals * unit + 50000
else if contract1.goals > 15:
    contract1.bonus = contract1.goals * unit + 10000
else:
    contract1.bonus = contract1.goals * unit

assert(contract1.bonus, 5000)

# contract1's goals is 20
contract1.goals = 20

assert(contract1.bonus, 30000)

# contract1's goals is 40
contract1.goals = 40

assert(contract1.bonus, 90000)

---

# Nucleoid calls a strike rate function in an assignment

# strikeRate returns the goals divided by the games
def strikeRate(goals, games):
    return goals / games

# scored is 24
scored = 24

# played is 12
played = 12

# perGame is the result of the strikeRate function call
perGame = strikeRate(scored, played)

assert(perGame, 2)

# played is 24
played = 24

assert(perGame, 1)

---

# Nucleoid updates a levy when its function is redefined

# levy returns the fee times 4 divided by 100
def levy(fee):
    return fee * 4 / 100

# transfer is 500000
transfer = 500000

# solidarity is the result of the levy function call with transfer
solidarity = levy(transfer)

assert(solidarity, 20000)

# levy returns the fee times 5 divided by 100
def levy(fee):
    return fee * 5 / 100

assert(solidarity, 25000)

---

# Nucleoid nests a tier lookup inside a fee function

# tier returns 6 times the level
def tier(level):
    return level * 6

# fee returns the caps times the tier of the level
def fee(caps, level):
    return caps * tier(level)

# appearances is 5
appearances = 5

# playerLevel is 3
playerLevel = 3

# payable is the result of the fee function call
payable = fee(appearances, playerLevel)

assert(payable, 90)

---

# Nucleoid finds a score with the three lambda forms

# scores is a list of 1, 2 and 3
scores = [1, 2, 3]

assert(scores.find(function(score) { return score == 3 }), 3)
assert(scores.find(score => { return score == 2 }), 2)
assert(scores.find(score => score == 1), 1)

---

# Nucleoid filters a squad by two thresholds

# There is a Player type,
# which has an age as a number
class Player(age: int):
    this.age = age

# There are Players whose ages are 18, 26 and 34
Player(18); Player(26); Player(34)

# ceiling is 30
ceiling = 30

# floorAge is 21
floorAge = 21

# prime is Players whose age is above floorAge and below ceiling
prime = Player.filter(p => p.age > floorAge).filter(p => p.age < ceiling)

assert(prime.length, 1)
assert(prime[0].age, 26)

# floorAge is 16
floorAge = 16

assert(prime.length, 2)
assert(prime[0].age, 18)

---

# Nucleoid maps appearances into points

# appearances is a list of 5, 10 and 15
appearances = [5, 10, 15]

# perGame is 3
perGame = 3

# points is appearances mapped to the appearance times perGame
points = appearances.map(a => a * perGame)

assert(points[0], 15)
assert(points[2], 45)

# perGame is 4
perGame = 4

assert(points[2], 60)

---

# Nucleoid reduces a set of scores into a total

# rounds is a list of 20, 30 and 40
rounds = [20, 30, 40]

# total is the sum of rounds
total = rounds.reduce((sum, round) => sum + round, 0)

assert(total, 90)

# Add 10 to rounds
rounds.push(10)

assert(total, 100)

---

# Nucleoid checks whether every player is fit

# There is a Player type
class Player:
    pass

# player1 is a Player who is fit
player1 = Player()
player1.fit = true

# player2 is a Player who is fit
player2 = Player()
player2.fit = true

# available is whether every player is fit
available = Player.every(p => p.fit == true)

assert(available, true)

# player2 is not fit
player2.fit = false

assert(available, false)

---

# Nucleoid checks whether any fixture is postponed

# There is a Fixture type
class Fixture:
    pass

# fixture1 is a Fixture that is on
fixture1 = Fixture()
fixture1.postponed = false

# fixture2 is a Fixture that is on
fixture2 = Fixture()
fixture2.postponed = false

# rearrange is whether any fixture is postponed
rearrange = Fixture.some(f => f.postponed == true)

assert(rearrange, false)

# fixture2 is postponed
fixture2.postponed = true

assert(rearrange, true)

---

# Nucleoid joins a tournament path into a single string

# stages is a list of "GROUP", "QUARTER" and "SEMI"
stages = ["GROUP", "QUARTER", "SEMI"]

# path is stages joined with " > "
path = stages.join(" > ")

assert(path, "GROUP > QUARTER > SEMI")

# Add "FINAL" to stages
stages.push("FINAL")

assert(path, "GROUP > QUARTER > SEMI > FINAL")

---

# Nucleoid creates a team sheet for every played fixture

# There is a Fixture type
class Fixture:
    pass

# fixture1 is a Fixture
fixture1 = Fixture()

# fixture2 is a Fixture that is abandoned
fixture2 = Fixture()
fixture2.abandoned = true

# fixture3 is a Fixture
fixture3 = Fixture()

# There is a Sheet type,
# which has a fixture as a Fixture
class Sheet(fixture):
    this.fixture = fixture

# Any sheet's kind is "LINEUP"
$Sheet.kind = "LINEUP"

# For each fixture of Fixture, if the fixture is not abandoned,
# then there is a Sheet whose fixture is the fixture
for fixture of Fixture:
    if not fixture.abandoned:
        Sheet(fixture)

assert(Sheet.length, 2)
assert(Sheet[0].fixture.id, "fixture1")
assert(Sheet[1].fixture.id, "fixture3")
assert(Sheet[0].kind, "LINEUP")

---

# Nucleoid rolls back a registration if a rule throws

# There is a Registration type
class Registration:
    pass

# If any registration's age is less than 16, then throw 'UNDER_AGE'
if $Registration.age < 16:
    throw 'UNDER_AGE'

# registration1 is a Registration
registration1 = Registration()

try:
    # registration1's age is 14
    registration1.age = 14
catch error:
    assert(error, "UNDER_AGE")

assert(registration1.age, null)

---

# Nucleoid refuses a cycle between a score and a total

# score is 8
score = 8

# total is score times 5
total = score * 5

assert(total, 40)

try:
    # score is total times 5
    score = total * 5
catch error:
    assert(error, TypeError("Circular Dependency"))

---

# Nucleoid validates a licence number with a regular expression

# There is a Licence type
class Licence:
    pass

# If any licence's number does not match /[A-Z][0-9]{5}/, then throw 'INVALID_LICENCE'
if not /[A-Z][0-9]{5}/.test($Licence.number):
    throw 'INVALID_LICENCE'

# licence1 is a Licence
licence1 = Licence()

assert(licence1.number, null)

try:
    # licence1's number is 'A12'
    licence1.number = 'A12'
catch error:
    assert(error, "INVALID_LICENCE")
```
