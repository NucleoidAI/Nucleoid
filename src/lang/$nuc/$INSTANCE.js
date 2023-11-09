const $ = require("./$");
const graph = require("../../graph");
const OBJECT = require("../../nuc/OBJECT");
const OBJECT$CLASS = require("../../nuc/OBJECT$CLASS");
const CLASS = require("../../nuc/CLASS");
const $LET = require("./$LET");
const Identifier = require("../ast/Identifier");
const random = require("../../lib/random");

function build(cls, object, name, args) {
  let statement = new $INSTANCE();
  statement.class = cls;
  statement.object = object;
  statement.name = name;
  statement.args = args;
  return statement;
}

class $INSTANCE extends $ {
  before() {
    if (!this.object && !this.name) {
      this.name = random(16, true);
    }
  }

  run(scope) {
    const cls = new Identifier(`$${this.class.name}`);
    const name = new Identifier(this.name);

    if (!graph.retrieve(cls)) {
      throw ReferenceError(`${cls} is not defined`);
    }

    if (this.object !== undefined && name.generate() === "value") {
      throw TypeError("Cannot use 'value' as a property");
    }

    if (this.object) {
      const object = new Identifier(this.object);

      if (scope.retrieve(object)) {
        let instance = new OBJECT();
        instance.class = this.class;
        return $LET(name, instance);
      }

      if (!graph.retrieve(object)) {
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
        const statement = new OBJECT(`${object}.${name}`);
        statement.class = graph.retrieve(cls);
        statement.name = name;
        statement.object = graph.retrieve(object);
        statement.args = this.args;
        return statement;
      }
    }

    let statement = new OBJECT(name);
    statement.class = graph.retrieve(cls);
    statement.name = name;
    statement.args = this.args;
    return statement;
  }
}

module.exports = build;
