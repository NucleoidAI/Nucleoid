var $ = require("./$");
var graph = require("./graph");
var CLASS = require("./class");
var PROPERTY$DECLARATION = require("./property$declaration");
var PROPERTY = require("./property");
var OBJECT$DECLARATION = require("./object$declaration");

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

    if (
      graph[this.object] instanceof CLASS ||
      graph[this.object] instanceof OBJECT$DECLARATION
    ) {
      let statement = new PROPERTY$DECLARATION();
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
