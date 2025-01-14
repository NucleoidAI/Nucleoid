import $ from "./$";
import FUNCTION from "../../nuc/FUNCTION";
import Identifier from "../ast/Identifier";
import { $BLOCK } from "./$BLOCK";
import { Pattern } from "acorn";

function build(name: Identifier, args: Pattern[], block: $BLOCK) {
  const statement = new $FUNCTION();
  statement.nme = name;
  statement.args = args;
  statement.blk = block;
  return statement;
}

class $FUNCTION extends $ {
  nme!: Identifier;
  args!: Pattern[];
  blk!: $BLOCK;

  run() {
    const name = new Identifier(this.nme);
    const statement = new FUNCTION(name);
    statement.name = name;
    statement.arguments = this.args;
    statement.block = this.blk;
    return statement;
  }
}

export default build;
export { $FUNCTION };
