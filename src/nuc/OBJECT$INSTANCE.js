const Id = require("../lib/identifier");
const OBJECT = require("./OBJECT");
const graph = require("../graph");

class OBJECT$INSTANCE extends OBJECT {
  before() {
    let declaration = Id.serialize(this.declaration);

    let parts = declaration.split(".");
    parts[0] = Id.serialize(this.instance);

    let p = Id.splitLast(parts.join("."));
    this.object = graph[p[1]];

    this.key = Id.serialize(this);
    this.class = this.declaration.class;
  }
}

module.exports = OBJECT$INSTANCE;
