<h1 align="center">
  Nucleoid
  <br />
  <img src="https://img.shields.io/badge/Apache-2.0-yellow?style=for-the-badge&logo=apache" alt="License" />
  <img src="https://img.shields.io/badge/NPM-red?style=for-the-badge&logo=npm" alt="npm" />
  <img src="https://img.shields.io/badge/Discord-lightgrey?style=for-the-badge&logo=discord" alt="Discord" />
</h1>

![Banner](.github/media/banner.png)

<h3 align="center">Build your APIs with the help of AI and built-in datastore</h3>

As writing just like any other codes in Node.js, it rerenders the very same JavaScript codes and makes the necessary adjustments in the state as well as stores on the disk, so that your application doesn't require external database or anything else.

## How it works

I. Write your business logic in JavaScript

II. Nucleoid renders your codes with AI

III. Creates APIs with built-in datastore

## Hello World

```shell
> npm i nucleoidjs
```

Once installed, you can simply run with Express.js

```javascript
const app = nucleoid();

class User {
  constructor(name) {
    this.name = name;
  }
}

// 👇 This is it!
app.post("/users", () => {
  new User("Daphne");
});

app.listen(3000);
```

**This is pretty much it, thanks to AI in the runtime, you successfully persisted your first object with this :point_up_2: without external database**

## Features

- Immediately start writing business logic
- Internal Data Management
- All you need is JavaScript
- Lighting fast

## Nucleoid IDE

Nucleoid IDE is a web interface that helps to run very same npm package with OpenAPI.

[Go to Nucleoid IDE](https://nucleoid.com/ide/)

![Nucleoid IDE 2](https://cdn.nucleoid.com/media/ide-1.png)
![Nucleoid IDE 2](https://cdn.nucleoid.com/media/ide-2.png)

### Under the hood: Declarative Runtime Environment

The declarative runtime environment isolates a behavior definition of a program from its technical instructions and executes declarative statements, which represent logical intention without carrying any technical details. In this paradigm, there is no segregation regarding what data is or not, instead approaches how data is related with others so that any type of data including business rules can be added without requiring any additional actions such as compiling, configuring, restarting as a result of plasticity.

# Join [Thinkers Club](https://github.com/NucleoidJS/Nucleoid/discussions/categories/thinkers-club)

If you have an opinion, you are already a philosopher. We are working on brand-new approach to data and logic. Come join us in [discussions](https://github.com/NucleoidJS/Nucleoid/discussions/categories/thinkers-club).

![Nobel](https://cdn.nucleoid.com/media/nobel.png)

| Pinned Discussions                                    |
| ----------------------------------------------------- |
| https://github.com/NucleoidJS/Nucleoid/discussions/11 |

## Project Status

Track at [Trello](https://trello.com/b/TZ73H1Fk/nucleoid)

- [x] [Beta](https://www.npmjs.com/package/nucleoidjs) is out
- [x] ES6 support
- [ ] ES2018 support
- [ ] ES2020 support
- [ ] TypeScript
- [ ] [IDE](https://github.com/NucleoidJS/IDE) (WiP)
- [ ] Production-ready

Please report an [issue](https://github.com/NucleoidJS/Nucleoid/issues) or ask a question at [Discussions](https://github.com/NucleoidJS/Nucleoid/discussions)

Learn more at [nucleoid.com](https://nucleoid.com)
