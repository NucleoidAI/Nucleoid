var state = require("./state"); // eslint-disable-line no-unused-vars
var PROPERTY = require("./property");
var EXPRESSION = require("./expression");
var Identifier = require("./identifier");

class PROPERTY$PROTOTYPE extends PROPERTY {
  prepare() {
    this.key = Identifier.serialize(this);

    this.value = new EXPRESSION(
      this.declaration.value.tokens.map(token => {
        let parts = token.split(".");
        if (parts[0] == Identifier.root(this.declaration).name)
          parts[0] = Identifier.serialize(this.template);
        return parts.join(".");
      })
    );
  }
}

PROPERTY$PROTOTYPE.prototype.type = "PROTOTYPE";
module.exports = PROPERTY$PROTOTYPE;
