const $ = require("./$");
const graph = require("../../graph");
const $BLOCK = require("./$BLOCK");
const $LET = require("./$LET");
const $EXP = require("./$EXPRESSION");

function construct(name, params) {
  const call = new $CALL();
  call.name = name;
  call.params = params;
  return call;
}

class $CALL extends $ {
  run() {
    const fn = graph[this.name];

    if (fn) {
      const block = fn.block;
      const params = this.params;
      const args = fn.args;

      for (let i = args.length - 1; i >= 0; i--) {
        block.statements.unshift($LET(args[i], $EXP(params[i]).statement));
      }

      return $BLOCK(block.statements, false);
    }
  }
}

module.exports = construct;
