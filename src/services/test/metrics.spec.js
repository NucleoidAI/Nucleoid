const test = require("../../libs/test");
const terminal = require("../../terminal");
const request = require("supertest");
const { equal, notEqual } = require("assert");

describe("Metrics service", () => {
  beforeEach(() => test.clear());

  it("retrieves memory details", async () => {
    const res = await request(terminal).get("/metrics").send();
    equal(res.status, 200);
    notEqual(res.body.free, undefined);
    notEqual(res.body.total, undefined);
  });
});
