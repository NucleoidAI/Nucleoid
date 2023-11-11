const $ = require("./$");
const graph = require("../../graph");
const $BLOCK = require("./$BLOCK");
const $LET = require("./$LET");
const $EXP = require("./$EXPRESSION");
const $FUNCTION = require("./$FUNCTION");
const _ = require("lodash");
const Identifier = require("../ast/Identifier");

function build(name, args) {
  const call = new $CALL();
  call.name = name;
  call.arguments = args;
  return call;
}

class $CALL extends $ {
  run() {
    const name = new Identifier(this.name);
    const fn = graph.retrieve(name);

    if (fn) {
      const block = fn.block;
      const args = fn.arguments;
      const values = this.arguments;

      const statements = _.cloneDeep(block.statements);

      for (let i = args.length - 1; i >= 0; i--) {
        statements.unshift($LET(args[i], values[i]));
      }

      return $BLOCK(statements);
    } else {
      throw TypeError(`${this.name} is not a function`);
    }
  }
}

module.exports = build;
