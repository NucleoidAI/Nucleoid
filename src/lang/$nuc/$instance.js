const $ = require("./$");
const graph = require("../../graph");
const OBJECT = require("../../object");
const OBJECT$CLASS = require("../../object$class");
const CLASS = require("../../class");
const Local = require("../../utils/local");
const $LET = require("./$let");

function construct(cls, name, object, args) {
  let statement = new $INSTANCE();
  statement.class = `$${cls}`;
  statement.name = name;
  statement.object = object;
  statement.args = args;
  return statement;
}

class $INSTANCE extends $ {
  run(scope) {
    if (graph[this.class] === undefined)
      throw ReferenceError(`${this.class} is not defined`);

    if (this.object !== undefined && this.name === "value") {
      throw TypeError("Cannot use 'value' as a property");
    }

    let local = this.object + "." + this.name;
    if (Local.check(scope, this.object)) {
      let instance = new $INSTANCE();
      instance.class = this.class;
      return $LET(local, instance);
    }

    if (this.object !== undefined && graph[this.object] === undefined)
      throw ReferenceError(`${this.object} is not defined`);

    if (
      this.object &&
      (graph[this.object] instanceof CLASS ||
        graph[this.object] instanceof OBJECT$CLASS)
    ) {
      let statement = new OBJECT$CLASS();
      statement.class = graph[this.class];
      statement.name = this.name;
      statement.object = graph[this.object];
      return statement;
    }

    let statement = new OBJECT();
    statement.class = graph[this.class];
    statement.name = this.name;
    statement.object = graph[this.object];
    statement.args = this.args;
    return statement;
  }
}

module.exports = construct;
