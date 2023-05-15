const $ = require("./$");
const graph = require("../../graph");
const CLASS = require("../../nuc/CLASS");
const PROPERTY$CLASS = require("../../nuc/PROPERTY$CLASS");
const PROPERTY = require("../../nuc/PROPERTY");
const OBJECT$CLASS = require("../../nuc/OBJECT$CLASS");
const REFERENCE = require("../../nuc/REFERENCE");
const PROPERTY$REFERENCE = require("../../nuc/PROPERTY$REFERENCE");
const Local = require("../../lib/local");
const FUNCTION = require("../../nuc/FUNCTION");

function construct(object, name, value) {
  let statement = new $PROPERTY();
  statement.object = object;
  statement.name = name;
  statement.value = value;
  return statement;
}

class $PROPERTY extends $ {
  run(scope) {
    if (this.object === "this") {
      this.object = Local.object(scope);
    }

    if (graph[this.object] === undefined) {
      throw ReferenceError(`${this.object} is not defined`);
    }

    if (this.name === "value" && !(graph[this.object] instanceof FUNCTION)) {
      throw TypeError("Cannot use 'value' as a name");
    }

    if (
      graph[this.object] instanceof CLASS ||
      graph[this.object] instanceof OBJECT$CLASS
    ) {
      let statement = new PROPERTY$CLASS();
      statement.object = graph[this.object];
      statement.name = this.name;
      statement.value = this.value.run();
      return statement;
    }

    let value = this.value.run(scope);

    if (value instanceof REFERENCE) {
      let statement = new PROPERTY$REFERENCE();
      statement.object = graph[this.object];
      statement.name = this.name;
      statement.value = value;
      return statement;
    }

    let statement = new PROPERTY();
    statement.object = graph[this.object];
    statement.name = this.name;
    statement.value = value;
    return statement;
  }
}

module.exports = construct;
