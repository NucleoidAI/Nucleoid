var $ = require("./$");
var graph = require("./graph");
var INSTANCE = require("./instance");

module.exports = function(cls, name, object) {
  let statement = new $INSTANCE();
  statement.class = cls;
  statement.name = name;
  statement.object = object;
  return statement;
};

class $INSTANCE extends $ {
  run() {
    let statement = new INSTANCE();
    statement.class = graph[this.class];
    statement.name = this.name;
    statement.object = graph[this.object];
    return statement;
  }
}
