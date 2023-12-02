const $ = require("./$");
const graph = require("../../graph");
const CLASS = require("../../nuc/CLASS");
const LET = require("../../nuc/LET");
const LET$CLASS = require("../../nuc/LET$CLASS");
const OBJECT = require("../../nuc/OBJECT");
const LET$OBJECT = require("../../nuc/LET$OBJECT");
const EXPRESSION = require("../../nuc/EXPRESSION");
const REFERENCE = require("../../nuc/REFERENCE");
const Local = require("../../lib/local");
const $EXPRESSION = require("./$EXPRESSION");
const Identifier = require("../ast/Identifier");
const $INSTANCE = require("./$INSTANCE");

function build(name, value, constant, reassign) {
  let statement = new $LET();
  statement.name = name;
  statement.value = value;
  statement.constant = constant;
  statement.reassign = reassign;
  return statement;
}

class $LET extends $ {
  before(scope) {
    if (
      this.value.type === "NewExpression" &&
      graph.retrieve(`$${this.value.callee.name}`) instanceof CLASS
    ) {
      this.value = $INSTANCE(
        this.value.callee,
        null,
        null,
        this.value.arguments
      );
      this.value.before(scope);
    } else {
      const expression = $EXPRESSION(this.value);
      this.value = expression.run(scope);
    }
  }

  run(scope) {
    const name = new Identifier(this.name);
    const object = this.object ? new Identifier(this.object) : undefined;

    if (object !== undefined && !Local.check(scope, object)) {
      throw ReferenceError(`${name} is not defined`);
    }

    if (object?.last.toString() === "value") {
      throw TypeError("Cannot use 'value' in local");
    }

    let value = this.value;
    if (value instanceof EXPRESSION || value instanceof REFERENCE) {
      const cls = this.value.tokens.find((node) => {
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
        statement.constant = this.constant;
        return statement;
      }

      let statement = new LET();
      statement.object = this.object;
      statement.name = name;
      statement.value = value;
      statement.constant = this.constant;
      statement.reassign = this.reassign;
      return statement;
    } else if (value.iof === "$INSTANCE") {
      const object = this.value.run(scope);

      const statement = new LET$OBJECT();
      statement.name = name;
      statement.object = object;
      statement.constant = this.constant;

      return [object, statement];
    }
  }
}

module.exports = build;
