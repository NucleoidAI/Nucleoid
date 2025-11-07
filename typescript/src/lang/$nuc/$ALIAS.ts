import $EXP, { $EXPRESSION } from "./$EXPRESSION";

import $ from "./$";
import $Identifier from "../ast/$Identifier";
import ALIAS from "../../nuc/ALIAS";
import { Identifier } from "acorn";

function build(alias: Identifier, name: string, value: $EXPRESSION) {
  const statement = new $ALIAS();
  statement.als = alias;
  statement.nme = name;
  statement.val = value;
  return statement;
}

class $ALIAS extends $ {
  als!: Identifier;
  nme!: string;
  val!: any;

  before(scope) {
    const expression = $EXP(this.val);
    this.val = expression.run(scope);
  }

  run(scope: any): any {
    const name = new $Identifier(this.nme);
    const statement = new ALIAS();
    statement.alias = new $Identifier(this.als);
    statement.name = name;
    statement.value = this.val;
    return statement;
  }
}

export default build;
export { $ALIAS };
