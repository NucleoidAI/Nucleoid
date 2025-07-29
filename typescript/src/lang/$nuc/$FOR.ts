import $ from "./$";
import FOR from "../../nuc/FOR";
import Instruction from "../../Instruction";
import Identifier from "../ast/Identifier";
import { Expression, Pattern } from "acorn";

function build(variable: Pattern, array: Expression, statements) {
  const statement = new $FOR();
  statement.var = variable;
  statement.arr = array;
  statement.stms = statements;
  return statement;
}

class $FOR extends $ {
  var!: Pattern;
  arr!: Expression;
  stms!: $[];

  run(scope) {
    const statement = new FOR();
    statement.variable = new Identifier(this.var);
    statement.array = new Identifier(this.arr);
    statement.statements = this.stms;

    return new Instruction(scope, statement, false, true, false, false);
  }
}

export default build;
export { $FOR };
