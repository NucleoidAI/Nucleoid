# Nucleoid Language Reference - Synthesized Use Cases 09

```

# Nucleoid converts a balance through a chain of variables

# cents is 250000
cents = 250000

# units is cents divided by 100
units = cents / 100

# thousands is units divided by 1000
thousands = units / 1000

assert(units, 2500)
assert(thousands, 2.5)

# cents is 500000
cents = 500000

assert(units, 5000)
assert(thousands, 5)

---

# Nucleoid replaces the source of a monthly payment

# standard is 1200
standard = 1200

# reduced is 600
reduced = 600

# payment is standard divided by 12
payment = standard / 12

assert(payment, 100)

# payment is reduced divided by 12
payment = reduced / 12

assert(payment, 50)

# reduced is 720
reduced = 720

assert(payment, 60)

---

# Nucleoid adds interest to a balance using its own value

# balance is 1000
balance = 1000

# balance is balance plus 250
balance = balance + 250

assert(balance, 1250)

---

# Nucleoid fixes an exchange rate with the value property

# rate is 2
rate = 2

# amount is 300
amount = 300

# converted is rate's value times amount
converted = rate.value * amount

assert(converted, 600)

# rate is 3
rate = 3

assert(converted, 600)

# amount is 500
amount = 500

assert(converted, 1000)

---

# Nucleoid clears a statement when its balance is deleted

# opening is 400
opening = 400

# fees is opening divided by 4
fees = opening / 4

# closing is opening minus fees
closing = opening - fees

assert(closing, 300)

# opening is deleted
delete opening

assert(fees, null)
assert(closing, null)

try:
    # opening
    opening
catch error:
    assert(error, ReferenceError("opening is not defined"))

---

# Nucleoid assigns an overdraft comparison to a variable

# balance is 50
balance = 50

# limit is 100
limit = 100

# overdrawn is whether balance is less than limit
overdrawn = balance < limit

assert(overdrawn, true)

# balance is 150
balance = 150

assert(overdrawn, false)

---

# Nucleoid builds an account reference from a template literal

# sortCode is "204060"
sortCode = "204060"

# number is 12345678
number = 12345678

# reference is the sort code and number in a template
reference = `${sortCode}-${number}`

assert(reference, "204060-12345678")

# number is 87654321
number = 87654321

assert(reference, "204060-87654321")

---

# Nucleoid combines mandate flags with logical operators

# verified is true
verified = true

# frozen is false
frozen = false

assert(verified and not frozen, true)
assert(verified && !frozen, true)
assert(frozen or verified, true)
assert(frozen || verified, true)

---

# Nucleoid respects parentheses in an interest formula

# principal is 2
principal = 2

# years is 3
years = 3

# rate is 4
rate = 4

assert(principal + years * rate, 14)
assert((principal + years) * rate, 20)

---

# Nucleoid splits a payment with the remainder operator

# pence is 1055
pence = 1055

# perPound is 100
perPound = 100

# remainder is pence modulo perPound
remainder = pence % perPound

assert(remainder, 55)

# pence is 1099
pence = 1099

assert(remainder, 99)

---

# Nucleoid appends a branch number to a bank code

# bank is "BNK-"
bank = "BNK-"

# branch is 42
branch = 42

# code is bank plus branch
code = bank + branch

assert(code, "BNK-42")

# branch is 43
branch = 43

assert(code, "BNK-43")

---

# Nucleoid counts the characters of an iban

# iban is "GB29"
iban = "GB29"

# width is iban's length
width = iban.length

assert(width, 4)

# iban is "GB29NWBK6016"
iban = "GB29NWBK6016"

assert(width, 12)

---

# Nucleoid lowercases a product name as a dependency

# product is "SAVINGS"
product = "SAVINGS"

# key is product lowercased
key = product.lower()

assert(key, "savings")

# product is "CURRENT"
product = "CURRENT"

assert(key, "current")

---

# Nucleoid reads the country letter of an account code

# account is "G1234"
account = "G1234"

# country is the character of account at 0
country = account.charAt(0)

assert(country, "G")

# account is "F1234"
account = "F1234"

assert(country, "F")

---

# Nucleoid rewrites a ledger path with replace

# ledger is "book/draft/entry"
ledger = "book/draft/entry"

# stage is "final"
stage = "final"

# posted is ledger with "draft" replaced by stage
posted = ledger.replace("draft", stage)

assert(posted, "book/final/entry")

# stage is "audit"
stage = "audit"

assert(posted, "book/audit/entry")

---

# Nucleoid takes the last digits from a card number

# card is "4000123456781234"
card = "4000123456781234"

# lastFour is the last four characters of card
lastFour = card[-4:]

assert(lastFour, "1234")

# card is "4000123456785678"
card = "4000123456785678"

assert(lastFour, "5678")

---

# Nucleoid declares an account type with a constructor

# There is an Account type,
# which has a holder as a string
class Account(holder: str):
    this.holder = holder

# account1 is an Account whose holder is "Nia"
account1 = Account("Nia")

assert(account1, { "id": "account1", "holder": "Nia" })
assert(Account.length, 1)

---

# Nucleoid declares a savings account as a subtype of an account

# There is an Account type,
# which has a holder as a string
class Account(holder: str):
    this.holder = holder

# There is a Savings type,
# which is a subtype of Account
# and has a rate as a number
class Savings: Account
    def init(holder, rate):
        super(holder)
        this.holder = holder
        this.rate = rate

# savings1 is a Savings whose holder is "Omar" and whose rate is 3
savings1 = Savings("Omar", 3)

assert(savings1, { "id": "savings1", "holder": "Omar", "rate": 3 })

---

# Nucleoid references every transaction with a class-level rule

# There is a Transaction type,
# which has a number as a number
class Transaction(number: int):
    this.number = number

# Any transaction's reference is "TXN-" plus the transaction's number
$Transaction.reference = "TXN-" + $Transaction.number

# transaction1 is a Transaction whose number is 7
transaction1 = Transaction(7)

assert(transaction1.reference, "TXN-7")

# transaction2 is a Transaction whose number is 8
transaction2 = Transaction(8)

assert(transaction2.reference, "TXN-8")

---

# Nucleoid replaces a class-level rule on a branch

# There is a Branch type
class Branch:
    pass

# branch1 is a Branch whose number is 12
branch1 = Branch()
branch1.number = 12

# Any branch's sign is "B" plus the branch's number
$Branch.sign = "B" + $Branch.number

assert(branch1.sign, "B12")

# Any branch's sign is "BRANCH-" plus the branch's number
$Branch.sign = "BRANCH-" + $Branch.number

assert(branch1.sign, "BRANCH-12")

# branch1's number is 13
branch1.number = 13

assert(branch1.sign, "BRANCH-13")

---

# Nucleoid marks a large payment with a class-level conditional

# There is a Payment type
class Payment:
    pass

# Any payment above 10000 is reportable
if $Payment.amount > 10000:
    $Payment.reportable = true

# payment1 is a Payment whose amount is 500
payment1 = Payment()
payment1.amount = 500

assert(payment1.reportable, null)

# payment1's amount is 20000
payment1.amount = 20000

assert(payment1.reportable, true)

---

# Nucleoid rates a borrower with a class-level chain

# There is a Borrower type
class Borrower:
    pass

# prime is "PRIME", fair is "FAIR", and poor is "POOR"
prime = "PRIME"; fair = "FAIR"; poor = "POOR"

# If any borrower's score is greater than 700, then the borrower's band is prime,
# else if the borrower's score is greater than 500, then the borrower's band is fair,
# else the borrower's band is poor
if $Borrower.score > 700:
    $Borrower.band = prime
else if $Borrower.score > 500:
    $Borrower.band = fair
else:
    $Borrower.band = poor

# borrower1 is a Borrower whose score is 600
borrower1 = Borrower()
borrower1.score = 600

assert(borrower1.band, "FAIR")

# fair is "STANDARD"
fair = "STANDARD"

assert(borrower1.band, "STANDARD")

# borrower1's score is 800
borrower1.score = 800

assert(borrower1.band, "PRIME")

---

# Nucleoid counts the cards of an account with a class-level aggregate

# There is an Account type
class Account:
    pass

# There is a Card type
class Card:
    pass

# account1 is an Account
account1 = Account()

# card1 is a Card whose account is account1
card1 = Card()
card1.account = account1

# card2 is a Card whose account is account1
card2 = Card()
card2.account = account1

# Any account's cards is the number of cards whose account is the account
$Account.cards = Card.filter(c => c.account == $Account).length

assert(account1.cards, 2)

# card3 is a Card whose account is account1
card3 = Card()
card3.account = account1

assert(account1.cards, 3)

---

# Nucleoid builds a mandate from a property that arrives later

# There is a Mandate type
class Mandate:
    pass

# mandate1 is a Mandate
mandate1 = Mandate()

# mandate1's full is "DD" plus mandate1's serial
mandate1.full = "DD" + mandate1.serial

assert(mandate1.full, null)

# mandate1's serial is "9081"
mandate1.serial = "9081"

assert(mandate1.full, "DD9081")

---

# Nucleoid reads a holder through an account reference

# There is a Card type
class Card:
    pass

# There is a Holder type
class Holder:
    pass

# card1 is a Card
card1 = Card()

# holder1 is a Holder whose name is "Iva"
holder1 = Holder()
holder1.name = "Iva"

# card1's holder is holder1
card1.holder = holder1

# card1's embossed is "Card of " plus card1's holder's name
card1.embossed = "Card of " + card1.holder.name

assert(card1.embossed, "Card of Iva")

# holder1's name is "Ivan"
holder1.name = "Ivan"

assert(card1.embossed, "Card of Ivan")

---

# Nucleoid clears a posting line when an amount is deleted

# There is a Posting type
class Posting:
    pass

# posting1 is a Posting
posting1 = Posting()

# posting1's code is "CR"
posting1.code = "CR"

# posting1's amount is "250"
posting1.amount = "250"

# posting1's line is posting1's code plus " " plus posting1's amount
posting1.line = posting1.code + " " + posting1.amount

assert(posting1.line, "CR 250")

# posting1's amount is deleted
delete posting1.amount

assert(posting1.line, null)
assert(posting1.code, "CR")

---

# Nucleoid computes a repayment in a block

# loan is 2400
loan = 2400

# repayment is null
repayment = null

# while in the block, monthly is a local variable that is loan divided by 12,
# and repayment is monthly times 12
{
    monthly = loan / 12
    repayment = monthly * 12
}

assert(repayment, 2400)

# loan is 3600
loan = 3600

assert(repayment, 3600)

---

# Nucleoid computes a compound base in a nested block

# rate is 3
rate = 3

# while in the block, square is a local variable that is rate squared,
# and in a nested block, factor is square times 100
{
    square = Math.pow(rate, 2)
    {
        factor = square * 100
    }
}

assert(factor, 900)

---

# Nucleoid raises a fraud alert in a nested if inside a block

# spend is 900
spend = 900

# usual is 200
usual = 200

# notify is true
notify = true

# while in the block, excess is a local variable that is spend minus usual,
# and if excess is greater than 500, then alert is notify
{
    excess = spend - usual
    if excess > 500:
        alert = notify
}

assert(alert, true)

# notify is false
notify = false

assert(alert, false)

---

# Nucleoid picks a review level in a nested else inside a block

# accounts is 2
accounts = 2

# products is 3
products = 3

# deep is "DEEP"
deep = "DEEP"

# light is "LIGHT"
light = "LIGHT"

# while in the block, exposure is a local variable that is accounts times products,
# and if exposure is greater than 10, then review is deep,
# else review is light
{
    exposure = accounts * products
    if exposure > 10:
        review = deep
    else:
        review = light
}

assert(review, "LIGHT")

# light is "L"
light = "L"

assert(review, "L")

---

# Nucleoid computes a charge in a class-level block

# There is a Facility type
class Facility:
    pass

# while in the block, fee is a local variable that is any facility's drawn times 2 divided by 100,
# and the facility's cost is the facility's drawn plus fee
{
    fee = $Facility.drawn * 2 / 100
    $Facility.cost = $Facility.drawn + fee
}

# facility1 is a Facility
facility1 = Facility()

assert(facility1.cost, null)

# facility1's drawn is 5000
facility1.drawn = 5000

assert(facility1.cost, 5100)

---

# Nucleoid computes a risk index in a nested class-level block

# There is a Portfolio type
class Portfolio:
    pass

# while in the block, unit is a local variable that is 200 divided by any portfolio's holdings,
# and in a nested block, the portfolio's index is the floor of unit times the portfolio's weight
{
    unit = 200 / $Portfolio.holdings
    {
        $Portfolio.index = Math.floor(unit * $Portfolio.weight)
    }
}

# portfolio1 is a Portfolio
portfolio1 = Portfolio()

# portfolio1's holdings is 3
portfolio1.holdings = 3

# portfolio1's weight is 7
portfolio1.weight = 7

assert(portfolio1.index, 466)

---

# Nucleoid flags a dispute with an if statement on a property

# There is a Charge type
class Charge:
    pass

# charge1 is a Charge whose state is "SETTLED"
charge1 = Charge()
charge1.state = "SETTLED"

# if charge1's state is "DISPUTED", then charge1's hold is true
if charge1.state == "DISPUTED":
    charge1.hold = true

assert(charge1.hold, null)

# charge1's state is "DISPUTED"
charge1.state = "DISPUTED"

assert(charge1.hold, true)

---

# Nucleoid chooses a statement note with an else statement on a property

# There is a Statement type
class Statement:
    pass

# statement1 is a Statement whose paperless is false
statement1 = Statement()
statement1.paperless = false

# digital is "EMAIL ONLY"
digital = "EMAIL ONLY"

# printed is "POST MONTHLY"
printed = "POST MONTHLY"

# if statement1's paperless is true, then statement1's delivery is digital,
# else statement1's delivery is printed
if statement1.paperless == true:
    statement1.delivery = digital
else:
    statement1.delivery = printed

assert(statement1.delivery, "POST MONTHLY")

# statement1's paperless is true
statement1.paperless = true

assert(statement1.delivery, "EMAIL ONLY")

---

# Nucleoid bands a commission with multiple else if statements on a property

# There is a Trade type
class Trade:
    pass

# trade1 is a Trade whose volume is 50
trade1 = Trade()
trade1.volume = 50

# unit is 4
unit = 4

# if trade1's volume is greater than 500, then trade1's fee is trade1's volume times unit minus 200,
# else if trade1's volume is greater than 100, then trade1's fee is trade1's volume times unit minus 50,
# else trade1's fee is trade1's volume times unit
if trade1.volume > 500:
    trade1.fee = trade1.volume * unit - 200
else if trade1.volume > 100:
    trade1.fee = trade1.volume * unit - 50
else:
    trade1.fee = trade1.volume * unit

assert(trade1.fee, 200)

# trade1's volume is 200
trade1.volume = 200

assert(trade1.fee, 750)

# trade1's volume is 600
trade1.volume = 600

assert(trade1.fee, 2200)

---

# Nucleoid calls an interest function in an assignment

# interest returns the principal times the rate divided by 100
def interest(principal, rate):
    return principal * rate / 100

# capital is 2000
capital = 2000

# percent is 5
percent = 5

# earned is the result of the interest function call
earned = interest(capital, percent)

assert(earned, 100)

# percent is 10
percent = 10

assert(earned, 200)

---

# Nucleoid updates a levy when its function is redefined

# levy returns the amount times 2 divided by 100
def levy(amount):
    return amount * 2 / 100

# transfer is 5000
transfer = 5000

# duty is the result of the levy function call with transfer
duty = levy(transfer)

assert(duty, 100)

# levy returns the amount times 4 divided by 100
def levy(amount):
    return amount * 4 / 100

assert(duty, 200)

---

# Nucleoid nests a band lookup inside a charge function

# band returns 5 times the tier
def band(tier):
    return tier * 5

# charge returns the value times the band of the tier
def charge(value, tier):
    return value * band(tier)

# amount is 4
amount = 4

# tier is 3
tier = 3

# payable is the result of the charge function call
payable = charge(amount, tier)

assert(payable, 60)

---

# Nucleoid finds a payment with the three lambda forms

# amounts is a list of 100, 200 and 300
amounts = [100, 200, 300]

assert(amounts.find(function(amount) { return amount == 300 }), 300)
assert(amounts.find(amount => { return amount == 200 }), 200)
assert(amounts.find(amount => amount == 100), 100)

---

# Nucleoid filters a book of trades by two thresholds

# There is a Deal type,
# which has a size as a number
class Deal(size: int):
    this.size = size

# There are Deals whose sizes are 100, 300 and 500
Deal(100); Deal(300); Deal(500)

# ceiling is 400
ceiling = 400

# floorSize is 200
floorSize = 200

# midbook is Deals whose size is above floorSize and below ceiling
midbook = Deal.filter(d => d.size > floorSize).filter(d => d.size < ceiling)

assert(midbook.length, 1)
assert(midbook[0].size, 300)

# floorSize is 50
floorSize = 50

assert(midbook.length, 2)
assert(midbook[0].size, 100)

---

# Nucleoid maps balances into converted amounts

# balances is a list of 10, 20 and 30
balances = [10, 20, 30]

# rate is 4
rate = 4

# converted is balances mapped to the balance times rate
converted = balances.map(b => b * rate)

assert(converted[0], 40)
assert(converted[2], 120)

# rate is 5
rate = 5

assert(converted[2], 150)

---

# Nucleoid reduces a ledger into a total

# entries is a list of 100, 250 and 400
entries = [100, 250, 400]

# total is the sum of entries
total = entries.reduce((sum, entry) => sum + entry, 0)

assert(total, 750)

# Add 250 to entries
entries.push(250)

assert(total, 1000)

---

# Nucleoid checks whether every account is verified

# There is an Account type
class Account:
    pass

# account1 is an Account that is verified
account1 = Account()
account1.verified = true

# account2 is an Account that is verified
account2 = Account()
account2.verified = true

# compliant is whether every account is verified
compliant = Account.every(a => a.verified == true)

assert(compliant, true)

# account2 is not verified
account2.verified = false

assert(compliant, false)

---

# Nucleoid checks whether any payment is held

# There is a Payment type
class Payment:
    pass

# payment1 is a Payment that is released
payment1 = Payment()
payment1.held = false

# payment2 is a Payment that is released
payment2 = Payment()
payment2.held = false

# blocked is whether any payment is held
blocked = Payment.some(p => p.held == true)

assert(blocked, false)

# payment2 is held
payment2.held = true

assert(blocked, true)

---

# Nucleoid joins a payment chain into a single string

# hops is a list of "GB", "DE" and "FR"
hops = ["GB", "DE", "FR"]

# chain is hops joined with ">"
chain = hops.join(">")

assert(chain, "GB>DE>FR")

# Add "ES" to hops
hops.push("ES")

assert(chain, "GB>DE>FR>ES")

---

# Nucleoid creates an advice for every settled trade

# There is a Trade type
class Trade:
    pass

# trade1 is a Trade
trade1 = Trade()

# trade2 is a Trade that is cancelled
trade2 = Trade()
trade2.cancelled = true

# trade3 is a Trade
trade3 = Trade()

# There is an Advice type,
# which has a trade as a Trade
class Advice(trade):
    this.trade = trade

# Any advice's kind is "SETTLEMENT"
$Advice.kind = "SETTLEMENT"

# For each trade of Trade, if the trade is not cancelled,
# then there is an Advice whose trade is the trade
for trade of Trade:
    if not trade.cancelled:
        Advice(trade)

assert(Advice.length, 2)
assert(Advice[0].trade.id, "trade1")
assert(Advice[1].trade.id, "trade3")
assert(Advice[0].kind, "SETTLEMENT")

---

# Nucleoid rolls back a withdrawal if a rule throws

# There is a Withdrawal type
class Withdrawal:
    pass

# If any withdrawal's amount is greater than 500, then throw 'LIMIT_EXCEEDED'
if $Withdrawal.amount > 500:
    throw 'LIMIT_EXCEEDED'

# withdrawal1 is a Withdrawal
withdrawal1 = Withdrawal()

try:
    # withdrawal1's amount is 900
    withdrawal1.amount = 900
catch error:
    assert(error, "LIMIT_EXCEEDED")

assert(withdrawal1.amount, null)

---

# Nucleoid refuses a cycle between a deposit and a total

# deposit is 100
deposit = 100

# total is deposit times 3
total = deposit * 3

assert(total, 300)

try:
    # deposit is total times 3
    deposit = total * 3
catch error:
    assert(error, TypeError("Circular Dependency"))

---

# Nucleoid validates a sort code with a regular expression

# There is a Mandate type
class Mandate:
    pass

# If any mandate's sortCode does not match /[0-9]{6}/, then throw 'INVALID_SORT_CODE'
if not /[0-9]{6}/.test($Mandate.sortCode):
    throw 'INVALID_SORT_CODE'

# mandate1 is a Mandate
mandate1 = Mandate()

assert(mandate1.sortCode, null)

try:
    # mandate1's sortCode is '2040'
    mandate1.sortCode = '2040'
catch error:
    assert(error, "INVALID_SORT_CODE")
```
