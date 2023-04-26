const state = require("../state");
const Node = require("./Node");
let REFERENCE;

setImmediate(() => {
  REFERENCE = require("./REFERENCE");
});

class VARIABLE extends Node {
  before(scope) {
    this.key = this.name;
    this.value.before(scope, this.key);
  }

  run(scope) {
    let value;
    if (this.value instanceof REFERENCE) {
      value = state.assign(scope, this.name, this.value);
    } else {
      const run = this.value.run(scope);
      value = state.assign(scope, this.name, run, true);
    }

    return { value };
  }

  graph(scope) {
    return this.value.graph(scope);
  }
}

module.exports = VARIABLE;
