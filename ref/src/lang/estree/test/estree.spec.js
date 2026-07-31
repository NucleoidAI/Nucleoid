const ESTree = require("../estree");
const { generate } = require("../generator");
const { equal } = require("assert");
describe("ESTree", () => {
  it("appends single expression to single expression", () => {
    const source = {
      type: "Identifier",
      name: "a",
    };

    const target = {
      type: "Identifier",
      name: "b",
    };

    equal(generate(ESTree.append(source, target)), "a.b");
  });

  it("appends single expression to member expression", () => {
    const source = {
      type: "Identifier",
      name: "a",
    };

    const target = {
      type: "MemberExpression",
      computed: false,
      object: {
        type: "Identifier",
        name: "b",
      },
      property: {
        type: "Identifier",
        name: "c",
      },
    };

    equal(generate(ESTree.append(source, target)), "a.b.c");
  });

  it("appends single expression to multiple member expression", () => {
    const source = {
      type: "Identifier",
      name: "a",
    };

    const target = {
      type: "MemberExpression",
      computed: false,
      object: {
        type: "MemberExpression",
        computed: false,
        object: {
          type: "Identifier",
          name: "b",
        },
        property: {
          type: "Identifier",
          name: "c",
        },
      },
      property: {
        type: "Identifier",
        name: "d",
      },
    };

    equal(generate(ESTree.append(source, target)), "a.b.c.d");
  });

  it("appends member expression to single expression", () => {
    const source = {
      type: "MemberExpression",
      computed: false,
      object: {
        type: "Identifier",
        name: "a",
      },
      property: {
        type: "Identifier",
        name: "b",
      },
    };

    const target = {
      type: "Identifier",
      name: "c",
    };

    equal(generate(ESTree.append(source, target)), "a.b.c");
  });

  it("appends multiple member expression to single expression", () => {
    const source = {
      type: "MemberExpression",
      computed: false,
      object: {
        type: "MemberExpression",
        computed: false,
        object: {
          type: "Identifier",
          name: "a",
        },
        property: {
          type: "Identifier",
          name: "b",
        },
      },
      property: {
        type: "Identifier",
        name: "c",
      },
    };

    const target = {
      type: "Identifier",
      name: "d",
    };

    equal(generate(ESTree.append(source, target)), "a.b.c.d");
  });

  it("appends multiple member expression to multiple member expression", () => {
    const source = {
      type: "MemberExpression",
      computed: false,
      object: {
        type: "MemberExpression",
        computed: false,
        object: {
          type: "Identifier",
          name: "a",
        },
        property: {
          type: "Identifier",
          name: "b",
        },
      },
      property: {
        type: "Identifier",
        name: "c",
      },
    };

    const target = {
      type: "MemberExpression",
      computed: false,
      object: {
        type: "MemberExpression",
        computed: false,
        object: {
          type: "Identifier",
          name: "d",
        },
        property: {
          type: "Identifier",
          name: "e",
        },
      },
      property: {
        type: "Identifier",
        name: "f",
      },
    };

    equal(generate(ESTree.append(source, target)), "a.b.c.d.e.f");
  });
});
