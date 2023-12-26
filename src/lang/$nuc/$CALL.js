const $ = require("./$");
const graph = require("../../graph");
const $BLOCK = require("./$BLOCK");
const $LET = require("./$LET");
const _ = require("lodash");
const Identifier = require("../ast/Identifier");

const NULL = {
  type: "Literal",
  value: null,
  raw: "null",
};

function build(func, args) {
  const call = new $CALL();
  call.func = func;
  call.args = args;
  return call;
}

class $CALL extends $ {
  run() {
    let block, args;

    if (this.func.constructor.name === "$FUNCTION") {
      block = this.func.blk;
      args = this.func.args;
    } else {
      const name = new Identifier(this.func);
      const func = graph.retrieve(name);
      block = func.block;
      args = func.arguments;
    }

    if (block && args) {
      const values = this.args;

      const statements = _.cloneDeep(block.stms);

      for (let i = args.length - 1; i >= 0; i--) {
        statements.unshift($LET(args[i], values[i] || NULL));
      }

      return $BLOCK(statements);
    } else {
      throw TypeError(`${this.name} is not a function`);
    }
  }
}

module.exports = build;
