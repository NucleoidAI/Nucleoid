const $ = require("./$");
const graph = require("./graph");
const CLASS = require("./class");
const PROPERTY$CLASS = require("./property$class");
const PROPERTY = require("./property");
const OBJECT$CLASS = require("./object$class");
const REFERENCE = require("./reference");
const PROPERTY$REFERENCE = require("./property$reference");
const Local = require("./local");
const FUNCTION = require("./function");

module.exports = function (object, name, value) {
  let statement = new $PROPERTY();
  statement.object = object;
  statement.name = name;
  statement.value = value;
  return statement;
};

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

    let value = this.value.run();

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
