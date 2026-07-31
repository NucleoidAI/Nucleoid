const IF = require("./IF");

class IF$INSTANCE extends IF {
  before(scope) {
    const $instance = scope.$instance;

    if (!$instance) {
      throw "Declaration missing instance in scope";
    }

    this.condition.tokens.traverse((node) => {
      const identifiers = [node.walk()].flat(Infinity);

      for (const identifier of identifiers) {
        if ($instance.class.name.toString() === identifier.first.toString()) {
          identifier.first = $instance.resolve();
        }
      }
    });

    this.key = `if(${this.condition.tokens})`;

    super.before(scope);
  }
}

module.exports = IF$INSTANCE;
