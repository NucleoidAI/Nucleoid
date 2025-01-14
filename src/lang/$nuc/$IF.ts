import $ from "./$";
import graph from "../../graph";
import IF from "../../nuc/IF";
import CLASS from "../../nuc/CLASS";
import IF$CLASS from "../../nuc/IF$CLASS";
import Instruction from "../../Instruction";
import $Expression from "../ast/$Expression";
import $EXPRESSION from "./$EXPRESSION";
import $Identifier from "../ast/$Identifier";
import { Expression } from "acorn";
import { $BLOCK } from "./$BLOCK";
import NODE from "../../nuc/NODE";
import EXPRESSION from "../../nuc/EXPRESSION";

function build(
  condition: Expression,
  trueStatement: $BLOCK,
  falseStatement: $BLOCK
) {
  const statement = new $IF();
  statement.con = condition;
  statement.tru = trueStatement; // truthy
  statement.fls = falseStatement; // falsy
  return statement;
}

class $IF extends $ {
  con!: Expression | NODE;
  tru!: $BLOCK;
  fls!: $BLOCK;

  before(scope) {
    const condition = new $Expression(this.con);
    const expression = $EXPRESSION(condition);
    this.con = expression.run(scope);
  }

  run(scope) {
    const con = this.con as EXPRESSION;
    // Look up first expression for deciding class declaration
    const declarations = con.graph(scope);

    if (declarations?.length) {
      const [declaration] = declarations;
      const identifier = new $Identifier(declaration.key);
      const cls = graph.retrieve(identifier.first);

      if (cls instanceof CLASS) {
        const statement = new IF$CLASS(`if(${con.tokens})`);
        statement.class = cls;
        statement.condition = con;
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

    const statement = new IF(`if(${con.tokens})`);
    statement.condition = con;
    statement.true = this.tru;
    statement.false = this.fls;

    return [
      new Instruction(scope, statement, true, true, false, false),
      new Instruction(scope, statement, false, false, true, true),
    ];
  }
}

export default build;
export { $IF };
