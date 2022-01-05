# Nucleoid ![NPM](https://img.shields.io/npm/l/nucleoidjs) ![npm](https://img.shields.io/npm/v/nucleoidjs) ![GitHub Workflow Status](https://img.shields.io/github/workflow/status/nucleoidjs/nucleoid/Test)

## What is Nucleoid?

Nucleoid is a state-based data storage with vanilla JavaScript. Nucleoid runtime is embedded inside Node.js and as
writing just any other codes in Node.js, it rerenders the same JavaScript codes and makes the necessary adjustments
in the state as well as stores on the disk, so that your application doesn't require external database.

### ...but why?

Even simple applications today require lots of coding, libraries, tuning etc., and majority of them are technical codes
rather than business logic. Declarative runtimes like Nucleoid can organically reduce numbers of code lines needed.

### Nucleoid in a nutshell

![Look! Up in the sky!](https://drive.google.com/uc?export=view&id=1bNaHtwcxrKSTjlJw4RAVRw-ImkC86juX)

## Hello World

```javascript
const nucleoid = require("nucleoidjs");
const app = nucleoid();

class User {}
nucleoid.register(User);

app.post("/users", () => new User());

app.listen(3000);
```

It is pretty much it, you successfully persisted your first object with this :point_up_2:

> Just the reminder, you don't need external database, `const app = nucleoid()` will do the magic.

<br/>

This passes HTTP information into the runtime

```javascript
class User {
  constructor(name) {
    this.name = name;
  }
}
nucleoid.register(User);

app.post("/users", (req) => new User(req.body.name));

app.get("/users", (req) => User.filter((user) => user.name === req.query.name));
```

<br/>

...and CRUD operations:

```javascript
app.post("/users", (req) => new User(req.body.name));

app.get("/users/:id", (req) => User[req.params.id]);

app.post("/users/:id", (req) => {
  let user = User[req.params.id];

  if (user) {
    user.name = req.body.name;
    return user;
  }
});

app.delete("/users/:id", (req) => delete User[req.params.id]);
```

<br/>

Nucleoid also opens terminal channel at `8448` port for queries like in SQL, so that you can write code snippet for data operations

![Terminal](https://media.giphy.com/media/aGQyuZ4ggB4SaPRc1g/giphy.gif)

In the meanwhile, you can still call underlying Express APIs for non-Nucleoidic functions

```javascript
const app = nucleoid();

const express = app.express();

express.get("/test", (req, res) => res.send("Hello!"));
```

---

### Declarative Runtime Environment

The declarative runtime environment isolates a behavior definition of a program from its technical instructions and
executes declarative statements, which represent logical intention without carrying any technical details. In this
paradigm, there is no segregation regarding what data is or not, instead approaches how data is related with others so
that any type of data including business rules can be added without requiring any additional actions such as compiling,
configuring, restarting as a result of plasticity.

### Syntax and Semantics

In declarative programming, as its name suggests, it runs based on definition of syntax, where a syntax is often
followed by developers in order to achieve behaviors in specifications. For example:

```
> a = 1
> b = a * 2
```

`=` represents assignment of expression to a variable. In Nucleoid runtime, it follows formal logic as semantics in ES6
syntax. So, if the expression contains another variable, it is considered as dependency, and by the design, whenever the
variable is changed, the dependent variable also has to change along with, otherwise it breaks its logical integrity.

However, in imperative programming, for the same example, when `a` is changed, it changes memory location where `a`
refers, but the change doesn't alter variable `b` because in the context of IP, it is just a representation of memory
location.

### Control Flow

An important difference in declarative programming oppose to imperative is who manages the control flow. In IP, a
programmer has full control of instructions that runs on CPU through programming language, but in declarative
programming, it is based on semantics, and in Nucleoid, it follows formal logic as semantics.

Due to its plasticity, the runtime is able to adjust control flow as receives more statements, so that externalized
configuration files are optional.

### Persistency

Nucleoid runtime cumulatively stores statements in order as received so that the runtime doesn't require external
database. This feature is enabled by declarative programming as a result of plasticity and lowers complexity of the
system along with gaining better performance since there is no network communication required.

Learn more at https://nucleoid.org/tutorial/
