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
    if (graph.node[this.instance] instanceof CLASS) {
      let statement = new PROPERTY$CLASS();
      statement.class = graph.node[this.instance];
      statement.name = this.name;
      statement.value = this.value.run();
      return statement;
    }

    let statement = new PROPERTY();
    statement.instance = graph.node[this.instance];
    statement.name = this.name;
    statement.value = this.value.run();
    return statement;
  }
}
