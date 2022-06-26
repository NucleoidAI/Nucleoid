const test = require("../../libs/test");
const terminal = require("../../terminal");
const request = require("supertest");
const { equal, notEqual } = require("assert");

describe("Logs service", () => {
  beforeEach(() => test.clear());

  it("retrieves logs", async () => {
    const res1 = await request(terminal).get("/logs").send();
    const logs1 = res1.body;
    equal(res1.status, 200);
    equal(logs1.length, 0);

    await request(terminal).post("/").send("'Hello'");

    const res2 = await request(terminal).get("/logs").send();
    const logs2 = res2.body;

    equal(res2.status, 200);
    equal(logs2.length, 1);
    equal(logs2[0].s, "'Hello'");
    notEqual(logs2[0].t, undefined);
    equal(logs2[0].r, "Hello");
    notEqual(logs2[0].d, undefined);
  });
});
