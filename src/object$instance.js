const Identifier = require("./identifier");
const OBJECT = require("./object");
const graph = require("./graph");

module.exports = class OBJECT$INSTANCE extends OBJECT {
  before() {
    let declaration = Identifier.serialize(this.declaration);

    let parts = declaration.split(".");
    parts[0] = Identifier.serialize(this.instance);

    let p = Identifier.splitLast(parts.join("."));
    this.object = graph[p[1]];

    this.key = Identifier.serialize(this);
    this.class = this.declaration.class;
  }
};
