const LET = require("./let");
const EXPRESSION = require("./expression");
const Id = require("./lib/identifier");
const BREAK = require("./break");
const state = require("./state");

class LET$INSTANCE extends LET {
  before() {
    this.value = new EXPRESSION(
      this.declaration.value.tokens.map((token) => {
        let parts = token.split(".");
        if (parts[0] === this.class.name)
          parts[0] = Id.serialize(this.instance, true);
        return parts.join(".");
      })
    );
  }

  run(scope) {
    try {
      let value = this.value.run(scope);
      let expression = "scope.local." + this.name + `=${value}`;
      state.run(scope, expression);
    } catch (error) {
      return { next: new BREAK(scope.block) };
    }
  }
}

module.exports = LET$INSTANCE;
