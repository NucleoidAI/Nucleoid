const $ = require("./$");
const THROW = require("../../throw");

module.exports = function (exception) {
  let statement = new $THROW();
  statement.exception = exception;
  return statement;
};

class $THROW extends $ {
  run() {
    let statement = new THROW();
    statement.exception = this.exception;
    return statement;
  }
}
