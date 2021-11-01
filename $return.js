const $ = require("./$");
const RETURN = require("./return");

module.exports = function (expression) {
  const statement = new $RETURN();
  statement.expression = expression;
  return statement;
};

class $RETURN extends $ {
  run() {
    const statement = new RETURN();
    statement.expression = this.expression.run();
    return statement;
  }
}
