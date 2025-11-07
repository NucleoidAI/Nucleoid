import { Expression, Literal } from "acorn";

import $ from "./$";
import $BLOCK from "./$BLOCK";
import { $FUNCTION } from "./$FUNCTION";
import $Identifier from "../ast/Identifier";
import $LET from "./$LET";
import NODE from "../../nuc/NODE";
import _ from "lodash";
import { retrieve } from "../../graph";

function build(func: Expression, args: Expression[]) {
  const call = new $CALL();
  call.func = func;
  call.args = args;
  return call;
}

class $CALL extends $ {
  func!: Expression | $FUNCTION;
  args!: Expression[];

  result: any;

  run(scope: any): $ | NODE | NODE[] | null {
    let block, args;

    if (this.func.constructor.name === "$FUNCTION") {
      const func = this.func as $FUNCTION;
      block = func.blk;
      args = func.args;
    } else {
      const name = new $Identifier(this.func);
      const func = retrieve(name);
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

      const blockInstance = $BLOCK(statements);
      this.result = blockInstance.run(scope);
      return this;
    } else {
      throw TypeError(`This is not a function`);
    }
  }
}

export default build;
export { $CALL };

