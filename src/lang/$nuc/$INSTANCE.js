const $ = require("./$");
const graph = require("../../graph");
const OBJECT = require("../../nuc/OBJECT");
const OBJECT$CLASS = require("../../nuc/OBJECT$CLASS");
const CLASS = require("../../nuc/CLASS");
const $LET = require("./$LET");
const Identifier = require("../ast/Identifier");

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
    const cls = new Identifier(`$${this.class}`);
    const name = this.name;

    if (!graph.retrieve(this.class)) {
      throw ReferenceError(`${cls} is not defined`);
    }

    if (this.object !== undefined && name.generate() === "value") {
      throw TypeError("Cannot use 'value' as a property");
    }

    if (this.object) {
      if (scope.retrieve(this.object)) {
        let instance = new OBJECT();
        instance.class = this.class;
        return $LET(name, instance);
      }

      const object = graph.retrieve(this.object);

      if (!object) {
        throw ReferenceError(`${this.object} is not defined`);
      }

      if (
        object &&
        (graph[object] instanceof CLASS ||
          graph[object] instanceof OBJECT$CLASS)
      ) {
        let statement = new OBJECT$CLASS();
        statement.class = graph[this.class];
        statement.name = name;
        statement.object = graph[this.object];
        return statement;
      } else {
        const statement = new OBJECT();
        statement.class = graph.retrieve(cls);
        statement.name = name;
        statement.object = graph.retrieve(this.object);
        statement.args = this.args;
        return statement;
      }
    }

    let statement = new OBJECT();
    statement.class = graph.retrieve(cls);
    statement.name = name;
    statement.args = this.args;
    return statement;
  }
}

module.exports = build;
