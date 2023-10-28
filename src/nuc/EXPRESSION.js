const graph = require("../graph");
const Evaluation = require("../lang/ast/Evaluation");
const Identifier = require("../lang/ast/Identifier");
const Call = require("../lang/ast/Call");

class EXPRESSION {
  constructor(tokens) {
    this.instanceof = this.constructor.name;
    this.tokens = tokens;
  }

  before() {}

  run() {
    const tokens = this.tokens;

    const expression = tokens.traverse((token) => token.resolve(true));

    const transactions = tokens.map((token) => {
      if (token instanceof Call) {
        return {
          exec: token.resolve(true),
        };
      }
    });

    return new Evaluation(expression.join(""), transactions);
  }

  next() {}

  graph() {
    return this.tokens.map((token) => {
      if (token instanceof Identifier) {
        const name = token.resolve();
        if (graph[name]) {
          return name;
        }
      }
    });
  }
}

module.exports = EXPRESSION;
