const $ = require("./$");
const graph = require("../../graph");
const IF = require("../../nuc/IF");
const CLASS = require("../../nuc/CLASS");
const IF$CLASS = require("../../nuc/IF$CLASS");
const Instruction = require("../../instruction");
const Expression = require("../ast/Expression");
const $EXPRESSION = require("./$EXPRESSION");

function build(condition, trueStatement, falseStatement) {
  let statement = new $IF();
  statement.condition = condition;
  statement.true = trueStatement; // truthy
  statement.false = falseStatement; // falsy
  return statement;
}

class $IF extends $ {
  before() {
    const condition = new Expression(this.condition);
    const expression = $EXPRESSION(condition);
    this.condition = expression.run();
  }

  run(scope) {
    /*
    for (let token of this.condition.node.list()) {
      let prefix = token.split(".")[0];

      if (graph[prefix] && graph[prefix] instanceof CLASS) {
        let statement = new IF$CLASS();
        statement.class = graph[prefix];
        statement.condition = this.condition.run();
        statement.true = this.true;
        statement.false = this.false;

        return [
          new Instruction(scope, statement, true, true, false),
          new Instruction(scope, statement, false, false, true),
        ];
      }
    }
    */

    let statement = new IF(`if(${this.condition.tokens})`);
    statement.condition = this.condition;
    statement.true = this.true;
    statement.false = this.false;

    return [
      new Instruction(scope, statement, true, true, false),
      new Instruction(scope, statement, false, false, true),
    ];
  }
}

module.exports = build;
