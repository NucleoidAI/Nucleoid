const { generate } = require("../estree/generator");
class Object {
  constructor(node) {
    this.node = node;
  }

  resolve() {
    return generate(this.node);
  }
}

module.exports = Object;
