const graph = require("../graph");
const Evaluation = require("../lang/ast/Evaluation");
const Identifier = require("../lang/ast/Identifier");
const Call = require("../lang/ast/Call");

class EXPRESSION {
  constructor(node) {
    this.instanceof = this.constructor.name;
    this.node = node;
  }

  before() {}

  run(scope) {
    const expression = this.node.traverse((node) => node.generate(scope));

    return new Evaluation(expression.join(""));
  }

  next() {}

  graph() {
    return this.node.map((node) => {
      if (node instanceof Identifier) {
        const name = node.generate();
        if (graph[name]) {
          return name;
        }
      }
    });
  }
}

module.exports = EXPRESSION;
