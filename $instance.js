var $ = require("./$");
var graph = require("./graph");
var OBJECT = require("./object");
var OBJECT$CLASS = require("./object$class");
var CLASS = require("./class");
var Local = require("./local");
var $LET = require("./$let");

module.exports = function(cls, name, object) {
  let statement = new $INSTANCE();
  statement.class = cls;
  statement.name = name;
  statement.object = object;
  return statement;
};

class $INSTANCE extends $ {
  run(scope) {
    if (graph[this.class] === undefined)
      throw new ReferenceError(`'${this.class}' is not defined`);

    if (this.object !== undefined && this.name === "value") {
      throw new TypeError("Cannot use 'value' as property");
    }

    let local = this.object + "." + this.name;
    if (Local.check(scope, this.object)) {
      let instance = new $INSTANCE();
      instance.class = this.class;
      return $LET(local, instance);
    }

    if (this.object !== undefined && graph[this.object] === undefined)
      throw new ReferenceError(`'${this.object}' is not defined`);

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
    return statement;
  }
}
