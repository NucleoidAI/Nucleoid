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

[![Banner](.github/media/banner.gif)](http://nucleoid.com/)

<p align="center">
  Low-code Framework for Node.js: Building APIs with the help of AI and built-in datastore
</p>

<br/>

Nucleoid uses symbolic (logic) AI in the runtime that tracks given statements in JavaScript and creates relationships between variables, objects, and functions etc. in the graph. So, as writing just like any other codes in Node.js, the runtime translates your business logic to fully working application by managing the JS state as well as stores in the built-in data store, so that your application doesn't require external database or anything else.

<p align="center">
  <img src=".github/media/graph.gif" width="450" alt="Graph Animation"/>
</p>

## How it works

I. Write your business logic in JavaScript (TypeScript support coming soon)

II. Nucleoid runtime renders your codes

III. Creates APIs with the built-in datastore

<br/>

## Hello World :zap:

```shell
> npm i nucleoidjs
```

Once installed, you can simply run with Express.js

```javascript
const nucleoid = require("nucleoidjs");
const app = nucleoid();

class Item {
  constructor(name, barcode) {
    this.name = name;
    this.barcode = barcode;
  }
}
nucleoid.register(Item);

// 👍 Only needed a business logic and 💖
// "Create an item with given name and barcode,
// but the barcode must be unique"
app.post("/items", (req) => {
  const barcode = req.body.barcode;

  const check = Item.find((i) => i.barcode === barcode);

  if (check) {
    throw "DUPLICATE_BARCODE";
  }

  return new Item(name, barcode);
});

app.listen(3000);
```

> :bulb: **This is pretty much it, thanks to [Nucleoid runtime](https://nucleoid.com/docs/runtime/), only with this :point_up_2:, you run your business logic and successfully persisted your object without external database.**

Learn more at [nucleoid.com/docs/get-started](https://nucleoid.com/docs/get-started)

<br/>

### Under the hood: Declarative Runtime Environment

Nucleoid is an implementation of symbolic AI for declarative (logic) programming at the runtime. As mentioned, the declarative runtime environment manages JavaScript state and stores each transaction in the built-in data store by declaratively rerendering JavaScript statements and building the knowledge graph (base) as well as an execution plan.

The declarative runtime isolates a behavior definition of a program from its technical instructions and executes declarative statements, which represent logical intention without carrying any technical detail. In this paradigm, there is no segregation regarding what data is or not, instead approaches how data (declarative statement) is related with others so that any type of data including business rules can be added without requiring any additional actions such as compiling, configuring, restarting as a result of plasticity. This approach also opens possibilities of storing data in the same box with the programming runtime.

In short, the main objective of the project is to manage both of data and logic under the same runtime. The declarative programming paradigm used by Nucleoid allows developers to focus on the business logic of the application, while the runtime manages the technical details.This allows for faster development and reduces the amount of code that needs to be written. Additionally, the sharding feature can help to distribute the load across multiple instances, which can further improve the performance of the system.

<br/>

## OpenAPI Integration with Nucleoid IDE

Nucleoid IDE is a web interface that helps to run very same npm package with OpenAPI.

[Go to Nucleoid IDE](https://nucleoid.com/ide/)

<p align="center">
  <img src="https://cdn.nucleoid.com/media/screenshot-1.png" alt="Nucleoid IDE Screenshot 1" width="650"/>
  <img src="https://cdn.nucleoid.com/media/screenshot-2.png" alt="Nucleoid IDE Screenshot 2" width="650"/>
</p>

<br/>

## Benchmark

This is the comparation our sample order app in Nucleoid IDE against MySQL and Postgres with using Express.js and Sequelize libraries.

https://nucleoid.com/ide/sample

<img src="https://cdn.nucleoid.com/media/benchmark.png" alt="Benchmark" width="550"/>

> Performance benchmark happened in t2.micro of AWS EC2 instance and both databases had dedicated server with <u>no indexes and default configurations</u>.

https://github.com/NucleoidJS/benchmark

This does not necessary mean Nucleoid runtime is faster than MySQL or Postgres, instead databases require constant maintenance by DBA teams with indexing, caching, purging etc. however, Nucleoid tries to solve this problem with managing logic and data internally. As seen in the chart, for applications with average complexity, Nucleoid's performance is close to linear because of on-chain data store as well as in-memory computing model.

<br/>

## Project Status :avocado:

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

## Contributors

<!-- NucBot -->

<table><tr><td align="center"><a href="https://github.com/NucBot"><img src="https://avatars.githubusercontent.com/u/110643717?v=4&s=100" width="100px;" alt="User NucBot"/><br/><sub>NucBot</sub></a></td><td align="center"><a href="https://github.com/canmingir"><img src="https://avatars.githubusercontent.com/u/54210920?v=4&s=100" width="100px;" alt="User canmingir"/><br/><sub>canmingir</sub></a></td><td align="center"><a href="https://github.com/322332"><img src="https://avatars.githubusercontent.com/u/16444899?v=4&s=100" width="100px;" alt="User 322332"/><br/><sub>322332</sub></a></td><td align="center"><a href="https://github.com/dependabot[bot]"><img src="https://avatars.githubusercontent.com/u/49699333?v=4&s=100" width="100px;" alt="User dependabot[bot]"/><br/><sub>dependabot[bot]</sub></a></td><td align="center"><a href="https://github.com/Gulshanaggarwal"><img src="https://avatars.githubusercontent.com/u/58553401?v=4&s=100" width="100px;" alt="User Gulshanaggarwal"/><br/><sub>Gulshanaggarwal</sub></a></td><td align="center"><a href="https://github.com/CanPacis"><img src="https://avatars.githubusercontent.com/u/37307107?v=4&s=100" width="100px;" alt="User CanPacis"/><br/><sub>CanPacis</sub></a></td><td align="center"><a href="https://github.com/EnesKeremAYDIN"><img src="https://avatars.githubusercontent.com/u/46195766?v=4&s=100" width="100px;" alt="User EnesKeremAYDIN"/><br/><sub>EnesKeremAYDIN</sub></a></td></tr></table>

<br/>

Generated by <a href="https://github.com/NucleoidJS/NucBot">NucBot</a>
