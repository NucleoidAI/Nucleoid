# Nucleoid Language Reference - Synthesized Use Cases 07

```

# Nucleoid converts a temperature through a chain of variables

# celsius is 100
celsius = 100

# fahrenheit is celsius times 9 divided by 5 plus 32
fahrenheit = celsius * 9 / 5 + 32

# kelvin is celsius plus 273
kelvin = celsius + 273

assert(fahrenheit, 212)
assert(kelvin, 373)

# celsius is 25
celsius = 25

assert(fahrenheit, 77)
assert(kelvin, 298)

---

# Nucleoid replaces the source of a dosage

# adult is 500
adult = 500

# child is 250
child = 250

# dose is adult divided by 2
dose = adult / 2

assert(dose, 250)

# dose is child divided by 2
dose = child / 2

assert(dose, 125)

# child is 300
child = 300

assert(dose, 150)

---

# Nucleoid increases a bed count using its own value

# beds is 120
beds = 120

# beds is beds plus 30
beds = beds + 30

assert(beds, 150)

---

# Nucleoid fixes a baseline reading with the value property

# baseline is 70
baseline = 70

# minutes is 4
minutes = 4

# total is baseline's value times minutes
total = baseline.value * minutes

assert(total, 280)

# baseline is 80
baseline = 80

assert(total, 280)

# minutes is 6
minutes = 6

assert(total, 420)

---

# Nucleoid clears a chart when its measurement is deleted

# height is 180
height = 180

# metres is height divided by 100
metres = height / 100

# index is metres times 2
index = metres * 2

assert(index, 3.6)

# height is deleted
delete height

assert(metres, null)
assert(index, null)

---

# Nucleoid assigns a fever comparison to a variable

# reading is 39
reading = 39

# limit is 38
limit = 38

# fever is whether reading is greater than limit
fever = reading > limit

assert(fever, true)

# reading is 37
reading = 37

assert(fever, false)

---

# Nucleoid builds a patient reference from a template literal

# ward is "A"
ward = "A"

# bed is 12
bed = 12

# reference is the ward and bed in a template
reference = `WARD-${ward}-BED-${bed}`

assert(reference, "WARD-A-BED-12")

# bed is 14
bed = 14

assert(reference, "WARD-A-BED-14")

---

# Nucleoid combines screening flags with logical operators

# symptomatic is false
symptomatic = false

# exposed is true
exposed = true

assert(symptomatic or exposed, true)
assert(symptomatic && exposed, false)
assert(not symptomatic and exposed, true)
assert(!symptomatic || exposed, true)

---

# Nucleoid respects parentheses in a scoring formula

# vital is 2
vital = 2

# risk is 3
risk = 3

# weightings is 4
weightings = 4

assert(vital + risk * weightings, 14)
assert((vital + risk) * weightings, 20)

---

# Nucleoid divides doses with the remainder operator

# tablets is 31
tablets = 31

# perDay is 4
perDay = 4

# leftover is tablets modulo perDay
leftover = tablets % perDay

assert(leftover, 3)

# tablets is 33
tablets = 33

assert(leftover, 1)

---

# Nucleoid appends a sequence to a sample label

# prefix is "SMP-"
prefix = "SMP-"

# number is 8
number = 8

# label is prefix plus number
label = prefix + number

assert(label, "SMP-8")

# number is 9
number = 9

assert(label, "SMP-9")

---

# Nucleoid counts the characters of a record number

# record is "NHS12"
record = "NHS12"

# digits is record's length
digits = record.length

assert(digits, 5)

# record is "NHS123456"
record = "NHS123456"

assert(digits, 9)

---

# Nucleoid lowercases a department name as a dependency

# department is "CARDIOLOGY"
department = "CARDIOLOGY"

# key is department lowercased
key = department.lower()

assert(key, "cardiology")

# department is "NEUROLOGY"
department = "NEUROLOGY"

assert(key, "neurology")

---

# Nucleoid reads the first letter of a blood group

# group is "AB"
group = "AB"

# lead is the character of group at 0
lead = group.charAt(0)

assert(lead, "A")

# group is "OB"
group = "OB"

assert(lead, "O")

---

# Nucleoid rewrites a clinic code with replace

# code is "CL-OLD-01"
code = "CL-OLD-01"

# state is "NEW"
state = "NEW"

# updated is code with "OLD" replaced by state
updated = code.replace("OLD", state)

assert(updated, "CL-NEW-01")

# state is "TMP"
state = "TMP"

assert(updated, "CL-TMP-01")

---

# Nucleoid takes the year from the end of an admission stamp

# stamp is "ADM-08-2021"
stamp = "ADM-08-2021"

# year is the last four characters of stamp
year = stamp[-4:]

assert(year, "2021")

# stamp is "ADM-09-2022"
stamp = "ADM-09-2022"

assert(year, "2022")

---

# Nucleoid declares a patient type with a constructor

# There is a Patient type,
# which has a name as a string
class Patient(name: str):
    this.name = name

# patient1 is a Patient whose name is "Iris"
patient1 = Patient("Iris")

assert(patient1, { "id": "patient1", "name": "Iris" })
assert(Patient.length, 1)

---

# Nucleoid declares an inpatient as a subtype of a patient

# There is a Patient type,
# which has a name as a string
class Patient(name: str):
    this.name = name

# There is an Inpatient type,
# which is a subtype of Patient
# and has a ward as a string
class Inpatient: Patient
    def init(name, ward):
        super(name)
        this.name = name
        this.ward = ward

# inpatient1 is an Inpatient whose name is "Rui" and whose ward is "B"
inpatient1 = Inpatient("Rui", "B")

assert(inpatient1, { "id": "inpatient1", "name": "Rui", "ward": "B" })

---

# Nucleoid identifies every sample with a class-level rule

# There is a Sample type,
# which has a number as a number
class Sample(number: int):
    this.number = number

# Any sample's tag is "S-" plus the sample's number
$Sample.tag = "S-" + $Sample.number

# sample1 is a Sample whose number is 12
sample1 = Sample(12)

assert(sample1.tag, "S-12")

# sample2 is a Sample whose number is 13
sample2 = Sample(13)

assert(sample2.tag, "S-13")

---

# Nucleoid replaces a class-level rule on a clinic

# There is a Clinic type
class Clinic:
    pass

# clinic1 is a Clinic whose room is 5
clinic1 = Clinic()
clinic1.room = 5

# Any clinic's sign is "R" plus the clinic's room
$Clinic.sign = "R" + $Clinic.room

assert(clinic1.sign, "R5")

# Any clinic's sign is "ROOM-" plus the clinic's room
$Clinic.sign = "ROOM-" + $Clinic.room

assert(clinic1.sign, "ROOM-5")

# clinic1's room is 6
clinic1.room = 6

assert(clinic1.sign, "ROOM-6")

---

# Nucleoid marks a reading as high with a class-level conditional

# There is a Reading type
class Reading:
    pass

# Any reading above 140 is high
if $Reading.systolic > 140:
    $Reading.high = true

# reading1 is a Reading whose systolic is 120
reading1 = Reading()
reading1.systolic = 120

assert(reading1.high, null)

# reading1's systolic is 160
reading1.systolic = 160

assert(reading1.high, true)

---

# Nucleoid triages a case with a class-level chain

# There is a Case type
class Case:
    pass

# urgent is "URGENT", soon is "SOON", and routine is "ROUTINE"
urgent = "URGENT"; soon = "SOON"; routine = "ROUTINE"

# If any case's score is greater than 8, then the case's priority is urgent,
# else if the case's score is greater than 4, then the case's priority is soon,
# else the case's priority is routine
if $Case.score > 8:
    $Case.priority = urgent
else if $Case.score > 4:
    $Case.priority = soon
else:
    $Case.priority = routine

# case1 is a Case whose score is 6
case1 = Case()
case1.score = 6

assert(case1.priority, "SOON")

# soon is "WITHIN A WEEK"
soon = "WITHIN A WEEK"

assert(case1.priority, "WITHIN A WEEK")

# case1's score is 9
case1.score = 9

assert(case1.priority, "URGENT")

---

# Nucleoid counts the appointments of a doctor with a class-level aggregate

# There is a Doctor type
class Doctor:
    pass

# There is an Appointment type
class Appointment:
    pass

# doctor1 is a Doctor
doctor1 = Doctor()

# appointment1 is an Appointment whose doctor is doctor1
appointment1 = Appointment()
appointment1.doctor = doctor1

# appointment2 is an Appointment whose doctor is doctor1
appointment2 = Appointment()
appointment2.doctor = doctor1

# Any doctor's booked is the number of appointments whose doctor is the doctor
$Doctor.booked = Appointment.filter(a => a.doctor == $Doctor).length

assert(doctor1.booked, 2)

# appointment3 is an Appointment whose doctor is doctor1
appointment3 = Appointment()
appointment3.doctor = doctor1

assert(doctor1.booked, 3)

---

# Nucleoid builds a prescription from a property that arrives later

# There is a Prescription type
class Prescription:
    pass

# prescription1 is a Prescription
prescription1 = Prescription()

# prescription1's full is "RX" plus prescription1's serial
prescription1.full = "RX" + prescription1.serial

assert(prescription1.full, null)

# prescription1's serial is "40021"
prescription1.serial = "40021"

assert(prescription1.full, "RX40021")

---

# Nucleoid reads a consultant through a ward reference

# There is a Ward type
class Ward:
    pass

# There is a Consultant type
class Consultant:
    pass

# ward1 is a Ward
ward1 = Ward()

# consultant1 is a Consultant whose name is "Dr Chen"
consultant1 = Consultant()
consultant1.name = "Dr Chen"

# ward1's consultant is consultant1
ward1.consultant = consultant1

# ward1's lead is "Lead: " plus ward1's consultant's name
ward1.lead = "Lead: " + ward1.consultant.name

assert(ward1.lead, "Lead: Dr Chen")

# consultant1's name is "Dr Silva"
consultant1.name = "Dr Silva"

assert(ward1.lead, "Lead: Dr Silva")

---

# Nucleoid clears a diagnosis summary when a finding is deleted

# There is a Diagnosis type
class Diagnosis:
    pass

# diagnosis1 is a Diagnosis
diagnosis1 = Diagnosis()

# diagnosis1's finding is "STABLE"
diagnosis1.finding = "STABLE"

# diagnosis1's stage is "I"
diagnosis1.stage = "I"

# diagnosis1's summary is diagnosis1's finding plus " " plus diagnosis1's stage
diagnosis1.summary = diagnosis1.finding + " " + diagnosis1.stage

assert(diagnosis1.summary, "STABLE I")

# diagnosis1's finding is deleted
delete diagnosis1.finding

assert(diagnosis1.summary, null)
assert(diagnosis1.stage, "I")

---

# Nucleoid computes a fluid balance in a block

# intake is 2400
intake = 2400

# balance is null
balance = null

# while in the block, hourly is a local variable that is intake divided by 24,
# and balance is hourly times 24
{
    hourly = intake / 24
    balance = hourly * 24
}

assert(balance, 2400)

# intake is 3600
intake = 3600

assert(balance, 3600)

---

# Nucleoid computes a surface area in a nested block

# side is 3
side = 3

# while in the block, face is a local variable that is side squared,
# and in a nested block, area is face times 6
{
    face = Math.pow(side, 2)
    {
        area = face * 6
    }
}

assert(area, 54)

---

# Nucleoid raises a sepsis alert in a nested if inside a block

# pulse is 130
pulse = 130

# resting is 70
resting = 70

# warning is true
warning = true

# while in the block, rise is a local variable that is pulse minus resting,
# and if rise is greater than 40, then alert is warning
{
    rise = pulse - resting
    if rise > 40:
        alert = warning
}

assert(alert, true)

# warning is false
warning = false

assert(alert, false)

---

# Nucleoid picks a care level in a nested else inside a block

# dependency is 2
dependency = 2

# mobility is 3
mobility = 3

# intensive is "INTENSIVE"
intensive = "INTENSIVE"

# standard is "STANDARD"
standard = "STANDARD"

# while in the block, need is a local variable that is dependency times mobility,
# and if need is greater than 10, then care is intensive,
# else care is standard
{
    need = dependency * mobility
    if need > 10:
        care = intensive
    else:
        care = standard
}

assert(care, "STANDARD")

# standard is "STD"
standard = "STD"

assert(care, "STD")

---

# Nucleoid computes a dosage in a class-level block

# There is a Course type
class Course:
    pass

# while in the block, daily is a local variable that is any course's dose times 3,
# and the course's total is daily times the course's days
{
    daily = $Course.dose * 3
    $Course.total = daily * $Course.days
}

# course1 is a Course
course1 = Course()

assert(course1.total, null)

# course1's dose is 200
course1.dose = 200

# course1's days is 5
course1.days = 5

assert(course1.total, 3000)

---

# Nucleoid computes a bay allocation in a nested class-level block

# There is a Unit type
class Unit:
    pass

# while in the block, share is a local variable that is 60 divided by any unit's staff,
# and in a nested block, the unit's minutes is the floor of share times the unit's patients
{
    share = 60 / $Unit.staff
    {
        $Unit.minutes = Math.floor(share * $Unit.patients)
    }
}

# unit1 is a Unit
unit1 = Unit()

# unit1's staff is 7
unit1.staff = 7

# unit1's patients is 4
unit1.patients = 4

assert(unit1.minutes, 34)

---

# Nucleoid flags an allergy with an if statement on a property

# There is a Chart type
class Chart:
    pass

# chart1 is a Chart whose allergy is "NONE"
chart1 = Chart()
chart1.allergy = "NONE"

# if chart1's allergy is "PENICILLIN", then chart1's warn is true
if chart1.allergy == "PENICILLIN":
    chart1.warn = true

assert(chart1.warn, null)

# chart1's allergy is "PENICILLIN"
chart1.allergy = "PENICILLIN"

assert(chart1.warn, true)

---

# Nucleoid chooses a discharge note with an else statement on a property

# There is a Stay type
class Stay:
    pass

# stay1 is a Stay whose complications is false
stay1 = Stay()
stay1.complications = false

# review is "REVIEW IN CLINIC"
review = "REVIEW IN CLINIC"

# routine is "DISCHARGE HOME"
routine = "DISCHARGE HOME"

# if stay1's complications is true, then stay1's note is review,
# else stay1's note is routine
if stay1.complications == true:
    stay1.note = review
else:
    stay1.note = routine

assert(stay1.note, "DISCHARGE HOME")

# stay1's complications is true
stay1.complications = true

assert(stay1.note, "REVIEW IN CLINIC")

---

# Nucleoid bands a waiting time with multiple else if statements on a property

# There is a Referral type
class Referral:
    pass

# referral1 is a Referral whose weeks is 3
referral1 = Referral()
referral1.weeks = 3

# step is 10
step = 10

# if referral1's weeks is greater than 18, then referral1's breach is referral1's weeks times step plus 100,
# else if referral1's weeks is greater than 6, then referral1's breach is referral1's weeks times step plus 50,
# else referral1's breach is referral1's weeks times step
if referral1.weeks > 18:
    referral1.breach = referral1.weeks * step + 100
else if referral1.weeks > 6:
    referral1.breach = referral1.weeks * step + 50
else:
    referral1.breach = referral1.weeks * step

assert(referral1.breach, 30)

# referral1's weeks is 10
referral1.weeks = 10

assert(referral1.breach, 150)

# referral1's weeks is 20
referral1.weeks = 20

assert(referral1.breach, 300)

---

# Nucleoid calls a body mass function in an assignment

# mass returns the weight divided by the height squared
def mass(weight, height):
    return weight / (height * height)

# kilos is 80
kilos = 80

# metres is 2
metres = 2

# bmi is the result of the mass function call
bmi = mass(kilos, metres)

assert(bmi, 20)

# kilos is 100
kilos = 100

assert(bmi, 25)

---

# Nucleoid updates a rate when its function is redefined

# rate returns the count times 60 divided by the minutes
def rate(count, minutes):
    return count * 60 / minutes

# beats is 30
beats = 30

# span is 2
span = 2

# perMinute is the result of the rate function call
perMinute = rate(beats, span)

assert(perMinute, 900)

# rate returns the count times 30 divided by the minutes
def rate(count, minutes):
    return count * 30 / minutes

assert(perMinute, 450)

---

# Nucleoid nests a factor lookup inside a dosage function

# factor returns 5 times the band
def factor(band):
    return band * 5

# dosage returns the weight times the factor of the band
def dosage(weight, band):
    return weight * factor(band)

# bodyWeight is 4
bodyWeight = 4

# ageBand is 2
ageBand = 2

# millilitres is the result of the dosage function call
millilitres = dosage(bodyWeight, ageBand)

assert(millilitres, 40)

---

# Nucleoid finds a reading with the three lambda forms

# levels is a list of 5, 10 and 15
levels = [5, 10, 15]

assert(levels.find(function(level) { return level == 15 }), 15)
assert(levels.find(level => { return level == 10 }), 10)
assert(levels.find(level => level == 5), 5)

---

# Nucleoid filters a cohort by two thresholds

# There is a Subject type,
# which has an age as a number
class Subject(age: int):
    this.age = age

# There are Subjects whose ages are 20, 40 and 60
Subject(20); Subject(40); Subject(60)

# oldest is 50
oldest = 50

# youngest is 30
youngest = 30

# cohort is Subjects whose age is above youngest and below oldest
cohort = Subject.filter(s => s.age > youngest).filter(s => s.age < oldest)

assert(cohort.length, 1)
assert(cohort[0].age, 40)

# youngest is 10
youngest = 10

assert(cohort.length, 2)
assert(cohort[0].age, 20)

---

# Nucleoid maps sample readings into corrected values

# samples is a list of 2, 4 and 6
samples = [2, 4, 6]

# calibration is 3
calibration = 3

# corrected is samples mapped to the sample times calibration
corrected = samples.map(s => s * calibration)

assert(corrected[0], 6)
assert(corrected[2], 18)

# calibration is 5
calibration = 5

assert(corrected[2], 30)

---

# Nucleoid reduces a set of readings into a total

# readings is a list of 12, 18 and 20
readings = [12, 18, 20]

# total is the sum of readings
total = readings.reduce((sum, reading) => sum + reading, 0)

assert(total, 50)

# Add 10 to readings
readings.push(10)

assert(total, 60)

---

# Nucleoid checks whether every screening is complete

# There is a Screening type
class Screening:
    pass

# screening1 is a Screening that is complete
screening1 = Screening()
screening1.complete = true

# screening2 is a Screening that is complete
screening2 = Screening()
screening2.complete = true

# cleared is whether every screening is complete
cleared = Screening.every(s => s.complete == true)

assert(cleared, true)

# screening2 is not complete
screening2.complete = false

assert(cleared, false)

---

# Nucleoid checks whether any bed is occupied

# There is a Bed type
class Bed:
    pass

# bed1 is a Bed that is free
bed1 = Bed()
bed1.occupied = false

# bed2 is a Bed that is free
bed2 = Bed()
bed2.occupied = false

# busy is whether any bed is occupied
busy = Bed.some(b => b.occupied == true)

assert(busy, false)

# bed2 is occupied
bed2.occupied = true

assert(busy, true)

---

# Nucleoid joins a care pathway into a single string

# steps is a list of "TRIAGE", "REVIEW" and "DISCHARGE"
steps = ["TRIAGE", "REVIEW", "DISCHARGE"]

# pathway is steps joined with " > "
pathway = steps.join(" > ")

assert(pathway, "TRIAGE > REVIEW > DISCHARGE")

# Add "FOLLOWUP" to steps
steps.push("FOLLOWUP")

assert(pathway, "TRIAGE > REVIEW > DISCHARGE > FOLLOWUP")

---

# Nucleoid creates a record for every active patient

# There is a Patient type
class Patient:
    pass

# patient1 is a Patient
patient1 = Patient()

# patient2 is a Patient who is discharged
patient2 = Patient()
patient2.discharged = true

# patient3 is a Patient
patient3 = Patient()

# There is a Record type,
# which has a patient as a Patient
class Record(patient):
    this.patient = patient

# Any record's kind is "DAILY"
$Record.kind = "DAILY"

# For each patient of Patient, if the patient is not discharged,
# then there is a Record whose patient is the patient
for patient of Patient:
    if not patient.discharged:
        Record(patient)

assert(Record.length, 2)
assert(Record[0].patient.id, "patient1")
assert(Record[1].patient.id, "patient3")
assert(Record[0].kind, "DAILY")

---

# Nucleoid rolls back an observation if a rule throws

# There is an Observation type
class Observation:
    pass

# If any observation's oxygen is less than 80, then throw 'CRITICAL'
if $Observation.oxygen < 80:
    throw 'CRITICAL'

# observation1 is an Observation
observation1 = Observation()

try:
    # observation1's oxygen is 70
    observation1.oxygen = 70
catch error:
    assert(error, "CRITICAL")

assert(observation1.oxygen, null)

---

# Nucleoid refuses a cycle between a dose and a course

# dose is 50
dose = 50

# course is dose times 4
course = dose * 4

assert(course, 200)

try:
    # dose is course times 4
    dose = course * 4
catch error:
    assert(error, TypeError("Circular Dependency"))

---

# Nucleoid validates a patient number with a regular expression

# There is a File type
class File:
    pass

# If any file's number does not match /[0-9]{6}/, then throw 'INVALID_NUMBER'
if not /[0-9]{6}/.test($File.number):
    throw 'INVALID_NUMBER'

# file1 is a File
file1 = File()

assert(file1.number, null)

try:
    # file1's number is '123'
    file1.number = '123'
catch error:
    assert(error, "INVALID_NUMBER")
```
