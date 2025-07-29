import * as openapi from "../lib/openapi";

import { deepStrictEqual as deepEqual, strictEqual as equal } from "assert";

import _ from "lodash";
import nucleoid from "../../";
import request from "supertest";
import test from "../lib/test";
import { v4 as uuid } from "uuid";

const options = { id: uuid(), test: true };

describe.skip("Nucleoid API", () => {
  before(() => nucleoid.start({ test: true }));
  beforeEach(() => test.clear());

  it("Hello World", async () => {
    const app = nucleoid(options);
    class User {
      name: string;
      constructor(name: string) {
        this.name = name;
      }
    }
    nucleoid.register(User);

    app.get("/users", (req: any) =>
      User.filter((user: any) => user.name === req.query.name)
    );
    app.get("/users/:user", (req: any) => User[req.params.user]);
    app.post("/users", (req: any) => new User(req.body.name));

    const res1 = await request(app).post("/users").send({ name: "Daphne" });
    const userId = res1.body.id;
    deepEqual(res1.body, { id: userId, name: "Daphne" });

    const res2 = await request(app).get("/users?name=Daphne").send();
    deepEqual(res2.body, [{ id: userId, name: "Daphne" }]);

    const res3 = await request(app).get(`/users/${userId}`).send();
    deepEqual(res3.body, { id: userId, name: "Daphne" });
  });

  it.skip("OpenAPI", async () => {
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
      name: string;
      constructor(name: string) {
        this.name = name;
      }
    }
    nucleoid.register(Item);

    express.get("/", (req: any, res: any) => res.send({ message: "Hello" }));
    express.post("/items", (req: any, res: any) => {
      const scope = { name: req.body.name };
      const item = nucleoid.run(
        (scopeParam: any) => new Item(scopeParam.name),
        scope
      );
      res.json(item);
    });
    express.get("/items", (req: any, res: any) => {
      const scope = { name: req.query.name };
      const list = nucleoid.run(
        (scopeParam: any) =>
          _.filter(Item, (item: Item) => item.name === scopeParam.name),
        scope
      );
      res.json(list);
    });

    const res1 = await request(app).get("/").send();
    deepEqual(res1.body, { message: "Hello" });

    const res2 = await request(app).post("/items").send({ name: "ITEM-1" });
    const itemId = res2.body.id;
    deepEqual(res2.body, { id: itemId, name: "ITEM-1" });

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
      name: string;
      constructor(name: string) {
        this.name = name;
      }
    }
    nucleoid.register(User);

    app.get("/users/:user", (req: any) => User[req.params.user]);
    app.post("/users", (req: any) => {
      const name = req.body.name;
      if (!name) {
        throw "INVALID_NAME";
      }
      return new User(req.body.name);
    });

    const res1 = await request(app).post("/users").send();
    equal(res1.status, 400);
    equal(res1.body, "INVALID_NAME");

    const res2 = await request(app).get("/users/invalid_user").send();
    equal(res2.status, 404);
    deepEqual(res2.text, "");
  });
});

