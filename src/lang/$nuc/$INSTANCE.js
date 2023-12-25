const $ = require("./$");
const graph = require("../../graph");
const CLASS = require("../../nuc/CLASS");
const $LET = require("./$LET");
const Identifier = require("../ast/Identifier");
const random = require("../../lib/random");
const Instruction = require("../../instruction");

function build(cls, object, name, args = []) {
  let statement = new $INSTANCE();
  statement.class = cls;
  statement.object = object;
  statement.name = name;
  statement.arguments = args;
  return statement;
}

class $INSTANCE extends $ {
  before() {
    if (this.prepared) {
      return;
    }

    if (!this.object && !this.name) {
      this.name = random(16, true);
    }

    this.prepared = true;
  }

  run(scope) {
    const OBJECT = require("../../nuc/OBJECT");

    const cls = new Identifier(`$${this.class.name}`);
    const name = new Identifier(this.name);
    const args = this.arguments.map((arg) => new Identifier(arg));

    if (!graph.retrieve(cls)) {
      throw ReferenceError(`${cls} is not defined`);
    }

    if (this.object && name.toString() === "value") {
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
        throw ReferenceError(`${new Identifier(this.object)} is not defined`);
      }

      if (graph.retrieve(object.first) instanceof CLASS) {
        const OBJECT$CLASS = require("../../nuc/OBJECT$CLASS");

        const statement = new OBJECT$CLASS(`${object}.${name}`);
        statement.class = graph.retrieve(cls);
        statement.name = name;
        statement.object = graph.retrieve(object);
        return [
          new Instruction(scope, statement, true, true, false, null, true),
          new Instruction(scope, statement, false, false, true, null, true),
        ];
      } else {
        const statement = new OBJECT(`${object}.${name}`);
        statement.class = graph.retrieve(cls);
        statement.name = name;
        statement.object = graph.retrieve(object);
        statement.arguments = args;
        return statement;
      }
    }

    let statement = new OBJECT(name);
    statement.class = graph.retrieve(cls);
    statement.name = name;
    statement.arguments = args;
    return statement;
  }
}

module.exports = build;
