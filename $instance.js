var $ = require("./$");
var graph = require("./graph");
var OBJECT = require("./object");

module.exports = function(cls, name, object) {
  let statement = new $INSTANCE();
  statement.class = cls;
  statement.name = name;
  statement.object = object;
  return statement;
};

class $INSTANCE extends $ {
  run() {
    let statement = new OBJECT();
    statement.class = graph[this.class];
    statement.name = this.name;
    statement.object = graph[this.object];
    return statement;
  }
}
