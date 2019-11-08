var state = require("./state"); // eslint-disable-line no-unused-vars
var Identifier = require("./identifier");
var OBJECT = require("./object");
var graph = require("./graph");

module.exports = class OBJECT$PROTOTYPE extends OBJECT {
  prepare() {
    let declaration = Identifier.serialize(this.declaration);
    let parts = declaration.split(".");
    parts[0] = Identifier.serialize(this.template);
    let p = Identifier.splitLast(parts.join("."));
    this.object = graph[p[1]];
    this.key = Identifier.serialize(this);
    this.class = this.declaration.class;
  }
};
