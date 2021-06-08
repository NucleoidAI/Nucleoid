const PROPERTY = require("./property");
const EXPRESSION = require("./expression");
const Identifier = require("./identifier");
const graph = require("./graph");

module.exports = class PROPERTY$INSTANCE extends PROPERTY {
  before(scope) {
    let declaration = Identifier.serialize(this.declaration);

    let parts = declaration.split(".");
    parts[0] = Identifier.serialize(this.instance);

    let p = Identifier.splitLast(parts.join("."));
    this.object = graph[p[1]];

    this.key = Identifier.serialize(this);
    this.value = new EXPRESSION(
      this.declaration.value.tokens.map((token) => {
        let parts = token.split(".");
        if (parts[0] === Identifier.root(this.declaration).name)
          parts[0] = Identifier.serialize(this.instance);
        return parts.join(".");
      })
    );
    this.value.before(scope, this.key);
  }
};
