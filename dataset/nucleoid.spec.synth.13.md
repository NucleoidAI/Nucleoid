# Nucleoid Language Reference - Synthesized Use Cases 13

```

# Nucleoid converts a basket total through a chain of variables

# pence is 45000
pence = 45000

# pounds is pence divided by 100
pounds = pence / 100

# baskets is pounds divided by 45
baskets = pounds / 45

assert(pounds, 450)
assert(baskets, 10)

# pence is 90000
pence = 90000

assert(pounds, 900)
assert(baskets, 20)

---

# Nucleoid replaces the source of a unit price

# list is 80
list = 80

# sale is 40
sale = 40

# price is list divided by 2
price = list / 2

assert(price, 40)

# price is sale divided by 2
price = sale / 2

assert(price, 20)

# sale is 60
sale = 60

assert(price, 30)

---

# Nucleoid raises a stock level using its own value

# stock is 250
stock = 250

# stock is stock plus 150
stock = stock + 150

assert(stock, 400)

---

# Nucleoid fixes a promotional price with the value property

# promo is 12
promo = 12

# quantity is 5
quantity = 5

# locked is promo's value times quantity
locked = promo.value * quantity

assert(locked, 60)

# promo is 15
promo = 15

assert(locked, 60)

# quantity is 10
quantity = 10

assert(locked, 120)

---

# Nucleoid clears an order total when its subtotal is deleted

# subtotal is 200
subtotal = 200

# vat is subtotal divided by 5
vat = subtotal / 5

# total is subtotal plus vat
total = subtotal + vat

assert(total, 240)

# subtotal is deleted
delete subtotal

assert(vat, null)
assert(total, null)

try:
    # subtotal
    subtotal
catch error:
    assert(error, ReferenceError("subtotal is not defined"))

---

# Nucleoid assigns a reorder comparison to a variable

# onHand is 20
onHand = 20

# reorderPoint is 50
reorderPoint = 50

# reorder is whether onHand is less than reorderPoint
reorder = onHand < reorderPoint

assert(reorder, true)

# onHand is 80
onHand = 80

assert(reorder, false)

---

# Nucleoid builds a product url from a template literal

# category is "books"
category = "books"

# sku is 4471
sku = 4471

# url is the category and sku in a template
url = `/shop/${category}/${sku}`

assert(url, "/shop/books/4471")

# category is "music"
category = "music"

assert(url, "/shop/music/4471")

---

# Nucleoid combines listing flags with logical operators

# published is true
published = true

# archived is false
archived = false

assert(published and not archived, true)
assert(published && !archived, true)
assert(archived or published, true)
assert(archived || published, true)

---

# Nucleoid respects parentheses in a bundle formula

# item is 2
item = 2

# extra is 3
extra = 3

# packs is 4
packs = 4

assert(item + extra * packs, 14)
assert((item + extra) * packs, 20)

---

# Nucleoid packs an order with the remainder operator

# items is 53
items = 53

# perBox is 8
perBox = 8

# loose is items modulo perBox
loose = items % perBox

assert(loose, 5)

# items is 56
items = 56

assert(loose, 0)

---

# Nucleoid appends a variant number to a product code

# product is "SKU-"
product = "SKU-"

# variant is 2
variant = 2

# code is product plus variant
code = product + variant

assert(code, "SKU-2")

# variant is 3
variant = 3

assert(code, "SKU-3")

---

# Nucleoid counts the characters of a coupon

# coupon is "SAVE"
coupon = "SAVE"

# width is coupon's length
width = coupon.length

assert(width, 4)

# coupon is "SAVE2024"
coupon = "SAVE2024"

assert(width, 8)

---

# Nucleoid lowercases a brand name as a dependency

# brand is "NORTHWIND"
brand = "NORTHWIND"

# slug is brand lowercased
slug = brand.lower()

assert(slug, "northwind")

# brand is "EASTGATE"
brand = "EASTGATE"

assert(slug, "eastgate")

---

# Nucleoid reads the department letter of a shelf code

# shelf is "H12"
shelf = "H12"

# department is the character of shelf at 0
department = shelf.charAt(0)

assert(department, "H")

# shelf is "K12"
shelf = "K12"

assert(department, "K")

---

# Nucleoid rewrites a catalogue path with replace

# catalogue is "cat/old/page"
catalogue = "cat/old/page"

# season is "aw24"
season = "aw24"

# current is catalogue with "old" replaced by season
current = catalogue.replace("old", season)

assert(current, "cat/aw24/page")

# season is "ss25"
season = "ss25"

assert(current, "cat/ss25/page")

---

# Nucleoid takes the check digits from the end of a barcode

# barcode is "5012345678900"
barcode = "5012345678900"

# check is the last three characters of barcode
check = barcode[-3:]

assert(check, "900")

# barcode is "5012345678911"
barcode = "5012345678911"

assert(check, "911")

---

# Nucleoid declares a product type with a constructor

# There is a Product type,
# which has a title as a string
class Product(title: str):
    this.title = title

# product1 is a Product whose title is "Kettle"
product1 = Product("Kettle")

assert(product1, { "id": "product1", "title": "Kettle" })
assert(Product.length, 1)

---

# Nucleoid declares a bundle as a subtype of a product

# There is a Product type,
# which has a title as a string
class Product(title: str):
    this.title = title

# There is a Bundle type,
# which is a subtype of Product
# and has a items as a number
class Bundle: Product
    def init(title, items):
        super(title)
        this.title = title
        this.items = items

# bundle1 is a Bundle whose title is "Starter" and whose items is 3
bundle1 = Bundle("Starter", 3)

assert(bundle1, { "id": "bundle1", "title": "Starter", "items": 3 })

---

# Nucleoid numbers every order with a class-level rule

# There is an Order type,
# which has a number as a number
class Order(number: int):
    this.number = number

# Any order's reference is "OR-" plus the order's number
$Order.reference = "OR-" + $Order.number

# order1 is an Order whose number is 31
order1 = Order(31)

assert(order1.reference, "OR-31")

# order2 is an Order whose number is 32
order2 = Order(32)

assert(order2.reference, "OR-32")

---

# Nucleoid replaces a class-level rule on an aisle

# There is an Aisle type
class Aisle:
    pass

# aisle1 is an Aisle whose number is 5
aisle1 = Aisle()
aisle1.number = 5

# Any aisle's sign is "A" plus the aisle's number
$Aisle.sign = "A" + $Aisle.number

assert(aisle1.sign, "A5")

# Any aisle's sign is "AISLE-" plus the aisle's number
$Aisle.sign = "AISLE-" + $Aisle.number

assert(aisle1.sign, "AISLE-5")

# aisle1's number is 6
aisle1.number = 6

assert(aisle1.sign, "AISLE-6")

---

# Nucleoid marks a bulk order with a class-level conditional

# There is an Order type
class Order:
    pass

# Any order over 100 items is bulk
if $Order.items > 100:
    $Order.bulk = true

# order1 is an Order whose items is 20
order1 = Order()
order1.items = 20

assert(order1.bulk, null)

# order1's items is 300
order1.items = 300

assert(order1.bulk, true)

---

# Nucleoid tiers a customer with a class-level chain

# There is a Customer type
class Customer:
    pass

# platinum is "PLATINUM", gold is "GOLD", and basic is "BASIC"
platinum = "PLATINUM"; gold = "GOLD"; basic = "BASIC"

# If any customer's spend is greater than 5000, then the customer's tier is platinum,
# else if the customer's spend is greater than 1000, then the customer's tier is gold,
# else the customer's tier is basic
if $Customer.spend > 5000:
    $Customer.tier = platinum
else if $Customer.spend > 1000:
    $Customer.tier = gold
else:
    $Customer.tier = basic

# customer1 is a Customer whose spend is 2000
customer1 = Customer()
customer1.spend = 2000

assert(customer1.tier, "GOLD")

# gold is "PREMIER"
gold = "PREMIER"

assert(customer1.tier, "PREMIER")

# customer1's spend is 9000
customer1.spend = 9000

assert(customer1.tier, "PLATINUM")

---

# Nucleoid counts the reviews of a product with a class-level aggregate

# There is a Product type
class Product:
    pass

# There is a Review type
class Review:
    pass

# product1 is a Product
product1 = Product()

# review1 is a Review whose product is product1
review1 = Review()
review1.product = product1

# review2 is a Review whose product is product1
review2 = Review()
review2.product = product1

# Any product's reviews is the number of reviews whose product is the product
$Product.reviews = Review.filter(r => r.product == $Product).length

assert(product1.reviews, 2)

# review3 is a Review whose product is product1
review3 = Review()
review3.product = product1

assert(product1.reviews, 3)

---

# Nucleoid builds a receipt from a property that arrives later

# There is a Receipt type
class Receipt:
    pass

# receipt1 is a Receipt
receipt1 = Receipt()

# receipt1's full is "RC" plus receipt1's serial
receipt1.full = "RC" + receipt1.serial

assert(receipt1.full, null)

# receipt1's serial is "2214"
receipt1.serial = "2214"

assert(receipt1.full, "RC2214")

---

# Nucleoid reads a supplier through a product reference

# There is a Product type
class Product:
    pass

# There is a Supplier type
class Supplier:
    pass

# product1 is a Product
product1 = Product()

# supplier1 is a Supplier whose name is "Bell"
supplier1 = Supplier()
supplier1.name = "Bell"

# product1's supplier is supplier1
product1.supplier = supplier1

# product1's sourced is "From " plus product1's supplier's name
product1.sourced = "From " + product1.supplier.name

assert(product1.sourced, "From Bell")

# supplier1's name is "Bellrose"
supplier1.name = "Bellrose"

assert(product1.sourced, "From Bellrose")

---

# Nucleoid clears a label when a price is deleted

# There is a Label type
class Label:
    pass

# label1 is a Label
label1 = Label()

# label1's name is "Mug"
label1.name = "Mug"

# label1's price is "6"
label1.price = "6"

# label1's text is label1's name plus " " plus label1's price
label1.text = label1.name + " " + label1.price

assert(label1.text, "Mug 6")

# label1's price is deleted
delete label1.price

assert(label1.text, null)
assert(label1.name, "Mug")

---

# Nucleoid computes a margin in a block

# revenue is 1200
revenue = 1200

# margin is null
margin = null

# while in the block, cost is a local variable that is revenue divided by 4,
# and margin is revenue minus cost
{
    cost = revenue / 4
    margin = revenue - cost
}

assert(margin, 900)

# revenue is 2000
revenue = 2000

assert(margin, 1500)

---

# Nucleoid computes a display area in a nested block

# side is 7
side = 7

# while in the block, face is a local variable that is side squared,
# and in a nested block, area is face times 3
{
    face = Math.pow(side, 2)
    {
        area = face * 3
    }
}

assert(area, 147)

---

# Nucleoid raises a stockout alert in a nested if inside a block

# demand is 500
demand = 500

# available is 380
available = 380

# notify is true
notify = true

# while in the block, gap is a local variable that is demand minus available,
# and if gap is greater than 100, then alert is notify
{
    gap = demand - available
    if gap > 100:
        alert = notify
}

assert(alert, true)

# notify is false
notify = false

assert(alert, false)

---

# Nucleoid picks a delivery option in a nested else inside a block

# weight is 2
weight = 2

# distance is 4
distance = 4

# freight is "FREIGHT"
freight = "FREIGHT"

# parcel is "PARCEL"
parcel = "PARCEL"

# while in the block, effort is a local variable that is weight times distance,
# and if effort is greater than 20, then shipping is freight,
# else shipping is parcel
{
    effort = weight * distance
    if effort > 20:
        shipping = freight
    else:
        shipping = parcel
}

assert(shipping, "PARCEL")

# parcel is "PCL"
parcel = "PCL"

assert(shipping, "PCL")

---

# Nucleoid computes a retail price in a class-level block

# There is a Line type
class Line:
    pass

# while in the block, uplift is a local variable that is any line's cost times 50 divided by 100,
# and the line's retail is the line's cost plus uplift
{
    uplift = $Line.cost * 50 / 100
    $Line.retail = $Line.cost + uplift
}

# line1 is a Line
line1 = Line()

assert(line1.retail, null)

# line1's cost is 40
line1.cost = 40

assert(line1.retail, 60)

---

# Nucleoid computes a shelf index in a nested class-level block

# There is a Store type
class Store:
    pass

# while in the block, share is a local variable that is 400 divided by any store's bays,
# and in a nested block, the store's index is the floor of share times the store's shelves
{
    share = 400 / $Store.bays
    {
        $Store.index = Math.floor(share * $Store.shelves)
    }
}

# store1 is a Store
store1 = Store()

# store1's bays is 3
store1.bays = 3

# store1's shelves is 7
store1.shelves = 7

assert(store1.index, 933)

---

# Nucleoid flags a return with an if statement on a property

# There is a Parcel type
class Parcel:
    pass

# parcel1 is a Parcel whose state is "DELIVERED"
parcel1 = Parcel()
parcel1.state = "DELIVERED"

# if parcel1's state is "REFUSED", then parcel1's returning is true
if parcel1.state == "REFUSED":
    parcel1.returning = true

assert(parcel1.returning, null)

# parcel1's state is "REFUSED"
parcel1.state = "REFUSED"

assert(parcel1.returning, true)

---

# Nucleoid chooses a packaging note with an else statement on a property

# There is an Item type
class Item:
    pass

# item1 is an Item whose gift is false
item1 = Item()
item1.gift = false

# wrapped is "GIFT WRAPPED"
wrapped = "GIFT WRAPPED"

# plain is "PLAIN BOX"
plain = "PLAIN BOX"

# if item1's gift is true, then item1's packing is wrapped,
# else item1's packing is plain
if item1.gift == true:
    item1.packing = wrapped
else:
    item1.packing = plain

assert(item1.packing, "PLAIN BOX")

# item1's gift is true
item1.gift = true

assert(item1.packing, "GIFT WRAPPED")

---

# Nucleoid bands a delivery charge with multiple else if statements on a property

# There is a Basket type
class Basket:
    pass

# basket1 is a Basket whose total is 20
basket1 = Basket()
basket1.total = 20

# unit is 1
unit = 1

# if basket1's total is greater than 100, then basket1's postage is basket1's total times unit minus 20,
# else if basket1's total is greater than 50, then basket1's postage is basket1's total times unit minus 10,
# else basket1's postage is basket1's total times unit
if basket1.total > 100:
    basket1.postage = basket1.total * unit - 20
else if basket1.total > 50:
    basket1.postage = basket1.total * unit - 10
else:
    basket1.postage = basket1.total * unit

assert(basket1.postage, 20)

# basket1's total is 80
basket1.total = 80

assert(basket1.postage, 70)

# basket1's total is 200
basket1.total = 200

assert(basket1.postage, 180)

---

# Nucleoid calls a discount function in an assignment

# discount returns the price times the percent divided by 100
def discount(price, percent):
    return price * percent / 100

# ticket is 200
ticket = 200

# rate is 25
rate = 25

# saving is the result of the discount function call
saving = discount(ticket, rate)

assert(saving, 50)

# rate is 50
rate = 50

assert(saving, 100)

---

# Nucleoid updates a markup when its function is redefined

# markup returns the cost times 30 divided by 100
def markup(cost):
    return cost * 30 / 100

# wholesale is 100
wholesale = 100

# uplift is the result of the markup function call with wholesale
uplift = markup(wholesale)

assert(uplift, 30)

# markup returns the cost times 60 divided by 100
def markup(cost):
    return cost * 60 / 100

assert(uplift, 60)

---

# Nucleoid nests a band lookup inside a postage function

# band returns 2 times the zone
def band(zone):
    return zone * 2

# postage returns the weight times the band of the zone
def postage(weight, zone):
    return weight * band(zone)

# parcelWeight is 6
parcelWeight = 6

# parcelZone is 3
parcelZone = 3

# charge is the result of the postage function call
charge = postage(parcelWeight, parcelZone)

assert(charge, 36)

---

# Nucleoid finds a price with the three lambda forms

# prices is a list of 5, 10 and 15
prices = [5, 10, 15]

assert(prices.find(function(price) { return price == 15 }), 15)
assert(prices.find(price => { return price == 10 }), 10)
assert(prices.find(price => price == 5), 5)

---

# Nucleoid filters a catalogue by two thresholds

# There is an Item type,
# which has a price as a number
class Item(price: int):
    this.price = price

# There are Items whose prices are 10, 50 and 90
Item(10); Item(50); Item(90)

# ceiling is 70
ceiling = 70

# floorPrice is 30
floorPrice = 30

# midrange is Items whose price is above floorPrice and below ceiling
midrange = Item.filter(i => i.price > floorPrice).filter(i => i.price < ceiling)

assert(midrange.length, 1)
assert(midrange[0].price, 50)

# floorPrice is 5
floorPrice = 5

assert(midrange.length, 2)
assert(midrange[0].price, 10)

---

# Nucleoid maps quantities into line totals

# quantities is a list of 1, 2 and 3
quantities = [1, 2, 3]

# unitPrice is 20
unitPrice = 20

# lines is quantities mapped to the quantity times unitPrice
lines = quantities.map(q => q * unitPrice)

assert(lines[0], 20)
assert(lines[2], 60)

# unitPrice is 30
unitPrice = 30

assert(lines[2], 90)

---

# Nucleoid reduces a basket into a total

# lines is a list of 30, 45 and 25
lines = [30, 45, 25]

# total is the sum of lines
total = lines.reduce((sum, line) => sum + line, 0)

assert(total, 100)

# Add 50 to lines
lines.push(50)

assert(total, 150)

---

# Nucleoid checks whether every item is in stock

# There is an Item type
class Item:
    pass

# item1 is an Item that is in stock
item1 = Item()
item1.inStock = true

# item2 is an Item that is in stock
item2 = Item()
item2.inStock = true

# fulfillable is whether every item is in stock
fulfillable = Item.every(i => i.inStock == true)

assert(fulfillable, true)

# item2 is out of stock
item2.inStock = false

assert(fulfillable, false)

---

# Nucleoid checks whether any order is late

# There is an Order type
class Order:
    pass

# order1 is an Order that is on time
order1 = Order()
order1.late = false

# order2 is an Order that is on time
order2 = Order()
order2.late = false

# escalate is whether any order is late
escalate = Order.some(o => o.late == true)

assert(escalate, false)

# order2 is late
order2.late = true

assert(escalate, true)

---

# Nucleoid joins a breadcrumb into a single string

# levels is a list of "HOME", "SHOP" and "BOOKS"
levels = ["HOME", "SHOP", "BOOKS"]

# breadcrumb is levels joined with " / "
breadcrumb = levels.join(" / ")

assert(breadcrumb, "HOME / SHOP / BOOKS")

# Add "FICTION" to levels
levels.push("FICTION")

assert(breadcrumb, "HOME / SHOP / BOOKS / FICTION")

---

# Nucleoid creates a pick list for every open order

# There is an Order type
class Order:
    pass

# order1 is an Order
order1 = Order()

# order2 is an Order that is cancelled
order2 = Order()
order2.cancelled = true

# order3 is an Order
order3 = Order()

# There is a Pick type,
# which has an order as an Order
class Pick(order):
    this.order = order

# Any pick's kind is "WAVE"
$Pick.kind = "WAVE"

# For each order of Order, if the order is not cancelled,
# then there is a Pick whose order is the order
for order of Order:
    if not order.cancelled:
        Pick(order)

assert(Pick.length, 2)
assert(Pick[0].order.id, "order1")
assert(Pick[1].order.id, "order3")
assert(Pick[0].kind, "WAVE")

---

# Nucleoid rolls back a discount if a rule throws

# There is a Discount type
class Discount:
    pass

# If any discount's percent is greater than 75, then throw 'DISCOUNT_TOO_DEEP'
if $Discount.percent > 75:
    throw 'DISCOUNT_TOO_DEEP'

# discount1 is a Discount
discount1 = Discount()

try:
    # discount1's percent is 90
    discount1.percent = 90
catch error:
    assert(error, "DISCOUNT_TOO_DEEP")

assert(discount1.percent, null)

---

# Nucleoid refuses a cycle between a price and a total

# price is 12
price = 12

# total is price times 10
total = price * 10

assert(total, 120)

try:
    # price is total times 10
    price = total * 10
catch error:
    assert(error, TypeError("Circular Dependency"))

---

# Nucleoid validates a postcode with a regular expression

# There is a Delivery type
class Delivery:
    pass

# If any delivery's postcode does not match /[A-Z]{2}[0-9]/, then throw 'INVALID_POSTCODE'
if not /[A-Z]{2}[0-9]/.test($Delivery.postcode):
    throw 'INVALID_POSTCODE'

# delivery1 is a Delivery
delivery1 = Delivery()

assert(delivery1.postcode, null)

try:
    # delivery1's postcode is 'ab'
    delivery1.postcode = 'ab'
catch error:
    assert(error, "INVALID_POSTCODE")
```
