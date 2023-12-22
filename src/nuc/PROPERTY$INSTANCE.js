const PROPERTY = require("./PROPERTY");

class PROPERTY$INSTANCE extends PROPERTY {
  before(scope) {
    const $instance = scope.$instance;

    if (!$instance) {
      throw "Declaration missing instance in scope";
    }

    this.value.tokens.traverse((node) => {
      const identifiers = [node.walk()].flat(Infinity);

      for (const identifier of identifiers) {
        if ($instance.class.name.toString() === identifier.first.toString()) {
          identifier.first = $instance.resolve();
        }
      }
    });

    super.before(scope);
  }
}

module.exports = PROPERTY$INSTANCE;
