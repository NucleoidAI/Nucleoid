var $ = require("./$");
var graph = require("./graph");
var CLASS = require("./class");
var LET = require("./let");
var LET$CLASS = require("./let$class");

module.exports = function(object, name, value) {
  let statement = new $LET();
  statement.name = name;
  statement.object = object;
  statement.value = value;
  return statement;
};

class $LET extends $ {
  run(scope) {
    for (let token of this.value.tokens) {
      let prefix = token.split(".")[0];

      if (graph[prefix] && graph[prefix] instanceof CLASS) {
        let statement = new LET$CLASS();
        statement.class = graph[prefix];
        statement.name = this.name;
        statement.object = this.object;
        statement.value = this.value.run();
        return statement;
      }
    }

    let statement = new LET();
    statement.name = this.name;
    statement.object = scope.graph[this.object];
    statement.value = this.value.run();
    return statement;
  }
}
