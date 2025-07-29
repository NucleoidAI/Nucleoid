import { equal, notEqual } from "assert";

import request from "supertest";
import terminal from "../terminal";
import test from "../../lib/test";

describe("Terminal", () => {
  beforeEach(() => test.clear());

  it("runs command", async () => {
    const res1 = await request(terminal)
      .post("/")
      .set("Content-Type", "application/javascript")
      .send("'Hello'");
    equal(res1.status, 200);
    equal(res1.body.result, "Hello");
    notEqual(res1.body.error, true);

    const res2 = await request(terminal)
      .post("/")
      .set("Content-Type", "application/javascript")
      .send("a");
    equal(res2.status, 200);
    equal(res2.body.result, "ReferenceError: a is not defined");
    equal(res2.body.error, true);
  });
});
