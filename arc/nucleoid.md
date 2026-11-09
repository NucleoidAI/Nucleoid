# Every human is mortal. Socrates is a human.
class Human:
    pass

$Human.mortal = true

socrates = Human()
socrates.mortal

# Area follows changes to a rectangle's dimensions.
class Rectangle(this.width, this.height)

$Rectangle.area = $Rectangle.width * $Rectangle.height

rectangle = Rectangle(3, 4)
rectangle.width = 5
rectangle.area

# Find the names of students with passing scores.
class Student(this.name, this.score)

alice = Student("Alice", 80)
bob = Student("Bob", 50)

passing = Student.filter(student => student.score >= 60)
passing.map(student => student.name)

# Derive department payroll from related employees.
class Department(this.name)
class Employee(this.department, this.salary)

$Department.payroll = Employee.filter(e => e.department == $Department).sum(e => e.salary)

engineering = Department("Engineering")
alice = Employee(engineering, 100)
bob = Employee(engineering, 120)

alice.salary = 150
engineering.payroll

# Subtype rules retain precedence when a parent rule changes.
class Employee(this.salary)
class Manager extends Employee

$Employee.bonus = $Employee.salary * 0.1
$Manager.bonus = $Manager.salary * 0.2

worker = Employee(100)
manager = Manager(100)

# Every task must have at least one suitably skilled worker.
class Task(this.skill)
class Worker(this.skill)

review = Task("review")
build = Task("build")
reviewer = Worker("review")

canCover = all t in Task: any w in Worker: w.skill == t.skill

builder = Worker("build")
canCover

# Explain how a particular input contributes to a result.
price = 10
quantity = 3
shipping = 5
subtotal = price * quantity
total = subtotal + shipping

trace = total |> why |> path(price)

price = 12
trace.map(step => [step.node, step.holds])

# Explore a hypothetical change without changing the real model.
price = 10
quantity = 3
total = price * quantity

forecast = suppose quantity = 4: total
changed = suppose quantity = 4:
(model |> diff).map(step => step.node)

# := captures once; object references still point to live objects.
class Item(this.price)
item = Item(10)

livePrice = item.price
capturedPrice := item.price
capturedItem := item

item.price = 20

# Read stored history through read-only version selectors.
balance = 100
balance = 120
balance = 90

# Declare a relationship before its input properties exist.
class Order
$Order.total = $Order.price * $Order.quantity

order = Order()
before := order.total

order.price = 10
order.quantity = 3

# An empty population satisfies all, but not any.
class Check(this.passed)

allPassed = all check in Check: check.passed
hasFailure = any check in Check: not check.passed
empty := [allPassed, hasFailure]

