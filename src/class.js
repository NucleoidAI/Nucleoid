const state = require("./state");
const Node = require("./node");
const $VAR = require("./lang/$nuc/$variable");
const $EXP = require("./lang/$nuc/$expression");
const graph = require("./graph");

class CLASS extends Node {
  constructor() {
    super();
    this.instances = {};
    this.declarations = {};
    this.sequence = 0;
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
      list.push($VAR(this.name.substring(1), context.statement));
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
