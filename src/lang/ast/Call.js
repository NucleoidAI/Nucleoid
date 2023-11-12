const Identifier = require("./Identifier");
const Node = require("./Node");
const graph = require("../../graph");
const $CALL = require("../$nuc/$CALL");
const ESTree = require("../estree/generator");

class Call extends Node {
  resolve(scope) {
    const name = new Identifier(this.node.callee);

    if (graph.retrieve(name)) {
      const stack = require("../../stack");

      return stack.process(
        [$CALL(this.node.callee, this.node.arguments)],
        scope
      );
    } else {
      const Expression = require("./Expression");
      const resolvedName = name.resolve(scope);

      const args = this.node.arguments
        .map((arg) => new Expression(arg))
        .map((arg) => arg.resolve(scope));

      return {
        type: "CallExpression",
        callee: resolvedName,
        arguments: args,
      };
    }
  }

  generate(scope) {
    const resolved = this.resolve(scope);

    if (typeof resolved === "string") {
      return resolved;
    } else {
      return ESTree.generate(resolved);
    }
  }
}

module.exports = Call;
