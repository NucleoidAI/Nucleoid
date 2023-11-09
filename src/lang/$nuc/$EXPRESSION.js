const $ = require("./$");
const EXPRESSION = require("../../nuc/EXPRESSION");
const Expression = require("../ast/Expression");

function build(tokens) {
  if (!(tokens instanceof Expression)) {
    if (typeof tokens === "string") {
      const { parse } = require("../estree/parser");
      const string = tokens;
      tokens = new Expression(parse(string, false));
    } else {
      tokens = new Expression(tokens);
    }
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
