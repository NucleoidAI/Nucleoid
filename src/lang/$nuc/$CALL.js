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
  call.function = func;
  call.arguments = args;
  return call;
}

class $CALL extends $ {
  run() {
    let func;

    if (this.function.constructor.name === "$FUNCTION") {
      func = this.function;
    } else {
      const name = new Identifier(this.function);
      func = graph.retrieve(name);
    }

    if (func) {
      const block = func.block;
      const args = func.arguments;
      const values = this.arguments;

      const statements = _.cloneDeep(block.statements);

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
