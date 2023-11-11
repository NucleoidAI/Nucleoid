const $ = require("./$");
const graph = require("../../graph");
const IF = require("../../nuc/IF");
const CLASS = require("../../nuc/CLASS");
const IF$CLASS = require("../../nuc/IF$CLASS");
const Instruction = require("../../instruction");
const Expression = require("../ast/Expression");
const $EXPRESSION = require("./$EXPRESSION");
const Identifier = require("../ast/Identifier");

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
    this.condition.tokens.traverse((node) => {
      if (node instanceof Identifier) {
        const cls = graph.retrieve(node.first);

        if (cls instanceof CLASS) {
          let statement = new IF$CLASS(`if(${this.condition.tokens})`);
          statement.class = cls;
          statement.condition = this.condition;
          statement.false = this.false;

          if (this.true) {
            statement.true = this.true;
            statement.true.class = cls;
          }

          if (this.false) {
            statement.false = this.false;
            statement.false.class = cls;
          }

          return [
            new Instruction(scope, statement, true, true, false),
            new Instruction(scope, statement, false, false, true),
          ];
        }
      }
    });

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
