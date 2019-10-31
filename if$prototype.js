var IF = require("./if");
var EXPRESSION = require("./expression");
var Identifier = require("./identifier");

class IF$PROTOTYPE extends IF {
  prepare() {
    this.condition = new EXPRESSION(
      this.declaration.condition.tokens.map(token => {
        let parts = token.split(".");
        if (parts[0] == this.class.name)
          parts[0] = Identifier.serialize(this.instance);
        return parts.join(".");
      })
    );
  }

  run(scope) {
    scope.instance[this.class.name] = this.instance;
    return super.run(scope);
  }
}

IF$PROTOTYPE.prototype.type = "PROTOTYPE";
module.exports = IF$PROTOTYPE;
