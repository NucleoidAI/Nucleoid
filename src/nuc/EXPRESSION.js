const graph = require("../graph");
const Evaluation = require("../lang/Evaluation");
const NODE = require("./NODE");
const state = require("../state");
const serialize = require("../lib/serialize");
const { append } = require("../lang/estree/estree");
const Identifier = require("../lang/ast/Identifier");

class EXPRESSION {
  constructor(tokens) {
    this.instanceof = this.constructor.name;
    this.tokens = tokens;
  }

  before(scope) {
    const $class = scope.$class;

    if ($class) {
      const instance = scope.instance($class.name);

      if (instance) {
        this.tokens.traverse((node) => {
          const identifiers = [node.walk()].flat(Infinity);

          for (const identifier of identifiers) {
            if (identifier.first.toString() === $class.name.toString()) {
              identifier.first = instance.resolve();
            }
          }
        });
      }
    }

    this.tokens.map((node) => {
      if (
        node.type === "MemberExpression" &&
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

  run(scope, force = false) {
    const expression = this.tokens.traverse((node) => {
      const evaluation = node.generate(scope);

      if (
        scope.retrieve(node) ||
        (node.type === "MemberExpression" && graph.retrieve(node.first))
      ) {
        try {
          const test = state.expression(scope, { value: evaluation });
          if (test === undefined) {
            return "undefined";
          }
        } catch (error) {
          return "undefined";
        }
      }

      return evaluation;
    });

    if (force || !expression.includes("undefined")) {
      return new Evaluation(expression.join(""));
    }
  }

  next() {
    const Call = require("../lang/ast/Call");

    return this.tokens.map((ast) => {
      if (ast instanceof Call) {
        const object = ast.object;

        if (object) {
          const node = graph.retrieve(object);

          if (node) {
            return Object.values(node.next);
          }
        }
      }
    });
  }

  graph(scope) {
    return this.tokens.graph(scope, (node) => {
      const retrieved = graph.retrieve(node);

      if (retrieved) {
        return retrieved;
      } else {
        const REFERENCE = require("./REFERENCE");

        for (const { left, right } of node) {
          const test = graph.retrieve(left);

          if (test?.value && test.value instanceof REFERENCE) {
            const link = new Identifier(
              append(test.value.link.node, right.node)
            );
            return graph.retrieve(link);
          }
        }

        const temporary = new NODE(node);
        // TODO NODE Direct
        graph.graph[node] = temporary;
        return temporary;
      }
    });
  }
}

module.exports = EXPRESSION;
