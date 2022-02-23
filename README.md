# Nucleoid [![NPM](https://img.shields.io/npm/l/nucleoidjs)](https://www.apache.org/licenses/LICENSE-2.0) [![GitHub Workflow Status](https://img.shields.io/github/workflow/status/nucleoidjs/nucleoid/Test)](https://github.com/NucleoidJS/Nucleoid/actions/workflows/test.yml) [![npm](https://img.shields.io/npm/v/nucleoidjs)](https://www.npmjs.com/package/nucleoidjs) [![Discord](https://img.shields.io/discord/848931276559482900)](https://discord.gg/eWXFCCuU5y)

## What is Nucleoid?

Nucleoid is a low-code framework for JavaScript and embedded inside Node.js. As writing just like any other codes in
Node.js, it rerenders the very same JavaScript codes and makes the necessary adjustments in the state as well as stores
on the disk, so that your application doesn't require external database or anything else.

### ..but why?

Even simple applications today require lots of coding, libraries, tuning etc., and majority of them are technical
requirements rather than business logic. Declarative runtimes like Nucleoid lets you immediately start writing business
logic with less code lines.

### Nucleoid in a nutshell

![Look! Up in the sky!](https://drive.google.com/uc?export=view&id=1bNaHtwcxrKSTjlJw4RAVRw-ImkC86juX)

## Hello World

```javascript
const nucleoid = require("nucleoidjs");
const app = nucleoid();

class User {constructor(name){this.name = name}}
nucleoid.register(User);

// 👇 This is it!
app.post("/users", () => new User("Daphne"));

app.listen(3000);
```

It is pretty much it, you successfully persisted your first object with this :point_up_2:

> Just the reminder, you don't need external database, `const app = nucleoid()` will do the magic. :sunglasses:

Learn more at [nucleoid.com](https://nucleoid.com)

# Status

Track at [Trello](https://trello.com/b/TZ73H1Fk/nucleoid)

- [X] Working [beta](https://www.npmjs.com/package/nucleoidjs) is out, but still testing
- [X] ES6 support
- [ ] ES2018 support
- [ ] IDE (WiP)
- [ ] Production-ready

Please report an issue or ask a question at [Issues](https://github.com/NucleoidJS/Nucleoid/issues)
