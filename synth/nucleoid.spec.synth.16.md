# Nucleoid Language Reference - Synthesized Use Cases 16

```

# Nucleoid converts a page count through a chain of variables

# words is 90000
words = 90000

# pages is words divided by 300
pages = words / 300

# signatures is pages divided by 16
signatures = pages / 16

assert(pages, 300)
assert(signatures, 18.75)

# words is 120000
words = 120000

assert(pages, 400)
assert(signatures, 25)

---

# Nucleoid replaces the source of a royalty rate

# hardback is 20
hardback = 20

# paperback is 10
paperback = 10

# royalty is hardback divided by 2
royalty = hardback / 2

assert(royalty, 10)

# royalty is paperback divided by 2
royalty = paperback / 2

assert(royalty, 5)

# paperback is 16
paperback = 16

assert(royalty, 8)

---

# Nucleoid raises a print run using its own value

# run is 3000
run = 3000

# run is run plus 2000
run = run + 2000

assert(run, 5000)

---

# Nucleoid fixes a page rate with the value property

# rate is 12
rate = 12

# pages is 20
pages = 20

# quoted is rate's value times pages
quoted = rate.value * pages

assert(quoted, 240)

# rate is 15
rate = 15

assert(quoted, 240)

# pages is 40
pages = 40

assert(quoted, 480)

---

# Nucleoid clears a budget when its fee is deleted

# fee is 800
fee = 800

# agent is fee divided by 10
agent = fee / 10

# net is fee minus agent
net = fee - agent

assert(net, 720)

# fee is deleted
delete fee

assert(agent, null)
assert(net, null)

try:
    # fee
    fee
catch error:
    assert(error, ReferenceError("fee is not defined"))

---

# Nucleoid assigns an overrun comparison to a variable

# submitted is 320
submitted = 320

# commissioned is 300
commissioned = 300

# long is whether submitted is greater than commissioned
long = submitted > commissioned

assert(long, true)

# submitted is 280
submitted = 280

assert(long, false)

---

# Nucleoid builds an issue reference from a template literal

# title is "quarterly"
title = "quarterly"

# issue is 42
issue = 42

# reference is the title and issue in a template
reference = `${title}-issue-${issue}`

assert(reference, "quarterly-issue-42")

# issue is 43
issue = 43

assert(reference, "quarterly-issue-43")

---

# Nucleoid combines publication flags with logical operators

# approved is true
approved = true

# embargoed is false
embargoed = false

assert(approved and not embargoed, true)
assert(approved && !embargoed, true)
assert(embargoed or approved, true)
assert(embargoed || approved, true)

---

# Nucleoid respects parentheses in a layout formula

# columns is 2
columns = 2

# rows is 3
rows = 3

# spreads is 4
spreads = 4

assert(columns + rows * spreads, 14)
assert((columns + rows) * spreads, 20)

---

# Nucleoid fits articles with the remainder operator

# lines is 137
lines = 137

# perColumn is 40
perColumn = 40

# overflow is lines modulo perColumn
overflow = lines % perColumn

assert(overflow, 17)

# lines is 160
lines = 160

assert(overflow, 0)

---

# Nucleoid appends an edition number to a title code

# titleCode is "TTL-"
titleCode = "TTL-"

# edition is 3
edition = 3

# code is titleCode plus edition
code = titleCode + edition

assert(code, "TTL-3")

# edition is 4
edition = 4

assert(code, "TTL-4")

---

# Nucleoid counts the characters of an imprint

# imprint is "OAK"
imprint = "OAK"

# width is imprint's length
width = imprint.length

assert(width, 3)

# imprint is "OAKRIDGE"
imprint = "OAKRIDGE"

assert(width, 8)

---

# Nucleoid lowercases a genre name as a dependency

# genre is "MYSTERY"
genre = "MYSTERY"

# slug is genre lowercased
slug = genre.lower()

assert(slug, "mystery")

# genre is "HISTORY"
genre = "HISTORY"

assert(slug, "history")

---

# Nucleoid rewrites a manuscript path with replace

# manuscript is "draft/old/chapter"
manuscript = "draft/old/chapter"

# stage is "final"
stage = "final"

# current is manuscript with "old" replaced by stage
current = manuscript.replace("old", stage)

assert(current, "draft/final/chapter")

# stage is "proof"
stage = "proof"

assert(current, "draft/proof/chapter")

---

# Nucleoid takes the year from the end of a catalogue code

# catalogue is "CAT-2022"
catalogue = "CAT-2022"

# year is the last four characters of catalogue
year = catalogue[-4:]

assert(year, "2022")

# catalogue is "CAT-2023"
catalogue = "CAT-2023"

assert(year, "2023")

---

# Nucleoid declares a title type with a constructor

# There is a Title type,
# which has a name as a string
class Title(name: str):
    this.name = name

# title1 is a Title whose name is "Northlight"
title1 = Title("Northlight")

assert(title1, { "id": "title1", "name": "Northlight" })
assert(Title.length, 1)

---

# Nucleoid declares an anthology as a subtype of a title

# There is a Title type,
# which has a name as a string
class Title(name: str):
    this.name = name

# There is an Anthology type,
# which is a subtype of Title
# and has a stories as a number
class Anthology: Title
    def init(name, stories):
        super(name)
        this.name = name
        this.stories = stories

# anthology1 is an Anthology whose name is "Voices" and whose stories is 12
anthology1 = Anthology("Voices", 12)

assert(anthology1, { "id": "anthology1", "name": "Voices", "stories": 12 })

---

# Nucleoid numbers every article with a class-level rule

# There is an Article type,
# which has a number as a number
class Article(number: int):
    this.number = number

# Any article's slug is "ART-" plus the article's number
$Article.slug = "ART-" + $Article.number

# article1 is an Article whose number is 9
article1 = Article(9)

assert(article1.slug, "ART-9")

# article2 is an Article whose number is 10
article2 = Article(10)

assert(article2.slug, "ART-10")

---

# Nucleoid replaces a class-level rule on a section

# There is a Section type
class Section:
    pass

# section1 is a Section whose number is 4
section1 = Section()
section1.number = 4

# Any section's sign is "S" plus the section's number
$Section.sign = "S" + $Section.number

assert(section1.sign, "S4")

# Any section's sign is "SECTION-" plus the section's number
$Section.sign = "SECTION-" + $Section.number

assert(section1.sign, "SECTION-4")

# section1's number is 5
section1.number = 5

assert(section1.sign, "SECTION-5")

---

# Nucleoid marks a long read with a class-level conditional

# There is a Feature type
class Feature:
    pass

# Any feature over 3000 words is a long read
if $Feature.words > 3000:
    $Feature.longRead = true

# feature1 is a Feature whose words is 900
feature1 = Feature()
feature1.words = 900

assert(feature1.longRead, null)

# feature1's words is 5000
feature1.words = 5000

assert(feature1.longRead, true)

---

# Nucleoid rates a submission with a class-level chain

# There is a Submission type
class Submission:
    pass

# accept is "ACCEPT", revise is "REVISE", and decline is "DECLINE"
accept = "ACCEPT"; revise = "REVISE"; decline = "DECLINE"

# If any submission's score is greater than 80, then the submission's outcome is accept,
# else if the submission's score is greater than 50, then the submission's outcome is revise,
# else the submission's outcome is decline
if $Submission.score > 80:
    $Submission.outcome = accept
else if $Submission.score > 50:
    $Submission.outcome = revise
else:
    $Submission.outcome = decline

# submission1 is a Submission whose score is 65
submission1 = Submission()
submission1.score = 65

assert(submission1.outcome, "REVISE")

# revise is "RESUBMIT"
revise = "RESUBMIT"

assert(submission1.outcome, "RESUBMIT")

# submission1's score is 90
submission1.score = 90

assert(submission1.outcome, "ACCEPT")

---

# Nucleoid counts the articles of an issue with a class-level aggregate

# There is an Issue type
class Issue:
    pass

# There is an Article type
class Article:
    pass

# issue1 is an Issue
issue1 = Issue()

# article1 is an Article whose issue is issue1
article1 = Article()
article1.issue = issue1

# article2 is an Article whose issue is issue1
article2 = Article()
article2.issue = issue1

# Any issue's articles is the number of articles whose issue is the issue
$Issue.articles = Article.filter(a => a.issue == $Issue).length

assert(issue1.articles, 2)

# article3 is an Article whose issue is issue1
article3 = Article()
article3.issue = issue1

assert(issue1.articles, 3)

---

# Nucleoid builds an isbn from a property that arrives later

# There is a Book type
class Book:
    pass

# book1 is a Book
book1 = Book()

# book1's isbn is "978" plus book1's serial
book1.isbn = "978" + book1.serial

assert(book1.isbn, null)

# book1's serial is "0306406"
book1.serial = "0306406"

assert(book1.isbn, "9780306406")

---

# Nucleoid reads an author through a title reference

# There is a Title type
class Title:
    pass

# There is an Author type
class Author:
    pass

# title1 is a Title
title1 = Title()

# author1 is an Author whose name is "Vera"
author1 = Author()
author1.name = "Vera"

# title1's author is author1
title1.author = author1

# title1's credit is "By " plus title1's author's name
title1.credit = "By " + title1.author.name

assert(title1.credit, "By Vera")

# author1's name is "Vera Lang"
author1.name = "Vera Lang"

assert(title1.credit, "By Vera Lang")

---

# Nucleoid clears a byline when a name is deleted

# There is a Byline type
class Byline:
    pass

# byline1 is a Byline
byline1 = Byline()

# byline1's name is "Ana"
byline1.name = "Ana"

# byline1's role is "EDITOR"
byline1.role = "EDITOR"

# byline1's text is byline1's name plus ", " plus byline1's role
byline1.text = byline1.name + ", " + byline1.role

assert(byline1.text, "Ana, EDITOR")

# byline1's name is deleted
delete byline1.name

assert(byline1.text, null)
assert(byline1.role, "EDITOR")

---

# Nucleoid computes a schedule in a block

# words is 60000
words = 60000

# schedule is null
schedule = null

# while in the block, daily is a local variable that is words divided by 30,
# and schedule is daily times 30
{
    daily = words / 30
    schedule = daily * 30
}

assert(schedule, 60000)

# words is 90000
words = 90000

assert(schedule, 90000)

---

# Nucleoid computes a cover area in a nested block

# side is 21
side = 21

# while in the block, face is a local variable that is side squared,
# and in a nested block, area is face times 2
{
    face = Math.pow(side, 2)
    {
        area = face * 2
    }
}

assert(area, 882)

---

# Nucleoid raises a deadline alert in a nested if inside a block

# planned is 300
planned = 300

# written is 180
written = 180

# notify is true
notify = true

# while in the block, remaining is a local variable that is planned minus written,
# and if remaining is greater than 100, then alert is notify
{
    remaining = planned - written
    if remaining > 100:
        alert = notify
}

assert(alert, true)

# notify is false
notify = false

assert(alert, false)

---

# Nucleoid picks an edit level in a nested else inside a block

# chapters is 3
chapters = 3

# passes is 3
passes = 3

# structural is "STRUCTURAL"
structural = "STRUCTURAL"

# copy is "COPY"
copy = "COPY"

# while in the block, effort is a local variable that is chapters times passes,
# and if effort is greater than 20, then edit is structural,
# else edit is copy
{
    effort = chapters * passes
    if effort > 20:
        edit = structural
    else:
        edit = copy
}

assert(edit, "COPY")

# copy is "LINE"
copy = "LINE"

assert(edit, "LINE")

---

# Nucleoid computes a cover price in a class-level block

# There is an Edition type
class Edition:
    pass

# while in the block, margin is a local variable that is any edition's cost times 60 divided by 100,
# and the edition's price is the edition's cost plus margin
{
    margin = $Edition.cost * 60 / 100
    $Edition.price = $Edition.cost + margin
}

# edition1 is an Edition
edition1 = Edition()

assert(edition1.price, null)

# edition1's cost is 5
edition1.cost = 5

assert(edition1.price, 8)

---

# Nucleoid computes a column index in a nested class-level block

# There is a Page type
class Page:
    pass

# while in the block, share is a local variable that is 900 divided by any page's columns,
# and in a nested block, the page's index is the floor of share times the page's lines
{
    share = 900 / $Page.columns
    {
        $Page.index = Math.floor(share * $Page.lines)
    }
}

# page1 is a Page
page1 = Page()

# page1's columns is 7
page1.columns = 7

# page1's lines is 4
page1.lines = 4

assert(page1.index, 514)

---

# Nucleoid flags a retraction with an if statement on a property

# There is a Paper type
class Paper:
    pass

# paper1 is a Paper whose state is "PUBLISHED"
paper1 = Paper()
paper1.state = "PUBLISHED"

# if paper1's state is "DISPUTED", then paper1's retract is true
if paper1.state == "DISPUTED":
    paper1.retract = true

assert(paper1.retract, null)

# paper1's state is "DISPUTED"
paper1.state = "DISPUTED"

assert(paper1.retract, true)

---

# Nucleoid chooses a binding note with an else statement on a property

# There is a Volume type
class Volume:
    pass

# volume1 is a Volume whose deluxe is false
volume1 = Volume()
volume1.deluxe = false

# cloth is "CLOTH BOUND"
cloth = "CLOTH BOUND"

# card is "PERFECT BOUND"
card = "PERFECT BOUND"

# if volume1's deluxe is true, then volume1's binding is cloth,
# else volume1's binding is card
if volume1.deluxe == true:
    volume1.binding = cloth
else:
    volume1.binding = card

assert(volume1.binding, "PERFECT BOUND")

# volume1's deluxe is true
volume1.deluxe = true

assert(volume1.binding, "CLOTH BOUND")

---

# Nucleoid bands an advance with multiple else if statements on a property

# There is a Deal type
class Deal:
    pass

# deal1 is a Deal whose sales is 2000
deal1 = Deal()
deal1.sales = 2000

# unit is 1
unit = 1

# if deal1's sales is greater than 50000, then deal1's advance is deal1's sales times unit plus 10000,
# else if deal1's sales is greater than 10000, then deal1's advance is deal1's sales times unit plus 2000,
# else deal1's advance is deal1's sales times unit
if deal1.sales > 50000:
    deal1.advance = deal1.sales * unit + 10000
else if deal1.sales > 10000:
    deal1.advance = deal1.sales * unit + 2000
else:
    deal1.advance = deal1.sales * unit

assert(deal1.advance, 2000)

# deal1's sales is 20000
deal1.sales = 20000

assert(deal1.advance, 22000)

# deal1's sales is 60000
deal1.sales = 60000

assert(deal1.advance, 70000)

---

# Nucleoid calls a rate function in an assignment

# rate returns the fee divided by the words times 1000
def rate(fee, words):
    return fee / words * 1000

# payment is 600
payment = 600

# length is 2000
length = 2000

# perThousand is the result of the rate function call
perThousand = rate(payment, length)

assert(perThousand, 300)

# payment is 800
payment = 800

assert(perThousand, 400)

---

# Nucleoid updates a commission when its function is redefined

# commission returns the sale times 15 divided by 100
def commission(sale):
    return sale * 15 / 100

# takings is 2000
takings = 2000

# agentFee is the result of the commission function call with takings
agentFee = commission(takings)

assert(agentFee, 300)

# commission returns the sale times 20 divided by 100
def commission(sale):
    return sale * 20 / 100

assert(agentFee, 400)

---

# Nucleoid nests a tier lookup inside a fee function

# tier returns 7 times the band
def tier(band):
    return band * 7

# fee returns the pages times the tier of the band
def fee(pages, band):
    return pages * tier(band)

# extent is 5
extent = 5

# rateBand is 2
rateBand = 2

# payable is the result of the fee function call
payable = fee(extent, rateBand)

assert(payable, 70)

---

# Nucleoid finds a price with the three lambda forms

# prices is a list of 8, 12 and 20
prices = [8, 12, 20]

assert(prices.find(function(price) { return price == 20 }), 20)
assert(prices.find(price => { return price == 12 }), 12)
assert(prices.find(price => price == 8), 8)

---

# Nucleoid filters a list of titles by two thresholds

# There is a Title type,
# which has an extent as a number
class Title(extent: int):
    this.extent = extent

# There are Titles whose extents are 120, 320 and 520
Title(120); Title(320); Title(520)

# ceiling is 400
ceiling = 400

# floorExtent is 200
floorExtent = 200

# midrange is Titles whose extent is above floorExtent and below ceiling
midrange = Title.filter(t => t.extent > floorExtent).filter(t => t.extent < ceiling)

assert(midrange.length, 1)
assert(midrange[0].extent, 320)

# floorExtent is 50
floorExtent = 50

assert(midrange.length, 2)
assert(midrange[0].extent, 120)

---

# Nucleoid maps print runs into costs

# runs is a list of 1000, 2000 and 3000
runs = [1000, 2000, 3000]

# perCopy is 2
perCopy = 2

# costs is runs mapped to the run times perCopy
costs = runs.map(r => r * perCopy)

assert(costs[0], 2000)
assert(costs[2], 6000)

# perCopy is 3
perCopy = 3

assert(costs[2], 9000)

---

# Nucleoid reduces a set of sales into a total

# months is a list of 400, 500 and 600
months = [400, 500, 600]

# total is the sum of months
total = months.reduce((sum, month) => sum + month, 0)

assert(total, 1500)

# Add 500 to months
months.push(500)

assert(total, 2000)

---

# Nucleoid checks whether every article is proofed

# There is an Article type
class Article:
    pass

# article1 is an Article that is proofed
article1 = Article()
article1.proofed = true

# article2 is an Article that is proofed
article2 = Article()
article2.proofed = true

# ready is whether every article is proofed
ready = Article.every(a => a.proofed == true)

assert(ready, true)

# article2 is not proofed
article2.proofed = false

assert(ready, false)

---

# Nucleoid checks whether any right is unsold

# There is a Right type
class Right:
    pass

# right1 is a Right that is sold
right1 = Right()
right1.unsold = false

# right2 is a Right that is sold
right2 = Right()
right2.unsold = false

# pitch is whether any right is unsold
pitch = Right.some(r => r.unsold == true)

assert(pitch, false)

# right2 is unsold
right2.unsold = true

assert(pitch, true)

---

# Nucleoid joins a running order into a single string

# items is a list of "LEAD", "FEATURE" and "REVIEW"
items = ["LEAD", "FEATURE", "REVIEW"]

# order is items joined with " + "
order = items.join(" + ")

assert(order, "LEAD + FEATURE + REVIEW")

# Add "LETTERS" to items
items.push("LETTERS")

assert(order, "LEAD + FEATURE + REVIEW + LETTERS")

---

# Nucleoid creates a proof for every live article

# There is an Article type
class Article:
    pass

# article1 is an Article
article1 = Article()

# article2 is an Article that is pulled
article2 = Article()
article2.pulled = true

# article3 is an Article
article3 = Article()

# There is a Proof type,
# which has an article as an Article
class Proof(article):
    this.article = article

# Any proof's kind is "FIRST"
$Proof.kind = "FIRST"

# For each article of Article, if the article is not pulled,
# then there is a Proof whose article is the article
for article of Article:
    if not article.pulled:
        Proof(article)

assert(Proof.length, 2)
assert(Proof[0].article.id, "article1")
assert(Proof[1].article.id, "article3")
assert(Proof[0].kind, "FIRST")

---

# Nucleoid rolls back a listing if a rule throws

# There is a Listing type
class Listing:
    pass

# If any listing's price is greater than 200, then throw 'PRICE_TOO_HIGH'
if $Listing.price > 200:
    throw 'PRICE_TOO_HIGH'

# listing1 is a Listing
listing1 = Listing()

try:
    # listing1's price is 500
    listing1.price = 500
catch error:
    assert(error, "PRICE_TOO_HIGH")

assert(listing1.price, null)

---

# Nucleoid refuses a cycle between a fee and a budget

# fee is 200
fee = 200

# budget is fee times 5
budget = fee * 5

assert(budget, 1000)

try:
    # fee is budget times 5
    fee = budget * 5
catch error:
    assert(error, TypeError("Circular Dependency"))

---

# Nucleoid validates an issue code with a regular expression

# There is an Issue type
class Issue:
    pass

# If any issue's code does not match /[A-Z]{3}[0-9]{2}/, then throw 'INVALID_ISSUE'
if not /[A-Z]{3}[0-9]{2}/.test($Issue.code):
    throw 'INVALID_ISSUE'

# issue1 is an Issue
issue1 = Issue()

assert(issue1.code, null)

try:
    # issue1's code is 'AB1'
    issue1.code = 'AB1'
catch error:
    assert(error, "INVALID_ISSUE")
```
