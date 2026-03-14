const Identifier = require("./Identifier");
const Node = require("./Node");
const graph = require("../../graph");
const $CALL = require("../$nuc/$CALL");
const Expression = require("../../Expression");
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
    const root = rootCallee(this.node);
    const identifier = new Identifier(root);
    return identifier.last;
  }

  get function() {
    const root = rootCallee(this.node);
    return new Identifier(root);
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
        const expression = new Expression(`(${json})`);
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

  graph(scope) {
    return [
      this.first.graph(scope),
      traverseCallee(this.node, (callee) =>
        callee.arguments.map((arg) => Node.convert(arg).graph(scope))
      ),
    ];
  }

  walk() {
    return [
      this.first.walk(),
      traverseCallee(this.node, (callee) =>
        callee.arguments.map((arg) => Node.convert(arg).walk())
      ),
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

function traverseCallee(node, func) {
  let current = node;
  const acc = [];

  while (
    ["MemberExpression", "CallExpression"].includes(
      current.type || current.object?.type || current.callee?.type
    )
  ) {
    if (current.callee) {
      acc.push(func(current));
    }

    current = current.object || current.callee;
  }

  return acc;
}

function resolveArguments(scope, node) {
  let current = node;

  while (
    ["MemberExpression", "CallExpression"].includes(
      current?.type || current.object?.type
    )
  ) {
    if (current.callee) {
      current.arguments = current.arguments.map((arg) =>
        Node.convert(arg).resolve(scope)
      );
    }

    current = current.callee || current.object;
  }
}

module.exports = Call;
