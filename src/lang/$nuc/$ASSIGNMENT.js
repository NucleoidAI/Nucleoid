const $PROPERTY = require("./$PROPERTY");
const $VARIABLE = require("./$VARIABLE");
const Local = require("../../lib/local");
const $LET = require("./$LET");
const $ = require("./$");
const Id = require("../../lib/identifier");

function construct(left, right) {
  let statement = new $ASSIGNMENT();
  statement.left = left.split(".");
  statement.right = right;
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
      let parts = Id.splitLast(this.left.join("."));

      if (Local.check(scope, parts[1])) {
        return $LET(this.left.join("."), this.right);
      } else {
        return $PROPERTY(parts[1], parts[0], this.right);
      }
    }
  }
}

module.exports = construct;
