var $ = require("./$");
var graph = require("./graph");
var CLASS = require("./class");
var LET = require("./let");
var LET$CLASS = require("./let$class");

module.exports = function(name, value) {
  let statement = new $LET();
  statement.name = name;
  statement.value = value;
  return statement;
};

class $LET extends $ {
  run() {
    for (let token of this.value.tokens) {
      let prefix = token.split(".")[0];

      if (graph[prefix] && graph[prefix] instanceof CLASS) {
        let statement = new LET$CLASS();
        statement.class = graph[prefix];
        statement.name = this.name;
        statement.value = this.value.run();
        return statement;
      }
    }

    let statement = new LET();
    statement.name = this.name;
    statement.value = this.value.run();
    return statement;
  }
}
