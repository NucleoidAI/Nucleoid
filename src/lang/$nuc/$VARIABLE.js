const $ = require("./$");
const VARIABLE = require("../../nuc/VARIABLE");
const $EXPRESSION = require("./$EXPRESSION");
const Identifier = require("../ast/Identifier");

function build(name, value) {
  const statement = new $VARIABLE();
  statement.name = name;
  statement.value = value;
  return statement;
}

class $VARIABLE extends $ {
  before(scope) {
    if (this.prepared) {
      return;
    }

    const expression = $EXPRESSION(this.value);
    this.value = expression.run(scope);
  }

  run() {
    const name = new Identifier(this.name);
    const statement = new VARIABLE(name);
    statement.name = name;
    statement.value = this.value;
    return statement;
  }
}

module.exports = build;
