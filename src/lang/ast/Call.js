const Identifier = require("./Identifier");
const Node = require("./Node");

class Call extends Node {
  resolve(scope) {
    const Expression = require("./Expression");

    const name = new Identifier(this.node.callee);
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

module.exports = Call;
