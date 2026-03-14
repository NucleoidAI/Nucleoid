const serialize = require("../serialize");
const { equal } = require("assert");

describe("Serialize lib", () => {
  it("serializes an Array", () => {
    const input = [1, 2, 3];
    const output = serialize(input);
    equal(output, "[1,2,3]");
  });

  it("serializes a Map", () => {
    const input = new Map();
    input.set("key", "value");
    const output = serialize(input);
    equal(output, 'new Map([["key","value"]])');
  });

  it("serializes a Set", () => {
    const input = new Set([1, 2, 3]);
    const output = serialize(input);
    equal(output, "new Set([1,2,3])");
  });

  it("serializes a Date", () => {
    const input = new Date("2023-05-17T00:00:00Z");
    const output = serialize(input);
    equal(output, `new Date(${input.getTime()})`);
  });

  it("serializes a String", () => {
    const input = String("hello");
    const output = serialize(input);
    equal(output, '"hello"');
  });

  it("serializes a Number", () => {
    const input = Number(123);
    const output = serialize(input);
    equal(output, "123");
  });

  it("serializes a Boolean", () => {
    const input = Boolean(true);
    const output = serialize(input);
    equal(output, "true");
  });

  it("serializes a Function", () => {
    const input = function () {
      return "hello";
    };
    const output = serialize(input);
    equal(output, 'function () { return "hello"; }');
  });

  it("serializes a RegExp", () => {
    const input = /hello/g;
    const output = serialize(input);
    equal(output, "/hello/g");
  });

  it("serializes an Object", () => {
    const input = { key: "value" };
    const output = serialize(input);
    equal(output, '{key:"value"}');
  });

  it("serializes an Object with reference", () => {
    const input = { id: "id1", prop: {} };
    input.prop.test = input;

    const output = serialize(input, "source");
    equal(output, "{id:\"id1\",prop:{test:{$ref:{id:'id1',source:'source'}}}}");
  });
});
