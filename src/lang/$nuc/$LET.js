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

function build(name, value, constant) {
  let statement = new $LET();
  statement.name = name;
  statement.value = value;
  statement.constant = constant;
  return statement;
}

class $LET extends $ {
  before(scope) {
    const expression = $EXPRESSION(this.value);
    this.value = expression.run(scope);
  }

  run(scope) {
    const name = this.name;
    const object = this.object;

    if (object !== undefined && !Local.check(scope, object)) {
      throw ReferenceError(`${name} is not defined`);
    }

    if (object?.last.toString() === "value") {
      throw TypeError("Cannot use 'value' in local");
    }

    let value = this.value;
    if (value instanceof EXPRESSION) {
      if (graph.retrieve(name) instanceof CLASS) {
        let statement = new LET$CLASS();
        statement.class = graph[name];
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
      return statement;
    } else if (value instanceof REFERENCE) {
      let statement = new LET();
      statement.object = this.object;
      statement.name = name;
      statement.value = value;
      statement.constant = this.constant;
      return statement;
    } else if (value instanceof OBJECT) {
      let object = new OBJECT();
      object.class = value.class;
      object.args = value.args;

      let statement = new LET$OBJECT();
      statement.name = name;
      statement.object = object;
      statement.constant = this.constant;

      return [object, statement];
    }
  }
}

module.exports = build;
