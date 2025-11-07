import { equal } from "assert";
import { random } from "../random";

describe("Random lib", () => {
  it("returns random string with default length", () => {
    const str: string = random();
    equal(str.length, 16);
  });

  it("returns random string with given length", () => {
    const str: string = random(32);
    equal(str.length, 32);
  });
});
