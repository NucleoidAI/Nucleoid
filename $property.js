var $ = require("./$");
var graph = require("./graph");
var CLASS = require("./class");
var PROPERTY$CLASS = require("./property$class");
var PROPERTY = require("./property");

module.exports = function(instance, name, value) {
  let statement = new $PROPERTY();
  statement.instance = instance;
  statement.name = name;
  statement.value = value;
  return statement;
};

class $PROPERTY extends $ {
  run() {
    if (graph[this.instance] instanceof CLASS) {
      let statement = new PROPERTY$CLASS();
      statement.class = graph[this.instance];
      statement.name = this.name;
      statement.value = this.value.run();
      return statement;
    }

    if (!graph[this.instance]) {
      throw TypeError();
    }

    let statement = new PROPERTY();
    statement.instance = graph[this.instance];
    statement.name = this.name;
    statement.value = this.value.run();
    return statement;
  }
}
