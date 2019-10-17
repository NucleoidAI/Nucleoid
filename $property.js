var $ = require("./$");
var graph = require("./graph");
var CLASS = require("./class");
var PROPERTY$CLASS = require("./property$class");
var PROPERTY = require("./property");

module.exports = function(object, name, value) {
  let statement = new $PROPERTY();
  statement.object = object;
  statement.name = name;
  statement.value = value;
  return statement;
};

class $PROPERTY extends $ {
  run() {
    if (graph[this.object] instanceof CLASS) {
      let statement = new PROPERTY$CLASS();
      statement.class = graph[this.object];
      statement.object = this.object;
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
