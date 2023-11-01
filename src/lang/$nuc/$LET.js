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
const Id = require("../../lib/identifier");

function build(name, value, constant) {
  let statement = new $LET();
  statement.name = name;
  statement.value = value;
  statement.constant = constant;
  return statement;
}

class $LET extends $ {
  run(scope) {
    const object = this.object?.resolve();
    const name = this.name.resolve();

    if (object !== undefined && !Local.check(scope, object)) {
      throw ReferenceError(`${name} is not defined`);
    }

    if (this.object?.last.resolve() === "value") {
      throw TypeError("Cannot use 'value' in local");
    }

    let value = this.value.run();

    if (value instanceof EXPRESSION) {
      if (graph[name] && graph[name] instanceof CLASS) {
        let statement = new LET$CLASS();
        statement.class = graph[name];
        statement.name = this.name;
        statement.value = value;
        statement.constant = this.constant;
        return statement;
      }

      let statement = new LET();
      statement.object = this.object;
      statement.name = this.name;
      statement.value = value;
      statement.constant = this.constant;
      return statement;
    } else if (value instanceof REFERENCE) {
      let statement = new LET();
      statement.object = this.object;
      statement.name = this.name;
      statement.value = value;
      statement.constant = this.constant;
      return statement;
    } else if (value instanceof OBJECT) {
      let object = new OBJECT();
      object.class = value.class;
      object.args = value.args;

      let statement = new LET$OBJECT();
      statement.name = this.name;
      statement.object = object;
      statement.constant = this.constant;

      return [object, statement];
    }
  }
}

module.exports = build;
