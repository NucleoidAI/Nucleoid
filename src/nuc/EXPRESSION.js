const graph = require("../graph");
const Evaluation = require("../lang/Evaluation");
const NODE = require("./NODE");
const state = require("../state");
const serialize = require("../lib/serialize");

class EXPRESSION {
  constructor(tokens) {
    this.instanceof = this.constructor.name;
    this.tokens = tokens;
  }

  before(scope) {
    this.tokens.map((node) => {
      if (
        node.type === "MemberExpression" &&
        graph.retrieve(node.first) &&
        node.last.toString() === "value"
      ) {
        const value = state.expression(scope, {
          value: node.object.generate(scope),
        });

        const { parse } = require("../lang/estree/parser");
        const newValue = parse(serialize(value, "state"), false);
        Object.assign(node.node, newValue);
      }
    });
  }

  run(scope) {
    const expression = this.tokens.traverse((node) => {
      const evaluation = node.generate(scope);

      if (
        scope.retrieve(node) ||
        (node.type === "MemberExpression" && graph.retrieve(node.first))
      ) {
        try {
          const test = state.expression(scope, { value: evaluation });
          if (test === undefined) {
            return "UNDEFINED";
          }
        } catch (error) {
          return "UNDEFINED";
        }
      }

      return evaluation;
    });

    if (!expression.includes("UNDEFINED")) {
      return new Evaluation(expression.join(""));
    }
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
