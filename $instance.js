var $ = require("./$");
var graph = require("./graph");
var INSTANCE = require("./instance");

module.exports = function(cls, name, instance) {
  let statement = new $INSTANCE();
  statement.class = cls;
  statement.name = name;
  statement.instance = instance;
  return statement;
};

class $INSTANCE extends $ {
  run() {
    let statement = new INSTANCE();
    statement.class = graph.node[this.class];
    statement.name = this.name;
    statement.instance = graph.node[this.instance];
    return statement;
  }
}
