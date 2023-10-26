const $PROPERTY = require("./$PROPERTY");
const $VARIABLE = require("./$VARIABLE");
const Local = require("../../lib/local");
const $LET = require("./$LET");
const $ = require("./$");
const Id = require("../../lib/identifier");

function build(left, right, bracket) {
  let statement = new $ASSIGNMENT();
  statement.left = left;
  statement.right = right;
  statement.bracket = bracket;
  return statement;
}

class $ASSIGNMENT extends $ {
  run(scope) {
    if (this.left.length === 1) {
      const local = Local.check(scope, this.left[0]);
      if (local) {
        if (local.constant) {
          throw TypeError("Assignment to constant variable.");
        }

        return $LET(this.left[0], this.right);
      } else return $VARIABLE(this.left[0], this.right);
    } else {
      if (Local.check(scope, this.left.last)) {
        return $LET(this.left.join("."), this.right);
      } else {
        return $PROPERTY(this.left.first, this.left.last, this.right);
      }
    }
  }
}

module.exports = build;
