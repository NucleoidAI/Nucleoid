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
    const callee = rootCallee(this.node);
    const identifier = new Identifier(callee);
    return identifier.first;
  }

  get object() {
    const callee = rootCallee(this.node);
    const identifier = new Identifier(callee);
    return identifier.object;
  }

  get last() {
    const callee = rootCallee(this.node);
    const identifier = new Identifier(callee);
    return identifier.last;
  }

  resolve(scope) {
    if (scope) {
      const clone = _.cloneDeep(this);
      const first = clone.first;

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
        if (first.type !== "Literal") {
          const resolved = first.resolve(scope);
          const rootNode = root(clone.node);

          if (resolved) {
            if (rootNode.object) {
              rootNode.object = resolved;
            } else {
              rootNode.callee = resolved;
            }
          }
        }

        resolveArguments(scope, clone.node);

        return clone.node;
      }
    } else {
      return this.node;
    }
  }

  graph() {
    return [
      this.first.graph(),
      this.node.arguments.map((arg) => Node.convertToAST(arg).graph()),
    ];
  }
}

function root(node) {
  let current = node;

  while (
    ["MemberExpression", "CallExpression"].includes(
      current.object?.type || current.callee?.type
    )
  ) {
    current = current.object || current.callee;
  }

  return current;
}

function rootCallee(node) {
  let current = node;
  let callee = node.callee;

  while (
    ["MemberExpression", "CallExpression"].includes(
      current.object?.type || current.callee?.type
    )
  ) {
    if (current.callee) {
      callee = current.callee;
    }

    current = current.object || current.callee;
  }

  return callee;
}

function resolveArguments(scope, node) {
  let current = node;

  while (
    ["MemberExpression", "CallExpression"].includes(
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
