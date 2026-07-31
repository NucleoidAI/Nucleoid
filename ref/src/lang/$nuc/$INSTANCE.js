const $ = require("./$");
const graph = require("../../graph");
const CLASS = require("../../nuc/CLASS");
const $LET = require("./$LET");
const Identifier = require("../ast/Identifier");
const random = require("../../lib/random");
const Instruction = require("../../Instruction");

function build(cls, object, name, args = []) {
  let statement = new $INSTANCE();
  statement.cls = cls;
  statement.obj = object;
  statement.nme = name;
  statement.args = args;

  return statement;
}

class $INSTANCE extends $ {
  before() {
    if (!this.obj && !this.nme) {
      this.nme = random(16, true);
    }
  }

  run(scope) {
    const OBJECT = require("../../nuc/OBJECT");

    const cls = new Identifier(`$${this.cls.name}`);
    const name = new Identifier(this.nme);
    const args = this.args.map((arg) => new Identifier(arg));

    if (!graph.retrieve(cls)) {
      throw ReferenceError(`${cls} is not defined`);
    }

    if (this.obj && name.toString() === "value") {
      throw TypeError("Cannot use 'value' as a property");
    }

    if (this.obj) {
      const object = new Identifier(this.obj);

      if (scope.retrieve(object)) {
        let instance = new OBJECT();
        instance.class = this.cls;
        return $LET(name, instance);
      }

      if (!graph.retrieve(object)) {
        throw ReferenceError(`${new Identifier(this.obj)} is not defined`);
      }

      if (graph.retrieve(object.first) instanceof CLASS) {
        const OBJECT$CLASS = require("../../nuc/OBJECT$CLASS");

        const statement = new OBJECT$CLASS(`${object}.${name}`);
        statement.class = graph.retrieve(cls);
        statement.name = name;
        statement.object = graph.retrieve(object);
        return [
          new Instruction(scope, statement, true, true, false, false),
          new Instruction(scope, statement, false, false, true, true),
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
