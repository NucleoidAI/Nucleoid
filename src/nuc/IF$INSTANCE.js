const IF = require("./IF");
const EXPRESSION = require("./EXPRESSION");
const Id = require("../lib/identifier");

class IF$INSTANCE extends IF {
  before() {
    this.condition = new EXPRESSION(
      this.declaration.condition.node.map((node) => {
        let parts = node.split(".");
        if (parts[0] === this.class.name)
          parts[0] = Id.serialize(this.instance);
        return parts.join(".");
      })
    );
  }

  run(scope) {
    scope.instances[this.class.name] = this.instance;
    return super.run(scope);
  }
}

module.exports = IF$INSTANCE;
