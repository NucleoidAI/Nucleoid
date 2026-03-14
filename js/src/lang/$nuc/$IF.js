const $ = require("./$");
const graph = require("../../graph");
const IF = require("../../nuc/IF");
const CLASS = require("../../nuc/CLASS");
const IF$CLASS = require("../../nuc/IF$CLASS");
const Instruction = require("../../Instruction");
const Expression = require("../../Expression");
const $EXPRESSION = require("./$EXPRESSION");
const Identifier = require("../ast/Identifier");

function build(condition, trueStatement, falseStatement) {
  let statement = new $IF();
  statement.con = condition;
  statement.tru = trueStatement; // truthy
  statement.fls = falseStatement; // falsy
  return statement;
}

class $IF extends $ {
  before(scope) {
    const condition = new Expression(this.con);
    const expression = $EXPRESSION(condition);
    this.con = expression.run(scope);
  }

  run(scope) {
    // Look up first expression for deciding class declaration
    const [declaration] = this.con.graph(scope);

    if (declaration) {
      const identifier = new Identifier(declaration.key);
      const cls = graph.retrieve(identifier.first);

      if (cls instanceof CLASS) {
        let statement = new IF$CLASS(`if(${this.con.tokens})`);
        statement.class = cls;
        statement.condition = this.con;
        statement.true = this.tru;

        if (this.fls) {
          statement.false = this.fls;
        }

        return [
          new Instruction(scope, statement, true, true, false, false),
          new Instruction(scope, statement, false, false, true, true),
        ];
      }
    }

    let statement = new IF(`if(${this.con.tokens})`);
    statement.condition = this.con;
    statement.true = this.tru;
    statement.false = this.fls;

    return [
      new Instruction(scope, statement, true, true, false, false),
      new Instruction(scope, statement, false, false, true, true),
    ];
  }
}

module.exports = build;
