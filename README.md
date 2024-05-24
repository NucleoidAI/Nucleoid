<h1 align="center">Nucleoid</h1>
<p align="center">
  Declarative (Logic) Runtime Environment
</p>

<p align="center">
  <a href="https://www.apache.org/licenses/LICENSE-2.0"><img src="https://img.shields.io/badge/Apache-2.0-yellow?style=for-the-badge&logo=apache" alt="License" /></a>
  <a href="https://www.npmjs.com/package/nucleoidjs"><img src="https://img.shields.io/badge/NPM-red?style=for-the-badge&logo=npm" alt="NPM" /></a>
  <a href="https://discord.gg/wN49SNssUw"><img src="https://img.shields.io/badge/Discord-lightgrey?style=for-the-badge&logo=discord" alt="Discord" /></a>
</p>

![Banner](.github/media/banner.png)

<p align="center">
  Reasoning Engine for Neuro-Symbolic AI with Declarative Logic
</p>

<br/>

D(L)RE is a type of Symbolic AI used for reasoning engine in Neuro-Symbolic AI. The runtime that tracks given statements in JavaScript syntax and creates relationships between variables, objects, and functions etc. in the graph. So, as writing just like any other codes in Node.js, the runtime translates your business logic to fully working application by managing the JS state as well as storing in the built-in data store, so that your application doesn't require external database or anything else.

