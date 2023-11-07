const $ = require("./$");
const ALIAS = require("../../nuc/ALIAS");

function build(alias, name, value) {
  const statement = new $ALIAS();
  statement.alias = alias;
  statement.name = name;
  statement.value = value;
  return statement;
}

class $ALIAS extends $ {
  before() {
    this.key = this.name.toString();
  }

  run(scope) {
    const value = this.value.run(scope);

    const statement = new ALIAS(this.key);
    statement.alias = this.alias;
    statement.name = this.name;
    statement.value = value;
    return statement;
  }
}

module.exports = build;
