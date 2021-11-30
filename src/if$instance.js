const IF = require("./if");
const EXPRESSION = require("./expression");
const Id = require("./utils/identifier");

module.exports = class IF$INSTANCE extends IF {
  before() {
    this.condition = new EXPRESSION(
      this.declaration.condition.tokens.map((token) => {
        let parts = token.split(".");
        if (parts[0] === this.class.name)
          parts[0] = Id.serialize(this.instance);
        return parts.join(".");
      })
    );
  }

  run(scope) {
    scope.instances[this.class.name] = this.instance;
    return super.run(scope);
  }
};
