const test = require("../../lib/test");
const terminal = require("../../terminal");
const request = require("supertest");
const { equal, deepEqual } = require("assert");
const data = require("./data.json");
const openapi = require("../../lib/openapi");

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
    equal(res2.body.name, "ITEM-1");
    equal(res2.body.barcode, "BARCODE-1");

    const itemId = res2.body.id;

    const res3 = await request(openapi.app())
      .get(`/api/items/${itemId}`)
      .send();
    equal(res3.status, 200);
    equal(res3.body.name, "ITEM-1");
    equal(res3.body.barcode, "BARCODE-1");

    const res4 = await request(openapi.app())
      .post("/api/items")
      .send({ barcode: "BARCODE-1" });
    equal(res4.status, 400);
    deepEqual(res4.body, { error: "INVALID_NAME" });

    const res5 = await request(openapi.app())
      .get("/api/items/invalid_item")
      .send();
    equal(res5.status, 404);
    equal(res5.text, "");

    const res6 = await request(terminal)
      .post("/openapi")
      .send({ action: "stop" });
    equal(res6.status, 200);
  });
});
