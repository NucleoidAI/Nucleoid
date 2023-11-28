const LET = require("./LET");
const EXPRESSION = require("./EXPRESSION");
const Id = require("../lib/identifier");
const BREAK = require("./BREAK");
const state = require("../state");

class LET$INSTANCE extends LET {
  before(scope) {
    this.value.tokens.traverse((node) => {
      const identifiers = [node.walk()].flat(Infinity);

      for (const identifier of identifiers) {
        if (identifier.first.toString() === this.class.name.toString()) {
          identifier.first = this.instance.resolve();
        }
      }
    });

    super.before(scope);
  }
}

module.exports = LET$INSTANCE;
