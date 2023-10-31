const $ = require("./$");
const EXPRESSION = require("../../nuc/EXPRESSION");
const acorn = require("acorn");
const Expression = require("../ast/Expression");

function build(node) {
  if (typeof node === "string") {
    const [{ expression }] = acorn.parse(node, { ecmaVersion: 2020 }).body;
    node = new Expression(expression);
  }

  const statement = new $EXPRESSION();
  statement.node = node;
  return statement;
}

class $EXPRESSION extends $ {
  run() {
    let statement = new EXPRESSION();
    statement.node = this.node;
    return statement;
  }
}

module.exports = build;
