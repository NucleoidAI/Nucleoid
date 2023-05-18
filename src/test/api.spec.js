const test = require("../lib/test");
const request = require("supertest");
const { equal, deepEqual } = require("assert");
const nucleoid = require("../../");
const openapi = require("../lib/openapi");
const _ = require("lodash");
const { v4: uuid } = require("uuid");

const options = { id: uuid(), test: true };

describe("Nucleoid API", () => {
  beforeEach(() => test.clear());
  after(() => test.clear());

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
    const userId = res1.body.id;
    deepEqual(res1.body, { id: userId, name: "Daphne" });

    const res2 = await request(app).get("/users?name=Daphne").send();
    deepEqual(res2.body, [{ id: userId, name: "Daphne" }]);

    const res3 = await request(app).get(`/users/${userId}`).send();
    deepEqual(res3.body, { id: userId, name: "Daphne" });
  });

  it("OpenAPI", async () => {
    const app = nucleoid(options);

    app.context(`${__dirname}/context.json`);
    app.openapi(`${__dirname}/openapi.json`);

    const res1 = await request(app)
      .post("/api/items")
      .send({ name: "ITEM-1", barcode: "BARCODE-1" });
    const itemId = res1.body.id;
    deepEqual(res1.body, {
      id: itemId,
      barcode: "BARCODE-1",
      name: "ITEM-1",
    });

    const res2 = await request(app).get(`/api/items/${itemId}`).send();
    deepEqual(res2.body, {
      id: itemId,
      barcode: "BARCODE-1",
      name: "ITEM-1",
    });

    openapi.stop();
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
    const itemId = res2.body.id;
    deepEqual(res2.body, {
      id: itemId,
      name: "ITEM-1",
    });

    const res3 = await request(app).get("/items?name=ITEM-1").send();
    deepEqual(res3.body, [
      {
        id: itemId,
        name: "ITEM-1",
      },
    ]);
  });

  it("Exception Handling", async () => {
    const app = nucleoid(options);
    class User {
      constructor(name) {
        this.name = name;
      }
    }
    nucleoid.register(User);

    app.get("/users/:user", (req) => User[req.params.user]);
    app.post("/users", (req) => {
      const name = req.body.name;
      if (!name) {
        throw "INVALID_NAME";
      }
      new User(req.body.name);
    });

    const res1 = await request(app).post("/users").send();
    equal(res1.status, 400);
    equal(res1.body, "INVALID_NAME");

    const res2 = await request(app).get("/users/invalid_user").send();
    equal(res2.status, 404);
    deepEqual(res2.text, "");
  });
});
