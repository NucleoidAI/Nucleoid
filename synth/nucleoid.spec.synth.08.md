# Nucleoid Language Reference - Synthesized Use Cases 08

```

# Nucleoid converts a term length through a chain of variables

# days is 180
days = 180

# weeks is days divided by 5
weeks = days / 5

# terms is weeks divided by 12
terms = weeks / 12

assert(weeks, 36)
assert(terms, 3)

# days is 240
days = 240

assert(weeks, 48)
assert(terms, 4)

---

# Nucleoid replaces the source of a class size

# infants is 24
infants = 24

# juniors is 30
juniors = 30

# roll is infants plus 2
roll = infants + 2

assert(roll, 26)

# roll is juniors plus 2
roll = juniors + 2

assert(roll, 32)

# juniors is 32
juniors = 32

assert(roll, 34)

---

# Nucleoid adds to a credit total using its own value

# credits is 90
credits = 90

# credits is credits plus 30
credits = credits + 30

assert(credits, 120)

---

# Nucleoid fixes a pass mark with the value property

# passMark is 40
passMark = 40

# papers is 5
papers = 5

# threshold is passMark's value times papers
threshold = passMark.value * papers

assert(threshold, 200)

# passMark is 50
passMark = 50

assert(threshold, 200)

# papers is 6
papers = 6

assert(threshold, 240)

---

# Nucleoid clears a report when its mark is deleted

# marks is 80
marks = 80

# average is marks divided by 2
average = marks / 2

# report is average plus 10
report = average + 10

assert(report, 50)

# marks is deleted
delete marks

assert(average, null)
assert(report, null)

try:
    # marks
    marks
catch error:
    assert(error, ReferenceError("marks is not defined"))

---

# Nucleoid assigns a pass comparison to a variable

# score is 72
score = 72

# required is 60
required = 60

# passed is whether score is greater than required
passed = score > required

assert(passed, true)

# score is 55
score = 55

assert(passed, false)

---

# Nucleoid builds a timetable slot from a template literal

# subject is "maths"
subject = "maths"

# period is 3
period = 3

# slot is the subject and period in a template
slot = `${subject}-period-${period}`

assert(slot, "maths-period-3")

# period is 4
period = 4

assert(slot, "maths-period-4")

---

# Nucleoid combines enrolment flags with logical operators

# registered is true
registered = true

# suspended is false
suspended = false

assert(registered and not suspended, true)
assert(registered && !suspended, true)
assert(suspended or registered, true)
assert(suspended || registered, true)

---

# Nucleoid respects parentheses in a grading formula

# coursework is 2
coursework = 2

# exam is 3
exam = 3

# weighting is 4
weighting = 4

assert(coursework + exam * weighting, 14)
assert((coursework + exam) * weighting, 20)

---

# Nucleoid groups pupils with the remainder operator

# pupils is 29
pupils = 29

# perGroup is 6
perGroup = 6

# spare is pupils modulo perGroup
spare = pupils % perGroup

assert(spare, 5)

# pupils is 30
pupils = 30

assert(spare, 0)

---

# Nucleoid appends a year to a cohort code

# cohort is "YEAR-"
cohort = "YEAR-"

# level is 7
level = 7

# code is cohort plus level
code = cohort + level

assert(code, "YEAR-7")

# level is 8
level = 8

assert(code, "YEAR-8")

---

# Nucleoid counts the characters of a course code

# course is "MATH"
course = "MATH"

# width is course's length
width = course.length

assert(width, 4)

# course is "MATH101"
course = "MATH101"

assert(width, 7)

---

# Nucleoid lowercases a faculty name as a dependency

# faculty is "SCIENCE"
faculty = "SCIENCE"

# slug is faculty lowercased
slug = faculty.lower()

assert(slug, "science")

# faculty is "HUMANITIES"
faculty = "HUMANITIES"

assert(slug, "humanities")

---

# Nucleoid reads the stream letter of a set code

# set is "B2"
set = "B2"

# stream is the character of set at 0
stream = set.charAt(0)

assert(stream, "B")

# set is "C2"
set = "C2"

assert(stream, "C")

---

# Nucleoid rewrites a syllabus reference with replace

# syllabus is "unit/old/topic"
syllabus = "unit/old/topic"

# revision is "new"
revision = "new"

# current is syllabus with "old" replaced by revision
current = syllabus.replace("old", revision)

assert(current, "unit/new/topic")

# revision is "draft"
revision = "draft"

assert(current, "unit/draft/topic")

---

# Nucleoid takes the intake year from the end of a student number

# student is "STU-2019"
student = "STU-2019"

# intake is the last four characters of student
intake = student[-4:]

assert(intake, "2019")

# student is "STU-2020"
student = "STU-2020"

assert(intake, "2020")

---

# Nucleoid declares a course type with a constructor

# There is a Course type,
# which has a title as a string
class Course(title: str):
    this.title = title

# course1 is a Course whose title is "Algebra"
course1 = Course("Algebra")

assert(course1, { "id": "course1", "title": "Algebra" })
assert(Course.length, 1)

---

# Nucleoid declares a seminar as a subtype of a course

# There is a Course type,
# which has a title as a string
class Course(title: str):
    this.title = title

# There is a Seminar type,
# which is a subtype of Course
# and has a places as a number
class Seminar: Course
    def init(title, places):
        super(title)
        this.title = title
        this.places = places

# seminar1 is a Seminar whose title is "Logic" and whose places is 12
seminar1 = Seminar("Logic", 12)

assert(seminar1, { "id": "seminar1", "title": "Logic", "places": 12 })

---

# Nucleoid numbers every module with a class-level rule

# There is a Module type,
# which has a number as a number
class Module(number: int):
    this.number = number

# Any module's code is "MOD-" plus the module's number
$Module.code = "MOD-" + $Module.number

# module1 is a Module whose number is 1
module1 = Module(1)

assert(module1.code, "MOD-1")

# module2 is a Module whose number is 2
module2 = Module(2)

assert(module2.code, "MOD-2")

---

# Nucleoid replaces a class-level rule on a classroom

# There is a Classroom type
class Classroom:
    pass

# classroom1 is a Classroom whose number is 9
classroom1 = Classroom()
classroom1.number = 9

# Any classroom's sign is "C" plus the classroom's number
$Classroom.sign = "C" + $Classroom.number

assert(classroom1.sign, "C9")

# Any classroom's sign is "ROOM-" plus the classroom's number
$Classroom.sign = "ROOM-" + $Classroom.number

assert(classroom1.sign, "ROOM-9")

# classroom1's number is 10
classroom1.number = 10

assert(classroom1.sign, "ROOM-10")

---

# Nucleoid marks a distinction with a class-level conditional

# There is a Result type
class Result:
    pass

# Any result above 85 is a distinction
if $Result.mark > 85:
    $Result.distinction = true

# result1 is a Result whose mark is 70
result1 = Result()
result1.mark = 70

assert(result1.distinction, null)

# result1's mark is 90
result1.mark = 90

assert(result1.distinction, true)

---

# Nucleoid grades an assessment with a class-level chain

# There is an Assessment type
class Assessment:
    pass

# top is "A", middle is "B", and lower is "C"
top = "A"; middle = "B"; lower = "C"

# If any assessment's score is greater than 89, then the assessment's grade is top,
# else if the assessment's score is greater than 79, then the assessment's grade is middle,
# else the assessment's grade is lower
if $Assessment.score > 89:
    $Assessment.grade = top
else if $Assessment.score > 79:
    $Assessment.grade = middle
else:
    $Assessment.grade = lower

# assessment1 is an Assessment whose score is 82
assessment1 = Assessment()
assessment1.score = 82

assert(assessment1.grade, "B")

# middle is "MERIT"
middle = "MERIT"

assert(assessment1.grade, "MERIT")

# assessment1's score is 95
assessment1.score = 95

assert(assessment1.grade, "A")

---

# Nucleoid counts the pupils of a tutor with a class-level aggregate

# There is a Tutor type
class Tutor:
    pass

# There is a Pupil type
class Pupil:
    pass

# tutor1 is a Tutor
tutor1 = Tutor()

# pupil1 is a Pupil whose tutor is tutor1
pupil1 = Pupil()
pupil1.tutor = tutor1

# pupil2 is a Pupil whose tutor is tutor1
pupil2 = Pupil()
pupil2.tutor = tutor1

# Any tutor's group is the number of pupils whose tutor is the tutor
$Tutor.group = Pupil.filter(p => p.tutor == $Tutor).length

assert(tutor1.group, 2)

# pupil3 is a Pupil whose tutor is tutor1
pupil3 = Pupil()
pupil3.tutor = tutor1

assert(tutor1.group, 3)

---

# Nucleoid builds a certificate from a property that arrives later

# There is a Certificate type
class Certificate:
    pass

# certificate1 is a Certificate
certificate1 = Certificate()

# certificate1's full is "CERT" plus certificate1's serial
certificate1.full = "CERT" + certificate1.serial

assert(certificate1.full, null)

# certificate1's serial is "5512"
certificate1.serial = "5512"

assert(certificate1.full, "CERT5512")

---

# Nucleoid reads a teacher through a lesson reference

# There is a Lesson type
class Lesson:
    pass

# There is a Teacher type
class Teacher:
    pass

# lesson1 is a Lesson
lesson1 = Lesson()

# teacher1 is a Teacher whose name is "Mr Ford"
teacher1 = Teacher()
teacher1.name = "Mr Ford"

# lesson1's teacher is teacher1
lesson1.teacher = teacher1

# lesson1's taken is "Taken by " plus lesson1's teacher's name
lesson1.taken = "Taken by " + lesson1.teacher.name

assert(lesson1.taken, "Taken by Mr Ford")

# teacher1's name is "Ms Ade"
teacher1.name = "Ms Ade"

assert(lesson1.taken, "Taken by Ms Ade")

---

# Nucleoid clears a transcript line when a mark is deleted

# There is a Transcript type
class Transcript:
    pass

# transcript1 is a Transcript
transcript1 = Transcript()

# transcript1's paper is "P1"
transcript1.paper = "P1"

# transcript1's mark is "72"
transcript1.mark = "72"

# transcript1's line is transcript1's paper plus ":" plus transcript1's mark
transcript1.line = transcript1.paper + ":" + transcript1.mark

assert(transcript1.line, "P1:72")

# transcript1's mark is deleted
delete transcript1.mark

assert(transcript1.line, null)
assert(transcript1.paper, "P1")

---

# Nucleoid computes a teaching load in a block

# lessons is 60
lessons = 60

# load is null
load = null

# while in the block, weekly is a local variable that is lessons divided by 5,
# and load is weekly times 5
{
    weekly = lessons / 5
    load = weekly * 5
}

assert(load, 60)

# lessons is 80
lessons = 80

assert(load, 80)

---

# Nucleoid computes a hall capacity in a nested block

# rows is 8
rows = 8

# while in the block, block is a local variable that is rows squared,
# and in a nested block, capacity is block times 2
{
    block = Math.pow(rows, 2)
    {
        capacity = block * 2
    }
}

assert(capacity, 128)

---

# Nucleoid raises an attendance alert in a nested if inside a block

# expected is 30
expected = 30

# present is 18
present = 18

# notify is true
notify = true

# while in the block, absent is a local variable that is expected minus present,
# and if absent is greater than 10, then alert is notify
{
    absent = expected - present
    if absent > 10:
        alert = notify
}

assert(alert, true)

# notify is false
notify = false

assert(alert, false)

---

# Nucleoid picks a support level in a nested else inside a block

# needs is 2
needs = 2

# hours is 3
hours = 3

# high = "HIGH"
high = "HIGH"

# low is "LOW"
low = "LOW"

# while in the block, demand is a local variable that is needs times hours,
# and if demand is greater than 10, then support is high,
# else support is low
{
    demand = needs * hours
    if demand > 10:
        support = high
    else:
        support = low
}

assert(support, "LOW")

# low is "L"
low = "L"

assert(support, "L")

---

# Nucleoid computes a course fee in a class-level block

# There is a Programme type
class Programme:
    pass

# while in the block, levy is a local variable that is any programme's fee times 10 divided by 100,
# and the programme's total is the programme's fee plus levy
{
    levy = $Programme.fee * 10 / 100
    $Programme.total = $Programme.fee + levy
}

# programme1 is a Programme
programme1 = Programme()

assert(programme1.total, null)

# programme1's fee is 900
programme1.fee = 900

assert(programme1.total, 990)

---

# Nucleoid computes a timetable index in a nested class-level block

# There is a Term type
class Term:
    pass

# while in the block, span is a local variable that is 100 divided by any term's weeks,
# and in a nested block, the term's index is the floor of span times the term's lessons
{
    span = 100 / $Term.weeks
    {
        $Term.index = Math.floor(span * $Term.lessons)
    }
}

# term1 is a Term
term1 = Term()

# term1's weeks is 6
term1.weeks = 6

# term1's lessons is 5
term1.lessons = 5

assert(term1.index, 83)

---

# Nucleoid flags a resit with an if statement on a property

# There is a Paper type
class Paper:
    pass

# paper1 is a Paper whose outcome is "PASS"
paper1 = Paper()
paper1.outcome = "PASS"

# if paper1's outcome is "FAIL", then paper1's resit is true
if paper1.outcome == "FAIL":
    paper1.resit = true

assert(paper1.resit, null)

# paper1's outcome is "FAIL"
paper1.outcome = "FAIL"

assert(paper1.resit, true)

---

# Nucleoid chooses a report line with an else statement on a property

# There is a Pupil type
class Pupil:
    pass

# pupil1 is a Pupil whose gifted is false
pupil1 = Pupil()
pupil1.gifted = false

# extend is "EXTENSION WORK"
extend = "EXTENSION WORK"

# usual is "CORE WORK"
usual = "CORE WORK"

# if pupil1's gifted is true, then pupil1's plan is extend,
# else pupil1's plan is usual
if pupil1.gifted == true:
    pupil1.plan = extend
else:
    pupil1.plan = usual

assert(pupil1.plan, "CORE WORK")

# pupil1's gifted is true
pupil1.gifted = true

assert(pupil1.plan, "EXTENSION WORK")

---

# Nucleoid bands a bursary with multiple else if statements on a property

# There is an Application type
class Application:
    pass

# application1 is an Application whose income is 20
application1 = Application()
application1.income = 20

# unit is 10
unit = 10

# if application1's income is greater than 60, then application1's award is application1's income times unit minus 400,
# else if application1's income is greater than 30, then application1's award is application1's income times unit minus 100,
# else application1's award is application1's income times unit
if application1.income > 60:
    application1.award = application1.income * unit - 400
else if application1.income > 30:
    application1.award = application1.income * unit - 100
else:
    application1.award = application1.income * unit

assert(application1.award, 200)

# application1's income is 40
application1.income = 40

assert(application1.award, 300)

# application1's income is 70
application1.income = 70

assert(application1.award, 300)

---

# Nucleoid calls an average function in an assignment

# average returns the total divided by the count
def average(total, count):
    return total / count

# marks is 240
marks = 240

# papers is 4
papers = 4

# mean is the result of the average function call
mean = average(marks, papers)

assert(mean, 60)

# papers is 6
papers = 6

assert(mean, 40)

---

# Nucleoid updates a weighting when its function is redefined

# weight returns the mark times 2
def weight(mark):
    return mark * 2

# raw is 30
raw = 30

# weighted is the result of the weight function call with raw
weighted = weight(raw)

assert(weighted, 60)

# weight returns the mark times 3
def weight(mark):
    return mark * 3

assert(weighted, 90)

---

# Nucleoid nests a scale lookup inside a total function

# scale returns 4 times the level
def scale(level):
    return level * 4

# total returns the marks times the scale of the level
def total(marks, level):
    return marks * scale(level)

# rawMarks is 5
rawMarks = 5

# entryLevel is 3
entryLevel = 3

# points is the result of the total function call
points = total(rawMarks, entryLevel)

assert(points, 60)

---

# Nucleoid finds a grade with the three lambda forms

# grades is a list of 40, 60 and 80
grades = [40, 60, 80]

assert(grades.find(function(grade) { return grade == 80 }), 80)
assert(grades.find(grade => { return grade == 60 }), 60)
assert(grades.find(grade => grade == 40), 40)

---

# Nucleoid filters a year group by two thresholds

# There is a Learner type,
# which has a score as a number
class Learner(score: int):
    this.score = score

# There are Learners whose scores are 30, 50 and 70
Learner(30); Learner(50); Learner(70)

# ceiling is 60
ceiling = 60

# floorScore is 40
floorScore = 40

# middle is Learners whose score is above floorScore and below ceiling
middle = Learner.filter(l => l.score > floorScore).filter(l => l.score < ceiling)

assert(middle.length, 1)
assert(middle[0].score, 50)

# floorScore is 20
floorScore = 20

assert(middle.length, 2)
assert(middle[0].score, 30)

---

# Nucleoid maps raw marks into scaled marks

# raws is a list of 10, 20 and 30
raws = [10, 20, 30]

# scaling is 2
scaling = 2

# scaled is raws mapped to the raw times scaling
scaled = raws.map(r => r * scaling)

assert(scaled[0], 20)
assert(scaled[2], 60)

# scaling is 3
scaling = 3

assert(scaled[2], 90)

---

# Nucleoid reduces a set of marks into a total

# papers is a list of 15, 25 and 30
papers = [15, 25, 30]

# total is the sum of papers
total = papers.reduce((sum, paper) => sum + paper, 0)

assert(total, 70)

# Add 30 to papers
papers.push(30)

assert(total, 100)

---

# Nucleoid checks whether every pupil is registered

# There is a Pupil type
class Pupil:
    pass

# pupil1 is a Pupil who is registered
pupil1 = Pupil()
pupil1.registered = true

# pupil2 is a Pupil who is registered
pupil2 = Pupil()
pupil2.registered = true

# complete is whether every pupil is registered
complete = Pupil.every(p => p.registered == true)

assert(complete, true)

# pupil2 is not registered
pupil2.registered = false

assert(complete, false)

---

# Nucleoid checks whether any room is double booked

# There is a Room type
class Room:
    pass

# room1 is a Room that is free
room1 = Room()
room1.clash = false

# room2 is a Room that is free
room2 = Room()
room2.clash = false

# conflict is whether any room has a clash
conflict = Room.some(r => r.clash == true)

assert(conflict, false)

# room2 has a clash
room2.clash = true

assert(conflict, true)

---

# Nucleoid joins a syllabus into a single string

# topics is a list of "SETS", "LOGIC" and "PROOF"
topics = ["SETS", "LOGIC", "PROOF"]

# outline is topics joined with ", "
outline = topics.join(", ")

assert(outline, "SETS, LOGIC, PROOF")

# Add "GRAPHS" to topics
topics.push("GRAPHS")

assert(outline, "SETS, LOGIC, PROOF, GRAPHS")

---

# Nucleoid creates an entry for every enrolled learner

# There is a Learner type
class Learner:
    pass

# learner1 is a Learner
learner1 = Learner()

# learner2 is a Learner who has withdrawn
learner2 = Learner()
learner2.withdrawn = true

# learner3 is a Learner
learner3 = Learner()

# There is an Entry type,
# which has a learner as a Learner
class Entry(learner):
    this.learner = learner

# Any entry's kind is "EXAM"
$Entry.kind = "EXAM"

# For each learner of Learner, if the learner has not withdrawn,
# then there is an Entry whose learner is the learner
for learner of Learner:
    if not learner.withdrawn:
        Entry(learner)

assert(Entry.length, 2)
assert(Entry[0].learner.id, "learner1")
assert(Entry[1].learner.id, "learner3")
assert(Entry[0].kind, "EXAM")

---

# Nucleoid rolls back an entry if a rule throws

# There is an Entry type
class Entry:
    pass

# If any entry's age is less than 16, then throw 'TOO_YOUNG'
if $Entry.age < 16:
    throw 'TOO_YOUNG'

# entry1 is an Entry
entry1 = Entry()

try:
    # entry1's age is 14
    entry1.age = 14
catch error:
    assert(error, "TOO_YOUNG")

assert(entry1.age, null)

---

# Nucleoid refuses a cycle between a mark and a total

# mark is 20
mark = 20

# total is mark times 5
total = mark * 5

assert(total, 100)

try:
    # mark is total times 5
    mark = total * 5
catch error:
    assert(error, TypeError("Circular Dependency"))

---

# Nucleoid validates a candidate number with a regular expression

# There is a Candidate type
class Candidate:
    pass

# If any candidate's number does not match /[0-9]{4}/, then throw 'INVALID_CANDIDATE'
if not /[0-9]{4}/.test($Candidate.number):
    throw 'INVALID_CANDIDATE'

# candidate1 is a Candidate
candidate1 = Candidate()

assert(candidate1.number, null)

try:
    # candidate1's number is '12'
    candidate1.number = '12'
catch error:
    assert(error, "INVALID_CANDIDATE")
```
