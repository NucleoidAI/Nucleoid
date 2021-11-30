const Id = require("./utils/identifier");
const OBJECT = require("./object");
const graph = require("./graph");

module.exports = class OBJECT$INSTANCE extends OBJECT {
  before() {
    let declaration = Id.serialize(this.declaration);

    let parts = declaration.split(".");
    parts[0] = Id.serialize(this.instance);

    let p = Id.splitLast(parts.join("."));
    this.object = graph[p[1]];

    this.key = Id.serialize(this);
    this.class = this.declaration.class;
  }
};
