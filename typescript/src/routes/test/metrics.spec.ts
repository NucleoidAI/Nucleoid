import { equal, notEqual } from "assert";

import nucleoid from "../../../index";
import request from "supertest";
import terminal from "../terminal";
import test from "../../lib/test";

describe("Metrics service", () => {
  beforeAll(() => nucleoid.start({ test: true }));
  beforeEach(() => test.clear());

  it("retrieves memory details", async () => {
    const res = await request(terminal).get("/metrics").send();
    equal(res.status, 200);
    notEqual(res.body.free, undefined);
    notEqual(res.body.total, undefined);
  });
});
