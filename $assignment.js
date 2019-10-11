var $PROPERTY = require("./$property");
var $VARIABLE = require("./$variable");
var Local = require("./local");
var $LET = require("./$let");
var $ = require("./$");
var Identifier = require("./identifier");

module.exports = function(left, right) {
  let statement = new $ASSIGNMENT();
  statement.left = left.split(".");
  statement.right = right;
  return statement;
};

class $ASSIGNMENT extends $ {
  run(scope) {
    if (this.left.length == 1) {
      return $VARIABLE(this.left[0], this.right);
    } else {
      if (Local.check(scope, this.left[0])) {
        return $LET(this.left.join("."), this.right);
      } else {
        let parts = Identifier.splitLast(this.left.join("."));
        return $PROPERTY(parts[1], parts[0], this.right);
      }
    }
  }
}
