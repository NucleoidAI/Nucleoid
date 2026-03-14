const $ = require("./$");
const VARIABLE = require("../../nuc/VARIABLE");
const $EXPRESSION = require("./$EXPRESSION");
const Identifier = require("../ast/Identifier");

function build(name, value) {
  const statement = new $VARIABLE();
  statement.nme = name;
  statement.val = value;
  return statement;
}

class $VARIABLE extends $ {
  before(scope) {
    const expression = $EXPRESSION(this.val);
    this.val = expression.run(scope);
  }

  run() {
    const name = new Identifier(this.nme);
    const statement = new VARIABLE(name);
    statement.name = name;
    statement.value = this.val;
    return statement;
  }
}

module.exports = build;
