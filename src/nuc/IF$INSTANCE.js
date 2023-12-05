const IF = require("./IF");

class IF$INSTANCE extends IF {
  before(scope) {
    this.condition.tokens.traverse((node) => {
      const identifiers = [node.walk()].flat(Infinity);

      for (const identifier of identifiers) {
        if (identifier.first.toString() === this.class.name.toString()) {
          identifier.first.node.name = this.instance.name;
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
