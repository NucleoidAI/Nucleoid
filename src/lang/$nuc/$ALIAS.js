const $ = require("./$");
const ALIAS = require("../../nuc/ALIAS");
const $EXPRESSION = require("./$EXPRESSION");
const Identifier = require("../ast/Identifier");

function build(alias, name, value) {
  const statement = new $ALIAS();
  statement.alias = alias;
  statement.name = name;
  statement.value = value;
  return statement;
}

class $ALIAS extends $ {
  before(scope) {
    const expression = $EXPRESSION(this.value);
    this.value = expression.run(scope);
  }

  run() {
    const name = new Identifier(this.name);

    const statement = new ALIAS(name);
    statement.alias = new Identifier(this.alias);
    statement.name = name;
    statement.value = this.value;
    return statement;
  }
}

module.exports = build;
