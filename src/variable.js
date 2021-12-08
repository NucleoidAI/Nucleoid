const state = require("./state");
const Node = require("./node");
let REFERENCE;

setImmediate(() => {
  REFERENCE = require("./reference");
});

module.exports = class VARIABLE extends Node {
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
      value = state.assign(scope, this.name, run);
    }

    return { value };
  }

  graph(scope) {
    return this.value.graph(scope);
  }
};
