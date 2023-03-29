const EXPRESSION = require("./expression");
const Instance = require("./lib/instance");
const Id = require("./lib/identifier");

class EXPRESSION$INSTANCE extends EXPRESSION {
  run(scope) {
    let instance = Instance.retrieve(scope, this.class.name);

    if (instance !== undefined) {
      this.tokens = this.tokens.map((token) => {
        let parts = token.split(".");
        if (parts[0] === this.class.name) parts[0] = Id.serialize(instance);
        return parts.join(".");
      });
    }

    return super.run(scope);
  }
}

module.exports = EXPRESSION$INSTANCE;
