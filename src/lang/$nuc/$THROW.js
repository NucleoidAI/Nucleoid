const $ = require("./$");
const THROW = require("../../nuc/THROW");
const Expression = require("../ast/Expression");

function build(exception) {
  let statement = new $THROW();
  statement.exception = exception;
  return statement;
}

class $THROW extends $ {
  run() {
    const exception = new Expression(this.exception);
    return new THROW(exception);
  }
}

module.exports = build;
