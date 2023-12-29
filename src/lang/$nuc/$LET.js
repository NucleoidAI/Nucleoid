const $ = require("./$");
const graph = require("../../graph");
const CLASS = require("../../nuc/CLASS");
const LET = require("../../nuc/LET");
const LET$CLASS = require("../../nuc/LET$CLASS");
const LET$OBJECT = require("../../nuc/LET$OBJECT");
const EXPRESSION = require("../../nuc/EXPRESSION");
const REFERENCE = require("../../nuc/REFERENCE");
const $EXPRESSION = require("./$EXPRESSION");
const Identifier = require("../ast/Identifier");
const $INSTANCE = require("./$INSTANCE");

function build(name, value, constant, reassign) {
  let statement = new $LET();
  statement.nme = name;
  statement.val = value;
  statement.cst = constant;
  statement.ras = reassign;
  return statement;
}

class $LET extends $ {
  before(scope) {
    if (
      this.val.type === "NewExpression" &&
      graph.retrieve(`$${this.val.callee.name}`) instanceof CLASS
    ) {
      this.val = $INSTANCE(this.val.callee, null, null, this.val.arguments);
      this.val.before(scope);
    } else {
      const expression = $EXPRESSION(this.val);
      this.val = expression.run(scope);
    }
  }

  run(scope) {
    // TODO Rename this to `identifier`?
    const name = new Identifier(this.nme);

    if (
      name.type === "MemberExpression" &&
      !scope.retrieve(name.object, true)
    ) {
      throw ReferenceError(`${name.object} is not defined`);
    }

    if (name.type === "MemberExpression" && name.last.toString() === "value") {
      throw TypeError("Cannot use 'value' in local");
    }

    let value = this.val;
    if (value instanceof EXPRESSION || value instanceof REFERENCE) {
      const cls = this.val.tokens.find((node) => {
        const identifiers = [node.walk()].flat(Infinity);

        for (const identifier of identifiers) {
          const cls = graph.retrieve(identifier.first);
          if (cls instanceof CLASS) {
            return cls;
          }
        }
      });

      if (cls) {
        let statement = new LET$CLASS();
        statement.class = cls;
        statement.name = name;
        statement.value = value;
        statement.constant = this.cst;
        return statement;
      }

      let statement = new LET();
      statement.name = name;
      statement.value = value;
      statement.constant = this.cst;
      statement.reassign = this.ras;
      return statement;
    } else if (value.iof === "$INSTANCE") {
      const object = this.val.run(scope);

      const statement = new LET$OBJECT();
      statement.name = name;
      statement.object = object;
      statement.constant = this.cst;

      return [object, statement];
    }
  }
}

module.exports = build;
