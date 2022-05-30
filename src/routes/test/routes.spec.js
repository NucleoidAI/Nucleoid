const terminal = require("../../terminal");
const request = require("supertest");
const { equal, notEqual, deepEqual } = require("assert");
const test = require("../../libs/test");
const data = require("./data.json");
const openapi = require("../../libs/openapi");

describe("Nucleoid Routes", () => {
  beforeEach(() => test.clear());

  it("OpenAPI", async () => {
    await request(terminal)
      .post("/openapi")
      .send({ ...data, action: "start" });

    const res1 = await request(openapi.app())
      .post("/api/items")
      .send({ name: "ITEM-1", barcode: "BARCODE-1" });

    deepEqual(res1.body, {
      barcode: "BARCODE-1",
      id: "item0",
      name: "ITEM-1",
    });

    const res2 = await request(openapi.app()).get("/api/items/item0").send();
    deepEqual(res2.body, {
      barcode: "BARCODE-1",
      id: "item0",
      name: "ITEM-1",
    });

    openapi.stop();
  });

  it("Logs", async () => {
    await request(terminal).post("/").send("'Hello'");
    const res = await request(terminal).get("/logs").send();
    const logs = res.body;

    equal(logs.length, 1);
    equal(logs[0].s, "'Hello'");
    notEqual(logs[0].t, undefined);
    equal(logs[0].r, "Hello");
    notEqual(logs[0].d, undefined);
  });

  it("Metrics", async () => {
    const res = await request(terminal).get("/metrics").send();
    notEqual(res.body.free, undefined);
    notEqual(res.body.total, undefined);
  });
});
