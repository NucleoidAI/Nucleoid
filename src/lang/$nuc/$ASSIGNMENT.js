const $PROPERTY = require("./$PROPERTY");
const $VARIABLE = require("./$VARIABLE");
const Local = require("../../lib/local");
const $LET = require("./$LET");
const $ = require("./$");

function build(left, right, bracket) {
  let statement = new $ASSIGNMENT();
  statement.left = left;
  statement.right = right;
  statement.bracket = bracket;
  return statement;
}

class $ASSIGNMENT extends $ {
  run(scope) {
    if (this.left.node.type === "Identifier") {
      const local = Local.check(scope, this.left.resolve());

      if (local) {
        if (local.constant) {
          throw TypeError("Assignment to constant variable.");
        }

        return $LET(this.left, this.right);
      } else {
        return $VARIABLE(this.left, this.right);
      }
    } else {
      if (scope.retrieve(this.left)) {
        return $LET(this.left.join("."), this.right);
      } else {
        return $PROPERTY(this.left.object, this.left.last, this.right);
      }
    }
  }
}

module.exports = build;
