const $ = require("./$");
const graph = require("../../graph");
const IF = require("../../nuc/IF");
const CLASS = require("../../nuc/CLASS");
const IF$CLASS = require("../../nuc/IF$CLASS");
const Instruction = require("../../instruction");

function build(condition, trueB, p3) {
  let statement = new $IF();
  statement.condition = condition;
  statement.true = trueB; // truthy
  statement.false = p3; // falsy
  return statement;
}

class $IF extends $ {
  before() {
    this.key = `if(${this.condition.node})`;
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

    let statement = new IF(this.key);
    statement.condition = this.condition.run();
    statement.true = this.true;
    statement.false = this.false;

    return [
      new Instruction(scope, statement, true, true, false),
      new Instruction(scope, statement, false, false, true),
    ];
  }
}

module.exports = build;
