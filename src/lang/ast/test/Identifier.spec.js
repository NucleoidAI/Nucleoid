const Identifier = require("../Identifier");
const { equal } = require("assert");

describe("Identifier", () => {
  it("serializes single expression", () => {
    const identifier = new Identifier("a");

    equal(identifier.toString(), "a");
    equal(identifier.first.toString(), "a");
    equal(identifier.last.toString(), "a");
    equal(identifier.object.toString(), "a");
  });

  it("serializes member expression", () => {
    const identifier = new Identifier("a.b");

    equal(identifier.toString(), "a.b");
    equal(identifier.first.toString(), "a");
    equal(identifier.last.toString(), "b");
    equal(identifier.object.toString(), "a");
  });

  it("serializes multiple member expression", () => {
    const identifier = new Identifier(
      "a.b.c.d.e.f.g.h.i.j.k.l.m.n.o.p.q.r.s.t.u.v.w.x.y.z"
    );

    equal(
      identifier.toString(),
      "a.b.c.d.e.f.g.h.i.j.k.l.m.n.o.p.q.r.s.t.u.v.w.x.y.z"
    );
    equal(identifier.first.toString(), "a");
    equal(identifier.last.toString(), "z");
    equal(
      identifier.object.toString(),
      "a.b.c.d.e.f.g.h.i.j.k.l.m.n.o.p.q.r.s.t.u.v.w.x.y"
    );
  });
});
