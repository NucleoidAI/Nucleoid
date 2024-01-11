const $ = require("./$");
const graph = require("../../graph");
const CLASS = require("../../nuc/CLASS");
const PROPERTY$CLASS = require("../../nuc/PROPERTY$CLASS");
const PROPERTY = require("../../nuc/PROPERTY");
const OBJECT$CLASS = require("../../nuc/OBJECT$CLASS");
const Local = require("../../lib/local");
const FUNCTION = require("../../nuc/FUNCTION");
const Identifier = require("../ast/Identifier");
const $EXPRESSION = require("./$EXPRESSION");
const Instruction = require("../../Instruction");

function build(object, name, value) {
  let statement = new $PROPERTY();
  statement.obj = object;
  statement.nme = name;
  statement.val = value;
  return statement;
}

class $PROPERTY extends $ {
  before(scope) {
    const expression = $EXPRESSION(this.val);
    this.val = expression.run(scope);
  }

  run(scope) {
    let object = new Identifier(this.obj);
    const name = new Identifier(this.nme);

    if (object.toString() === "this") {
      object = Local.object(scope);
    }

    if (!graph.retrieve(object)) {
      throw ReferenceError(`${object} is not defined`);
    }

    if (name.toString() === "value" && !(graph[object] instanceof FUNCTION)) {
      throw TypeError("Cannot use 'value' as a name");
    }

    const cls = graph.retrieve(object);
    if (cls instanceof CLASS || cls instanceof OBJECT$CLASS) {
      let statement = new PROPERTY$CLASS(`${object}.${name}`);
      statement.class = cls;
      statement.object = graph.retrieve(object);
      statement.name = name;
      statement.value = this.val;
      return [
        new Instruction(scope, statement, true, true, false, false),
        new Instruction(scope, statement, false, false, true, true),
      ];
    }

    let statement = new PROPERTY(`${object}.${name}`);
    statement.object = graph.retrieve(object);
    statement.name = name;
    statement.value = this.val;
    return new Instruction(scope, statement, true, true, true, true);
  }
}

module.exports = build;
