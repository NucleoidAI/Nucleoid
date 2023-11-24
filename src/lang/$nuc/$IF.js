const $ = require("./$");
const graph = require("../../graph");
const IF = require("../../nuc/IF");
const CLASS = require("../../nuc/CLASS");
const IF$CLASS = require("../../nuc/IF$CLASS");
const Instruction = require("../../instruction");
const Expression = require("../../Expression");
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
    // Look up first expression for deciding class declaration
    const [declaration] = this.condition.graph(scope);

    if (declaration) {
      const identifier = new Identifier(declaration.key);
      const cls = graph.retrieve(identifier.first);

      if (cls instanceof CLASS) {
        let statement = new IF$CLASS(`if(${this.condition.tokens})`);
        statement.class = cls;
        statement.condition = this.condition;
        statement.false = this.false;
        statement.true = this.true;
        statement.true.class = declaration;

        if (this.false) {
          statement.false = this.false;
          statement.false.class = declaration;
        }

        return [new Instruction(scope, statement, true, true, true)];
      }
    }

    let statement = new IF(`if(${this.condition.tokens})`);
    statement.condition = this.condition;
    statement.true = this.true;
    statement.false = this.false;

    return [new Instruction(scope, statement, true, true, true)];
  }
}

module.exports = build;
