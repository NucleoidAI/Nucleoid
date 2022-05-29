const data = require("../libs/data");
data.clear();

const request = require("supertest");
const { deepEqual } = require("assert");
const nucleoid = require("../../index");
const { state } = require("../state");
const graph = require("../graph");
const _ = require("lodash");

const options = { terminal: false };

describe("Nucleoid API", () => {
  beforeEach(() => {
    for (let property in state) delete state[property];
    for (let property in graph) delete graph[property];

    state["classes"] = [];
    graph["classes"] = { name: "classes" };

    data.clear();
  });
  after(() => data.clear());

  it("Hello World", async () => {
    const app = nucleoid(options);
    class User {
      constructor(name) {
        this.name = name;
      }
    }
    nucleoid.register(User);

    app.get("/users", (req) =>
      User.filter((user) => user.name === req.query.name)
    );
    app.get("/users/:user", (req) => User[req.params.user]);
    app.post("/users", (req) => new User(req.body.name));

    const res1 = await request(app).post("/users").send({ name: "Daphne" });
    deepEqual(res1.body, { id: "user0", name: "Daphne" });

    const res2 = await request(app).get("/users?name=Daphne").send();
    deepEqual(res2.body, [{ id: "user0", name: "Daphne" }]);

    const res3 = await request(app).get("/users/user0").send();
    deepEqual(res3.body, { id: "user0", name: "Daphne" });
  });

  it("OpenAPI", async () => {
    const app = nucleoid(options);

    app.context(`${__dirname}/context.json`);
    app.openapi(`${__dirname}/openapi.json`);

    const res1 = await request(app)
      .post("/api/items")
      .send({ name: "ITEM-1", barcode: "BARCODE-1" });
    deepEqual(res1.body, {
      barcode: "BARCODE-1",
      id: "item0",
      name: "ITEM-1",
    });

    const res2 = await request(app).get("/api/items/item0").send();
    deepEqual(res2.body, {
      barcode: "BARCODE-1",
      id: "item0",
      name: "ITEM-1",
    });
  });

  it("Custom Scope", async () => {
    const app = nucleoid(options);
    const express = app.express();

    class Item {
      constructor(name) {
        this.name = name;
      }
    }
    nucleoid.register(Item);

    express.get("/", (req, res) => res.send({ message: "Hello" }));
    express.post("/items", (req, res) => {
      const scope = { name: req.body.name };
      const item = nucleoid.run((scope) => new Item(scope.name), scope);
      res.json(item);
    });
    express.get("/items", (req, res) => {
      const scope = { name: req.query.name };
      const list = nucleoid.run(
        (scope) => _.filter(Item, (item) => item.name === scope.name),
        scope
      );
      res.json(list);
    });

    const res1 = await request(app).get("/").send();
    deepEqual(res1.body, { message: "Hello" });

    const res2 = await request(app).post("/items").send({ name: "ITEM-1" });
    deepEqual(res2.body, {
      id: "item0",
      name: "ITEM-1",
    });

    const res3 = await request(app).get("/items?name=ITEM-1").send();
    deepEqual(res3.body, [
      {
        id: "item0",
        name: "ITEM-1",
      },
    ]);
  });
});
