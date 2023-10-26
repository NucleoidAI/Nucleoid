const $ = require("./$");
const EXPRESSION = require("../../nuc/EXPRESSION");
const acorn = require("acorn");
const Expression = require("../ast/Expression");

function build(tokens) {
  if (typeof tokens === "string") {
    const [{ expression }] = acorn.parse(tokens, { ecmaVersion: 2020 }).body;
    tokens = new Expression(expression);
  }

  const statement = new $EXPRESSION();
  statement.tokens = tokens;
  return statement;
}

class $EXPRESSION extends $ {
  run() {
    let statement = new EXPRESSION();
    statement.tokens = this.tokens;
    return statement;
  }
}

module.exports = build;
