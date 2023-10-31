const PROPERTY = require("./PROPERTY");
const EXPRESSION = require("./EXPRESSION");
const Id = require("../lib/identifier");
const graph = require("../graph");

class PROPERTY$INSTANCE extends PROPERTY {
  before(scope) {
    let declaration = Id.serialize(this.declaration);

    let parts = declaration.split(".");
    parts[0] = Id.serialize(this.instance);

    let p = Id.splitLast(parts.join("."));
    this.object = graph[p[1]];

    this.key = Id.serialize(this);
    this.value = new EXPRESSION(
      this.declaration.value.node.map((node) => {
        let parts = node.split(".");
        if (parts[0] === Id.root(this.declaration).name)
          parts[0] = Id.serialize(this.instance);
        return parts.join(".");
      })
    );
    this.value.before(scope, this.key);
  }
}

module.exports = PROPERTY$INSTANCE;
