const Identifier = require("./Identifier");
const Node = require("./Node");
const graph = require("../../graph");
const $CALL = require("../$nuc/$CALL");
const Expression = require("./Expression");
const _ = require("lodash");
const FUNCTION = require("../../nuc/FUNCTION");
const serialize = require("../../lib/serialize");

class Call extends Node {
  get first() {
    if (["Identifier", "ThisExpression"].includes(this.node.callee.type)) {
      return new Identifier(this.node.callee);
    } else if (this.node.callee.type === "MemberExpression") {
      return new Identifier(root(this.node).object);
    } else {
      throw new Error(`Unknown call type ${this.node.callee.type}`);
    }
  }

  resolve(scope) {
    if (scope) {
      const cloned = _.cloneDeep(this.node);
      const rootNode = root(cloned);
      const first = new Identifier(rootNode?.object || rootNode?.callee);

      if (graph.retrieve(first) instanceof FUNCTION) {
        const stack = require("../../stack");
        const { value } = stack.process(
          [$CALL(this.node.callee, this.node.arguments)],
          scope
        );
        const json = serialize(value, "state");
        const expression = new Expression(json);
        return expression.resolve(scope);
      } else {
        const resolved = first.resolve(scope);

        if (rootNode.object) {
          rootNode.object = resolved;
        } else {
          rootNode.callee = resolved;
        }

        resolveArguments(scope, cloned);

        return cloned;
      }
    } else {
      return this.node;
    }
  }
}

function root(node) {
  let current = node;

  while (
    !["Identifier", "ThisExpression"].includes(
      current.object?.type || current.callee?.type
    )
  ) {
    current = current.object || current.callee;
  }

  return current;
}

function resolveArguments(scope, node) {
  let current = node;

  while (
    !["Identifier", "ThisExpression"].includes(
      current.object?.type || current.callee?.type
    )
  ) {
    if (current.callee) {
      current.arguments = current.arguments
        .map((arg) => new Expression(arg))
        .map((arg) => arg.resolve(scope));
    }

    current = current.object || current.callee;
  }
}

module.exports = Call;
