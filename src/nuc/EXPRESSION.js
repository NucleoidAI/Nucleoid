const graph = require("../graph");
const Evaluation = require("../lang/ast/Evaluation");
const Identifier = require("../lang/ast/Identifier");

class EXPRESSION {
  constructor(tokens) {
    this.instanceof = this.constructor.name;
    this.tokens = tokens;
  }

  before() {}

  run(scope) {
    const expression = this.tokens.traverse((node) => node.generate(scope));

    return new Evaluation(expression.join(""));
  }

  next() {}

  graph() {
    return this.tokens.map((node) => {
      if (node instanceof Identifier) {
        let current = node;

        while (current) {
          const dependency = graph.retrieve(current);

          if (dependency) {
            return dependency;
          }

          current = current.object;
        }
      }
    });
  }
}

module.exports = EXPRESSION;
