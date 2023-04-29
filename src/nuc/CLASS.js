const state = require("../state");
const Node = require("./Node");
const $EXP = require("../lang/$nuc/$EXPRESSION");
const graph = require("../graph");
const $ALIAS = require("../lang/$nuc/$ALIAS");

class CLASS extends Node {
  constructor() {
    super();
    this.instances = {};
    this.declarations = {};
    this.sequence = 1;
  }

  before() {
    this.key = this.name;
  }

  run(scope) {
    state.assign(scope, this.name, `class {}`);

    let list = [];

    if (!graph[this.name]) {
      let context = $EXP(`classes.push("${this.name}")`);
      list.push(context.statement);

      context = $EXP("[]");
      const alias = $ALIAS(this, this.name.substring(1), context.statement);
      list.push(alias);
    } else {
      this.sequence = graph[this.name].sequence;
    }

    return { next: list };
  }

  beforeGraph() {
    if (graph[this.key] && graph[this.key] instanceof CLASS) {
      this.declarations = graph[this.key].declarations;
    }
  }
}

CLASS.prototype.instanceof = "CLASS";
module.exports = CLASS;
