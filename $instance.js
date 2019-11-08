var $ = require("./$");
var graph = require("./graph");
var OBJECT = require("./object");
var OBJECT$DECLARATION = require("./object$declaration");
var CLASS = require("./class");

module.exports = function(cls, name, object) {
  let statement = new $INSTANCE();
  statement.class = cls;
  statement.name = name;
  statement.object = object;
  return statement;
};

class $INSTANCE extends $ {
  run() {
    if (
      this.object &&
      (graph[this.object] instanceof CLASS ||
        graph[this.object] instanceof OBJECT$DECLARATION)
    ) {
      let statement = new OBJECT$DECLARATION();
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
