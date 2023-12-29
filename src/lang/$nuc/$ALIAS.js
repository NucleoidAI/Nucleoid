const $ = require("./$");
const ALIAS = require("../../nuc/ALIAS");
const $EXPRESSION = require("./$EXPRESSION");
const Identifier = require("../ast/Identifier");

function build(alias, name, value) {
  const statement = new $ALIAS();
  statement.als = alias;
  statement.nme = name;
  statement.val = value;
  return statement;
}

class $ALIAS extends $ {
  before(scope) {
    const expression = $EXPRESSION(this.val);
    this.val = expression.run(scope);
  }

  run() {
    const name = new Identifier(this.nme);

    const statement = new ALIAS(name);
    statement.alias = new Identifier(this.als);
    statement.name = name;
    statement.value = this.val;
    return statement;
  }
}

module.exports = build;
