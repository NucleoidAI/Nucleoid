import $ from "./$";
import VARIABLE from "../../nuc/VARIABLE";
import $EXPRESSION from "./$EXPRESSION";
import $Identifier from "../ast/$Identifier";
import { Expression, Identifier, Node, Pattern } from "acorn";
import NODE from "../../nuc/NODE";

function build(name: Pattern, value: Expression) {
  const statement = new $VARIABLE();
  statement.nme = name;
  statement.val = value;
  return statement;
}

class $VARIABLE extends $ {
  nme!: Pattern;
  val!: Expression | NODE;

  before(scope): void {
    const expression = $EXPRESSION(this.val);
    this.val = expression.run(scope);
  }

  run(): VARIABLE {
    const name = new $Identifier(this.nme);
    const statement = new VARIABLE(name);
    statement.name = name;
    statement.value = this.val;
    return statement;
  }
}

export default build;
export { $VARIABLE };
