const test = require("../../libs/test");
const terminal = require("../../terminal");
const request = require("supertest");
const { equal, deepEqual } = require("assert");
const data = require("./data.json");
const openapi = require("../../libs/openapi");

describe("OpenAPI service", () => {
  beforeEach(() => test.clear());

  it("starts app", async () => {
    const res1 = await request(terminal)
      .post("/openapi")
      .send({ ...data, action: "start" });
    equal(res1.status, 200);

    const res2 = await request(openapi.app())
      .post("/api/items")
      .send({ name: "ITEM-1", barcode: "BARCODE-1" });
    equal(res2.status, 200);
    deepEqual(res2.body, {
      barcode: "BARCODE-1",
      id: "item0",
      name: "ITEM-1",
    });

    const res3 = await request(openapi.app()).get("/api/items/item0").send();
    equal(res3.status, 200);
    deepEqual(res3.body, {
      barcode: "BARCODE-1",
      id: "item0",
      name: "ITEM-1",
    });

    const res4 = await request(terminal)
      .post("/openapi")
      .send({ action: "stop" });
    equal(res4.status, 200);
  });
});
