const PROPERTY = require("./PROPERTY");
const Identifier = require("../lang/ast/Identifier");

class PROPERTY$INSTANCE extends PROPERTY {
  before(scope) {
    this.value.tokens.traverse((node) => {
      if (node instanceof Identifier) {
        if (node.first.toString() === this.class.name.toString()) {
          node.first = this.object.name; // TODO Go to object root
        }
      }
    });

    super.before(scope);
  }
}

module.exports = PROPERTY$INSTANCE;
