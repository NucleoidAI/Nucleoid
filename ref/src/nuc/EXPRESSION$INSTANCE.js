const EXPRESSION = require("./EXPRESSION");

class EXPRESSION$INSTANCE extends EXPRESSION {
  before(scope) {
    const $instance = scope.$instance;

    if ($instance) {
      this.tokens.traverse((node) => {
        const identifiers = [node.walk()].flat(Infinity);

        for (const identifier of identifiers) {
          if (identifier.first.toString() === $instance.class.name.toString()) {
            identifier.first = $instance.resolve();
          }
        }
      });
    }

    super.before(scope);
  }
}

module.exports = EXPRESSION$INSTANCE;
