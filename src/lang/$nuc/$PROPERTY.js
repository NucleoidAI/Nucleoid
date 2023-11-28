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
const Identifier = require("../ast/Identifier");
const $EXPRESSION = require("./$EXPRESSION");

function build(object, name, value) {
  let statement = new $PROPERTY();
  statement.object = object;
  statement.name = name;
  statement.value = value;
  return statement;
}

class $PROPERTY extends $ {
  before(scope) {
    const expression = $EXPRESSION(this.value);
    this.value = expression.run(scope);
  }

  run(scope) {
    let object = new Identifier(this.object);
    const name = new Identifier(this.name);
    const key = `${object}.${name}`;

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
      let statement = new PROPERTY$CLASS(key);
      statement.class = cls;
      statement.object = graph.retrieve(object);
      statement.name = name;
      statement.value = this.value;
      return statement;
    }

    let statement = new PROPERTY(key);
    statement.object = graph.retrieve(object);
    statement.name = name;
    statement.value = this.value;
    return statement;
  }
}

module.exports = build;
