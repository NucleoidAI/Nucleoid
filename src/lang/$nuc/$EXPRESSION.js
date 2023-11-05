const $ = require("./$");
const EXPRESSION = require("../../nuc/EXPRESSION");
const Expression = require("../ast/Expression");

function build(node) {
  if (typeof node === "string") {
    const { parse } = require("../estree/parser");
    const string = node;
    node = new Expression(parse(string, false));
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
