var $ = require("./$");
var graph = require("./graph");
var IF = require("./if");
var CLASS = require("./class");
var IF$CLASS = require("./if$class");
var Instruction = require("./instruction");

module.exports = function(condition, trueB, p3) {
  let statement = new $IF();
  statement.condition = condition;
  statement.true = trueB;
  statement.false = p3;
  return statement;
};

class $IF extends $ {
  run(scope) {
    for (let token of this.condition.tokens) {
      let prefix = token.split(".")[0];

      if (graph[prefix] && graph[prefix] instanceof CLASS) {
        let statement = new IF$CLASS();
        statement.class = graph[prefix];
        statement.condition = this.condition.run();
        statement.true = this.true;
        return [
          new Instruction(scope, statement, true, true, false),
          new Instruction(scope, statement, false, false, true)
        ];
      }
    }

    let statement = new IF();
    statement.condition = this.condition.run();
    statement.true = this.true;
    statement.false = this.false;
    return [
      new Instruction(scope, statement, true, true, false),
      new Instruction(scope, statement, false, false, true)
    ];
  }
}
