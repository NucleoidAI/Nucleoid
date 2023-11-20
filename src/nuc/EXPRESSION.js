const graph = require("../graph");
const Evaluation = require("../lang/Evaluation");
const NODE = require("./NODE");

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

  graph(scope) {
    return this.tokens.graph(scope, (node) => {
      const retrieved = graph.retrieve(node);

      if (retrieved) {
        return retrieved;
      } else {
        const temporary = new NODE(node);
        // TODO NODE Direct
        graph.graph[node] = temporary;
        return temporary;
      }
    });
  }
}

module.exports = EXPRESSION;
