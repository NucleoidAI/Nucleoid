var state = require("./state"); // eslint-disable-line no-unused-vars
var PROPERTY = require("./property");
var EXPRESSION = require("./expression");
var Identifier = require("./identifier");

class PROPERTY$INSTANCE extends PROPERTY {
  prepare() {
    this.key = Identifier.serialize(this);

    this.value = new EXPRESSION(
      this.declaration.value.tokens.map(token => {
        let parts = token.split(".");
        if (parts[0] == this.class.name)
          parts[0] = Identifier.serialize(this.instance);
        return parts.join(".");
      })
    );
  }
}

PROPERTY$INSTANCE.prototype.type = "INSTANCE";
module.exports = PROPERTY$INSTANCE;
