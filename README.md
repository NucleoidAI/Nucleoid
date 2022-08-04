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

Nucleoid low-code framework lets you build your APIs with the help of AI and built-in datastore in declarative runtime environment.

As writing just like any other codes in Node.js, AI inside the runtime rerenders the very same JavaScript codes and makes the necessary adjustments in the state as well as stores on the disk so that your application doesn't require external database or anything else.

<br/>

## How it works

I. Write your business logic in JavaScript

II. Nucleoid renders your codes with AI

III. Creates APIs with built-in datastore

<br/>

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

<br/>

### Under the hood: Declarative Runtime Environment

Nucleoid is a declarative runtime environment that applies declarative programming at the runtime as rerendering JavaScript statements and creating the graph, so as a result, the declarative runtime system isolates a behavior definition of a program from its technical instructions and executes declarative statements, which represent logical intention without carrying any technical detail.

<br/>

## OpenAPI Integration with Nucleoid IDE

Nucleoid IDE is a web interface that helps to run very same npm package with OpenAPI.

[Go to Nucleoid IDE](https://nucleoid.com/ide/)

![Nucleoid IDE 1](https://cdn.nucleoid.com/media/screenshot-1.png)

![Nucleoid IDE 2](https://cdn.nucleoid.com/media/screenshot-2.png)

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

<br/>

---

<p align="center">
  <b>⭐️ Star us on GitHub for the support</b>
</p>

Thanks to declarative programming, we have a brand-new approach to data and logic. As we are still discovering what we can do with this powerful programming model, please join us with any types of contribution!

<p align="center">
  <img src="https://cdn.nucleoid.com/media/nobel.png" alt="Nobel" />
</p>

---
