const $ = require("./$");
const graph = require("../../graph");
const OBJECT = require("../../nuc/OBJECT");
const OBJECT$CLASS = require("../../nuc/OBJECT$CLASS");
const CLASS = require("../../nuc/CLASS");
const Local = require("../../lib/local");
const $LET = require("./$LET");

function build(cls, object, name, args) {
  let statement = new $INSTANCE();
  statement.class = cls;
  statement.name = name;
  statement.object = object;
  statement.args = args;
  return statement;
}

class $INSTANCE extends $ {
  run(scope) {
    const cls = `$${this.class.resolve()}`;
    const name = this.name.resolve();

    if (graph[cls] === undefined) {
      throw ReferenceError(`${cls} is not defined`);
    }

    if (this.object !== undefined && name === "value") {
      throw TypeError("Cannot use 'value' as a property");
    }

    if (this.object) {
      const object = this.object.resolve();
      const local = object + "." + name;
      if (Local.check(scope, object)) {
        let instance = new OBJECT();
        instance.class = this.class;
        return $LET(local, instance);
      }

      if (object !== undefined && graph[object] === undefined) {
        throw ReferenceError(`${this.object} is not defined`);
      }

      if (
        object &&
        (graph[object] instanceof CLASS ||
          graph[object] instanceof OBJECT$CLASS)
      ) {
        let statement = new OBJECT$CLASS();
        statement.class = graph[this.class];
        statement.name = this.name;
        statement.object = graph[this.object];
        return statement;
      } else {
        const statement = new OBJECT();
        statement.class = graph[cls];
        statement.name = name;
        statement.object = graph[object];
        statement.args = this.args;
        return statement;
      }
    }

    let statement = new OBJECT();
    statement.class = graph[cls];
    statement.name = name;
    statement.args = this.args;
    return statement;
  }
}

module.exports = build;
