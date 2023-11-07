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
  before() {
    this.key = this.name.toString();
  }

  run(scope) {
    const expression = this.value.run(scope);

    if (["EXPRESSION", "REFERENCE"].includes(expression.instanceof)) {
      const statement = new VARIABLE(this.key);
      statement.name = this.name;
      statement.value = expression;
      return statement;
    } else if (expression instanceof OBJECT) {
      const statement = expression;
      statement.name = this.name;
      return statement;
    }
  }
}

module.exports = build;
