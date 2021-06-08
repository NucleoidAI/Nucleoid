const state = require("./state");
const Node = require("./node");

module.exports = class FUNCTION extends Node {
  constructor() {
    super();
    this.properties = {};
  }

  before() {
    this.key = this.name;
  }

  run(scope) {
    state.assign(scope, this.name, new Function(this.args, this.block));
  }
};
