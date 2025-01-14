import $ from "./$";
import graph from "../../graph";
import $BLOCK from "./$BLOCK";
import $LET from "./$LET";
import _ from "lodash";
import $Identifier from "../ast/Identifier";
import { Expression, Literal } from "acorn";
import { $FUNCTION } from "./$FUNCTION";

function build(func: Expression, args: Expression[]) {
  const call = new $CALL();
  call.func = func;
  call.args = args;
  return call;
}

class $CALL extends $ {
  func!: Expression | $FUNCTION;
  args!: Expression[];

  run() {
    let block, args;

    if (this.func.constructor.name === "$FUNCTION") {
      const func = this.func as $FUNCTION;
      block = func.blk;
      args = func.args;
    } else {
      const name = new $Identifier(this.func);
      const func = graph.retrieve(name);
      block = func.block;
      args = func.arguments;
    }

    if (block && args) {
      const values = this.args;

      const statements = _.cloneDeep(block.stms);

      for (let i = args.length - 1; i >= 0; i--) {
        statements.unshift(
          $LET(
            args[i],
            values[i] ||
              ({
                type: "Literal",
                value: null,
                raw: "null",
              } as Literal)
          )
        );
      }

      return $BLOCK(statements);
    } else {
      throw TypeError(`This is not a function`);
    }
  }
}

export default build;
export { $CALL };