[Nucleoid Chat](https://github.com/NucleoidAI/Nucleoid/assets/54210920/28282d4f-9e87-40c4-8589-b646e301bfa4)

### Neural Networks: The Learning Component

Neural networks in Neuro-Symbolic AI are adept at learning patterns, relationships, and features from large datasets. These networks excel in tasks that involve classification, prediction, and pattern recognition, making them invaluable for processing unstructured data, such as images, text, and audio. Neural networks, through their learning capabilities, can generalize from examples to understand complex data structures and nuances in the data.

### Symbolic AI: The Reasoning Component

The symbolic component of Neuro-Symbolic AI focuses on logic, rules, and symbolic representations of knowledge. Unlike neural networks that learn from data, symbolic AI uses predefined rules and knowledge bases to perform reasoning, make inferences, and understand relationships between entities. This aspect of AI is transparent, interpretable, and capable of explaining its decisions and reasoning processes in a way that humans can understand.

<br/>

<p align="center">
  <img src=".github/media/neuro-symbolic.png" width="225" alt="Neuro-Symbolic Diagram"/>
</p>

#### Declarative Logic in Symbolic Reasoning

Declarative logic is a subset of declarative programming, a style of building programs that expresses the logic of a computation without describing its control flow. In declarative logic, you state the facts and rules that define the problem domain. The runtime environment or the system itself figures out how to satisfy those conditions or how to apply those rules to reach a conclusion. This contrasts with imperative programming, where the developer writes code that describes the exact steps to achieve a goal.

Symbolic reasoning refers to the process of using symbols to represent problems and applying logical rules to manipulate these symbols and derive conclusions or solutions. In AI and computer science, it involves using symbolic representations for entities and actions, enabling the system to perform logical inferences, decision making, and problem-solving based on the rules and knowledge encoded in the symbols.

By integrating Nucleoid into Neuro-Symbolic AI, the system benefits from enhanced interpretability and reliability. The declarative logic and rules defined in Nucleoid provide clear explanations for the AI's decisions, making it easier for users to understand and trust the system's outputs. Furthermore, the explicit reasoning capabilities help ensure that decisions are made based on logical principles, adding a layer of reliability and consistency to the AI's behavior.

<p align="center">
  <img src=".github/media/graph.gif" width="450" alt="Graph Animation"/>
</p>

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
  const name = req.body.name;
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

<p align="center">
  <img src="https://cdn.nucleoid.com/media/taxonomy.png" width="450" alt="Nucleoid's Taxonomy"/>
</p>

The declarative runtime isolates a behavior definition of a program from its technical instructions and executes declarative statements, which represent logical intention without carrying any technical detail. In this paradigm, there is no segregation regarding what data is or not, instead approaches how data (declarative statement) is related with others so that any type of data including business rules can be added without requiring any additional actions such as compiling, configuring, restarting as a result of plasticity. This approach also opens possibilities of storing data in the same box with the programming runtime.

<div align="center">
  <table>
    <tr>
      <th>
        <img src="https://cdn.nucleoid.com/media/diagram1.png" width="225" alt="Logical Diagram 1"/>
      </th>
      <th>
        <img src="https://cdn.nucleoid.com/media/diagram2.png" width="275" alt="Logical Diagram 2"/>
      </th>
    </tr>
  </table>
</div>

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

> Performance benchmark happened in t2.micro of AWS EC2 instance and both databases had dedicated servers with <u>no indexes and default configurations</u>.

https://github.com/NucleoidJS/benchmark

This does not necessary mean Nucleoid runtime is faster than MySQL or Postgres, instead databases require constant maintenance by DBA teams with indexing, caching, purging etc. however, Nucleoid tries to solve this problem with managing logic and data internally. As seen in the chart, for applications with average complexity, Nucleoid's performance is close to linear because of on-chain data store, in-memory computing model as well as limiting the IO process.

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

<table><tr><td align="center"><a href="https://github.com/NucBot"><img src="https://avatars.githubusercontent.com/u/110643717?v=4&s=100" width="100px;" alt="User NucBot"/><br/><sub>NucBot</sub></a></td><td align="center"><a href="https://github.com/canmingir"><img src="https://avatars.githubusercontent.com/u/54210920?v=4&s=100" width="100px;" alt="User canmingir"/><br/><sub>canmingir</sub></a></td><td align="center"><a href="https://github.com/322332"><img src="https://avatars.githubusercontent.com/u/16444899?v=4&s=100" width="100px;" alt="User 322332"/><br/><sub>322332</sub></a></td><td align="center"><a href="https://github.com/dependabot[bot]"><img src="https://avatars.githubusercontent.com/u/49699333?v=4&s=100" width="100px;" alt="User dependabot[bot]"/><br/><sub>dependabot[bot]</sub></a></td><td align="center"><a href="https://github.com/francisco-giancarelli-crombie"><img src="https://avatars.githubusercontent.com/u/104434958?v=4&s=100" width="100px;" alt="User francisco-giancarelli-crombie"/><br/><sub>francisco-giancarelli-crombie</sub></a></td><td align="center"><a href="https://github.com/Gulshanaggarwal"><img src="https://avatars.githubusercontent.com/u/58553401?v=4&s=100" width="100px;" alt="User Gulshanaggarwal"/><br/><sub>Gulshanaggarwal</sub></a></td><td align="center"><a href="https://github.com/CanPacis"><img src="https://avatars.githubusercontent.com/u/37307107?v=4&s=100" width="100px;" alt="User CanPacis"/><br/><sub>CanPacis</sub></a></td></tr><tr><td align="center"><a href="https://github.com/durulkoca"><img src="https://avatars.githubusercontent.com/u/134300732?v=4&s=100" width="100px;" alt="User durulkoca"/><br/><sub>durulkoca</sub></a></td><td align="center"><a href="https://github.com/halilcengel"><img src="https://avatars.githubusercontent.com/u/49736917?v=4&s=100" width="100px;" alt="User halilcengel"/><br/><sub>halilcengel</sub></a></td><td align="center"><a href="https://github.com/EnesKeremAYDIN"><img src="https://avatars.githubusercontent.com/u/46195766?v=4&s=100" width="100px;" alt="User EnesKeremAYDIN"/><br/><sub>EnesKeremAYDIN</sub></a></td><td align="center"><a href="https://github.com/russle-smith"><img src="https://avatars.githubusercontent.com/u/109499168?v=4&s=100" width="100px;" alt="User russle-smith"/><br/><sub>russle-smith</sub></a></td><td align="center"><a href="https://github.com/russellgray"><img src="https://avatars.githubusercontent.com/u/143818261?v=4&s=100" width="100px;" alt="User russellgray"/><br/><sub>russellgray</sub></a></td></tr></table>

<br/>

Generated by <a href="https://github.com/NucleoidJS/NucBot">NucBot</a>
