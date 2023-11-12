const IF = require("./IF");
const Identifier = require("../lang/ast/Identifier");

class IF$INSTANCE extends IF {
  before(scope) {
    this.condition.tokens.traverse((node) => {
      if (node instanceof Identifier) {
        if (node.first.toString() === this.class.name.toString()) {
          node.first = this.instance.name; // TODO Go to object root
        }
      }
    });

    this.key = `if(${this.condition.tokens})`;

    super.before(scope);
  }

  run(scope) {
    scope.instances[this.class.name] = this.instance;
    return super.run(scope);
  }
}

module.exports = IF$INSTANCE;
