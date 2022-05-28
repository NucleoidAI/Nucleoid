const data = require("../libs/data");
data.clear();

const request = require("supertest");
const { deepEqual } = require("assert");
const nucleoid = require("../../index");
const { state } = require("../state");
const graph = require("../graph");

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

    app.get("/users", () => User);
    app.post("/users", (req) => new User(req.body.name));

    const res = await request(app).post("/users").send({ name: "Daphne" });
    deepEqual(res.body, { id: "user0", name: "Daphne" });

    return request(app)
      .get("/users")
      .expect(200, [{ id: "user0", name: "Daphne" }]);
  });
});
