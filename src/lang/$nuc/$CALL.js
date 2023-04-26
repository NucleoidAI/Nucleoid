const $ = require("./$");
const graph = require("../../graph");
const $BLOCK = require("./$BLOCK");

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

      const object = {};

      for (let i = 0; i < args.length; i++) {
        object[args[i]] = params[i];
      }

      return $BLOCK(block.statements, false, object);
    }
  }
}

module.exports = construct;
