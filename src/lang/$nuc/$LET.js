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
  statement.name = name;
  statement.value = value;
  statement.constant = constant;
  statement.reassign = reassign;
  return statement;
}

class $LET extends $ {
  before(scope) {
    if (this.prepared) {
      return;
    }

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

    this.prepared = true;
  }

  run(scope) {
    // TODO Rename this to `identifier`?
    const name = new Identifier(this.name);

    if (
      name.type === "MemberExpression" &&
      !scope.retrieve(name.object, true)
    ) {
      throw ReferenceError(`${name.object} is not defined`);
    }

    if (name.type === "MemberExpression" && name.last.toString() === "value") {
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
