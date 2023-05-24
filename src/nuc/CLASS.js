const state = require("../state");
const Node = require("./Node");
const $EXP = require("../lang/$nuc/$EXPRESSION");
const graph = require("../graph");
const $ALIAS = require("../lang/$nuc/$ALIAS");
const { deepEqual } = require("../lib/deep");

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

    state.assign(scope, this.name, `class {}`);

    let list = [];

    if (!graph[this.name]) {
      let context = $EXP(`classes.push("${this.name}")`);
      list.push(context.statement);

      context = $EXP("[]");
      const alias = $ALIAS(this, this.name.substring(1), context.statement);
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
