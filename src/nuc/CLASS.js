const state = require("../state");
const Node = require("./Node");
const graph = require("../graph");
const $ALIAS = require("../lang/$nuc/$ALIAS");
const $EXPRESSION = require("../lang/$nuc/$EXPRESSION");
const { deepEqual } = require("../lib/deep");
const Evaluation = require("../lang/ast/Evaluation");

class CLASS extends Node {
  constructor() {
    super();
    this.instances = {};
    this.declarations = {};
  }

  before() {
    this.key = this.name;
  }

  run(scope) {
    if (graph[this.name]) {
      if (
        deepEqual(this.args, graph[this.name].args) &&
        deepEqual(this.construct, graph[this.name].construct)
      ) {
        this.destroyed = true;
        return;
      }
    }

    state.assign(scope, this.name, new Evaluation(`class ${this.name}{}`));

    let list = [];

    if (!graph[this.name]) {
      state.call(scope, "classes.push", [`state.${this.name}`]);

      const alias = $ALIAS(this, this.list, $EXPRESSION("[]"));
      list.push(alias);
    }

    return { next: list };
  }

  beforeGraph() {
    if (this.destroyed) {
      return { destroyed: true };
    }

    if (graph[this.key] && graph[this.key] instanceof CLASS) {
      this.declarations = graph[this.key].declarations;
    }
  }
}

module.exports = CLASS;
