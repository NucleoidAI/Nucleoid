var EXPRESSION = require("./expression");
var Instance = require("./instance");
var Identifier = require("./identifier");

module.exports = class EXPRESSION$INSTANCE extends EXPRESSION {
  run(scope) {
    let instance = Instance.retrieve(scope, this.class.name);

    if (instance !== undefined) {
      this.tokens = this.tokens.map((token) => {
        let parts = token.split(".");
        if (parts[0] === this.class.name)
          parts[0] = Identifier.serialize(instance);
        return parts.join(".");
      });
    }

    return super.run(scope);
  }
};
