const state = require("../state");
const Node = require("./NODE");
const graph = require("../graph");
const $ALIAS = require("../lang/$nuc/$ALIAS");
const { deepEqual } = require("../lib/deep");
const Evaluation = require("../lang/ast/Evaluation");

class CLASS extends Node {
  constructor(key) {
    super(key);
    this.arguments = [];
    this.instances = {};
    this.declarations = {};
  }

  run(scope) {
    if (graph.retrieve(this.name)) {
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

      const empty = { type: "ArrayExpression", elements: [] };
      const alias = $ALIAS(this.name.node, this.list.node, empty);
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
