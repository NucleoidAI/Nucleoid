import * as ESTree from "../estree";

import { strictEqual as equal } from "assert";
import { generate } from "../generator";

describe("ESTree.append", () => {
  it("appends single expression to single expression", () => {
    const source = { type: "Identifier" as const, name: "a" };
    const target = { type: "Identifier" as const, name: "b" };

    equal(generate(ESTree.append(source, target)), "a.b");
  });

  it("appends single expression to member expression", () => {
    const source = { type: "Identifier" as const, name: "a" };
    const target = {
      type: "MemberExpression" as const,
      computed: false,
      object: { type: "Identifier" as const, name: "b" },
      property: { type: "Identifier" as const, name: "c" },
    };

    equal(generate(ESTree.append(source, target)), "a.b.c");
  });

  it("appends single expression to nested member expression", () => {
    const source = { type: "Identifier" as const, name: "a" };
    const target = {
      type: "MemberExpression" as const,
      computed: false,
      object: {
        type: "MemberExpression" as const,
        computed: false,
        object: { type: "Identifier" as const, name: "b" },
        property: { type: "Identifier" as const, name: "c" },
      },
      property: { type: "Identifier" as const, name: "d" },
    };

    equal(generate(ESTree.append(source, target)), "a.b.c.d");
  });

  it("appends member expression to single expression", () => {
    const source = {
      type: "MemberExpression" as const,
      computed: false,
      object: { type: "Identifier" as const, name: "a" },
      property: { type: "Identifier" as const, name: "b" },
    };
    const target = { type: "Identifier" as const, name: "c" };

    equal(generate(ESTree.append(source, target)), "a.b.c");
  });

  it("appends nested member expression to single expression", () => {
    const source = {
      type: "MemberExpression" as const,
      computed: false,
      object: {
        type: "MemberExpression" as const,
        computed: false,
        object: { type: "Identifier" as const, name: "a" },
        property: { type: "Identifier" as const, name: "b" },
      },
      property: { type: "Identifier" as const, name: "c" },
    };
    const target = { type: "Identifier" as const, name: "d" };

    equal(generate(ESTree.append(source, target)), "a.b.c.d");
  });

  it("appends nested member expression to nested member expression", () => {
    const source = {
      type: "MemberExpression" as const,
      computed: false,
      object: {
        type: "MemberExpression" as const,
        computed: false,
        object: { type: "Identifier" as const, name: "a" },
        property: { type: "Identifier" as const, name: "b" },
      },
      property: { type: "Identifier" as const, name: "c" },
    };
    const target = {
      type: "MemberExpression" as const,
      computed: false,
      object: {
        type: "MemberExpression" as const,
        computed: false,
        object: { type: "Identifier" as const, name: "d" },
        property: { type: "Identifier" as const, name: "e" },
      },
      property: { type: "Identifier" as const, name: "f" },
    };

    equal(generate(ESTree.append(source, target)), "a.b.c.d.e.f");
  });
});
