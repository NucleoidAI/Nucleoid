const $ = require("./$");
const VARIABLE = require("../../nuc/VARIABLE");
const OBJECT = require("../../nuc/OBJECT");

function build(name, value) {
  const statement = new $VARIABLE();
  statement.name = name;
  statement.value = value;
  return statement;
}

class $VARIABLE extends $ {
  run(scope) {
    const expression = this.value.run(scope);

    if (["EXPRESSION", "REFERENCE"].includes(expression.instanceof)) {
      const statement = new VARIABLE();
      statement.name = this.name.resolve();
      statement.value = expression;
      return statement;
    } else if (expression instanceof OBJECT) {
      const statement = expression;
      statement.name = this.name.resolve();
      return statement;
    }
  }
}

module.exports = build;
