const LET = require("./LET");
const EXPRESSION = require("./EXPRESSION");
const Id = require("../lib/identifier");
const BREAK = require("./BREAK");
const state = require("../state");

class LET$INSTANCE extends LET {
  before() {
    this.value = new EXPRESSION(
      this.declaration.value.node.map((node) => {
        let parts = node.split(".");
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
