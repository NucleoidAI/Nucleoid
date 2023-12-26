const $ = require("./$");
const THROW = require("../../nuc/THROW");
const Expression = require("../../Expression");

function build(exception) {
  let statement = new $THROW();
  statement.exc = exception;
  return statement;
}

class $THROW extends $ {
  run() {
    const exception = new Expression(this.exc);
    return new THROW(exception);
  }
}

module.exports = build;
