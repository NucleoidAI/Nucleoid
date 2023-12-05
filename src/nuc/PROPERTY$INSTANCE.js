const PROPERTY = require("./PROPERTY");

class PROPERTY$INSTANCE extends PROPERTY {
  before(scope) {
    this.value.tokens.traverse((node) => {
      const identifiers = [node.walk()].flat(Infinity);

      for (const identifier of identifiers) {
        if (identifier.first.toString() === this.class.name.toString()) {
          identifier.first = this.object.resolve();
        }
      }
    });

    super.before(scope);
  }
}

module.exports = PROPERTY$INSTANCE;
