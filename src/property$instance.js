const PROPERTY = require("./property");
const EXPRESSION = require("./expression");
const Id = require("./utils/identifier");
const graph = require("./graph");

module.exports = class PROPERTY$INSTANCE extends PROPERTY {
  before(scope) {
    let declaration = Id.serialize(this.declaration);

    let parts = declaration.split(".");
    parts[0] = Id.serialize(this.instance);

    let p = Id.splitLast(parts.join("."));
    this.object = graph[p[1]];

    this.key = Id.serialize(this);
    this.value = new EXPRESSION(
      this.declaration.value.tokens.map((token) => {
        let parts = token.split(".");
        if (parts[0] === Id.root(this.declaration).name)
          parts[0] = Id.serialize(this.instance);
        return parts.join(".");
      })
    );
    this.value.before(scope, this.key);
  }
};
