const Identifier = require("../Identifier");
const { equal } = require("assert");
const Scope = require("../../../Scope");

describe("Identifier", () => {
  it("serializes single expression", () => {
    const identifier = new Identifier("a");

    equal(identifier, "a");
    equal(identifier.first, "a");
    equal(identifier.last, "a");
    equal(identifier.object, null);
  });

  it("serializes member expression", () => {
    const identifier = new Identifier("a.b");

    equal(identifier, "a.b");
    equal(identifier.first, "a");
    equal(identifier.last, "b");
    equal(identifier.object, "a");
  });

  it("serializes multiple member expression", () => {
    const identifier = new Identifier(
      "a.b.c.d.e.f.g.h.i.j.k.l.m.n.o.p.q.r.s.t.u.v.w.x.y.z"
    );

    equal(identifier, "a.b.c.d.e.f.g.h.i.j.k.l.m.n.o.p.q.r.s.t.u.v.w.x.y.z");
    equal(identifier.first, "a");
    equal(identifier.last, "z");
    equal(
      identifier.object,
      "a.b.c.d.e.f.g.h.i.j.k.l.m.n.o.p.q.r.s.t.u.v.w.x.y"
    );
  });

  it("serializes this expression", () => {
    const identifier = new Identifier("this.b.c");

    equal(identifier, "this.b.c");
    equal(identifier.first, "this");
    equal(identifier.last, "c");
    equal(identifier.object, "this.b");
  });

  it("serializes single expression with regexp", () => {
    const identifier = new Identifier("/[A-Z]/");

    equal(identifier, "/[A-Z]/");
    equal(identifier.first, "/[A-Z]/");
    equal(identifier.last, "/[A-Z]/");
    equal(identifier.object, null);
  });

  it("serializes member expression with regexp", () => {
    const identifier = new Identifier("/[A-Z]/.test");

    equal(identifier, "/[A-Z]/.test");
    equal(identifier.first, "/[A-Z]/");
    equal(identifier.last, "test");
    equal(identifier.object, "/[A-Z]/");
  });

  it("serializes single expression with bracket", () => {
    const scope = new Scope();
    scope.graph.test = {};
    const identifier = new Identifier("User[test]");

    equal(identifier, "User[test]");
    equal(identifier.first, "User");
    equal(identifier.last, "test");
    equal(identifier.object, "User");
    equal(identifier.generate(scope), "User[scope.local.test]");
  });

  it("serializes member expression with bracket", () => {
    const scope = new Scope();
    scope.graph.test = {};
    const identifier = new Identifier("User.address[test]");

    equal(identifier, "User.address[test]");
    equal(identifier.first, "User");
    equal(identifier.last, "test");
    equal(identifier.object, "User.address");
    equal(identifier.generate(scope), "User.address[scope.local.test]");
  });
});
