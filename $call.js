const $ = require("./$");
const graph = require("./graph");
const $BLOCK = require("./$block");

module.exports = function (name, params) {
  const call = new $CALL();
  call.name = name;
  call.params = params;
  return call;
};

class $CALL extends $ {
  run() {
    const func = graph[this.name];
    if (func) {
      const block = func.block;
      const params = this.params;
      const args = func.args;

      const object = {};

      for (let i = 0; i < args.length; i++) {
        object[args[i]] = params[i];
      }

      return $BLOCK(block.statements, false, object);
    }
  }
}
