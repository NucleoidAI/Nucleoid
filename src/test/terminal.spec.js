const test = require("../libs/test");
const terminal = require("../terminal");
const request = require("supertest");
const { equal, notEqual } = require("assert");

describe("Terminal", () => {
  beforeEach(() => test.clear());

  it("runs command", async () => {
    const res1 = await request(terminal).post("/").send("'Hello'");
    equal(res1.status, 200);
    equal(res1.body.result, "Hello");
    notEqual(res1.body.error, true);

    const res2 = await request(terminal).post("/").send("a");
    equal(res2.status, 200);
    equal(res2.body.result, "ReferenceError: a is not defined");
    equal(res2.body.error, true);
  });
});
