var $ = require("./$");
var graph = require("./graph");
var CLASS = require("./class");
var PROPERTY$CLASS = require("./property$class");
var PROPERTY = require("./property");
var OBJECT$CLASS = require("./object$class");

module.exports = function(object, name, value) {
  let statement = new $PROPERTY();
  statement.object = object;
  statement.name = name;
  statement.value = value;
  return statement;
};

class $PROPERTY extends $ {
  run() {
    if (!graph[this.object]) {
      throw TypeError();
    }

    if (this.name === "value") {
      throw new TypeError();
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

    let statement = new PROPERTY();
    statement.object = graph[this.object];
    statement.name = this.name;
    statement.value = this.value.run();
    return statement;
  }
}
