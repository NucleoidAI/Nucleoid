<h1 align="center">Nucleoid</h1>

<p align="center">
  <a href="https://www.apache.org/licenses/LICENSE-2.0">
    <img src="https://img.shields.io/badge/Apache-2.0-yellow?style=for-the-badge&logo=apache" alt="License" />
  </a>
  <a href="https://www.npmjs.com/package/nucleoidjs">
    <img src="https://img.shields.io/badge/NPM-red?style=for-the-badge&logo=npm" alt="NPM" />
  </a>
  <a href="https://discord.com/invite/eWXFCCuU5y">
    <img src="https://img.shields.io/badge/Discord-lightgrey?style=for-the-badge&logo=discord" alt="Discord" />
  </a>
</p>

[![Banner](.github/media/banner.png)](http://nucleoid.com/)

<p align="center">
  <strong>Build your APIs with the help of AI and built-in datastore</strong>
</p>

Nucleoid is a low-code framework for Node.js that as writing just like any other codes in Node.js, AI inside the runtime rerenders the very same JavaScript codes and makes the necessary adjustments in the state as well as stores on the disk so that your application doesn't require external database or anything else.

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

> :bulb: **This is pretty much it, thanks to AI in the runtime, only with this :point_up_2:, you successfully persisted your first object without external database.**

## Features

- Immediately start writing business logic
- Internal Data Management
- All you need is JavaScript
- Lighting fast

## Nucleoid IDE

Nucleoid IDE is a web interface that helps to run very same npm package with OpenAPI.

[Go to Nucleoid IDE](https://nucleoid.com/ide/)

[![Nucleoid IDE 2](https://cdn.nucleoid.com/media/ide-1.png)](https://nucleoid.com/ide/)
[![Nucleoid IDE 2](https://cdn.nucleoid.com/media/ide-2.png)](https://nucleoid.com/ide/)

### Under the hood: Declarative Runtime Environment

Nucleoid is a declarative runtime environment that applies declarative programming at the runtime as rerendering JavaScript statements and creating the graph, so as a result, the declarative runtime system isolates a behavior definition of a program from its technical instructions and executes declarative statements, which represent logical intention without carrying any technical detail.

# Join our [Thinkers Club](https://github.com/NucleoidJS/Nucleoid/discussions/categories/thinkers-club)

If you have an opinion, you are already a philosopher. We are working on brand-new approach to data and logic. Come join us in [discussions](https://github.com/NucleoidJS/Nucleoid/discussions/categories/thinkers-club).

[![Nobel](https://cdn.nucleoid.com/media/nobel.png)](https://github.com/NucleoidJS/Nucleoid/discussions/categories/thinkers-club)

| Pinned Discussions |
| ------------------ |

[![Discussion 25](https://cdn.nucleoid.com/media/discussion-25x500.png)](https://github.com/NucleoidJS/Nucleoid/discussions/25)
[![Discussion 26](https://cdn.nucleoid.com/media/discussion-26x500.png)](https://github.com/NucleoidJS/Nucleoid/discussions/26)
[![Discussion 28](https://cdn.nucleoid.com/media/discussion-28x500.png)](https://github.com/NucleoidJS/Nucleoid/discussions/28)

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
