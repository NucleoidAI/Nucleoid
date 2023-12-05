const state = require("../state");
const Node = require("./NODE");
const graph = require("../graph");
const $ALIAS = require("../lang/$nuc/$ALIAS");
const Evaluation = require("../lang/Evaluation");
const _ = require("lodash");
const $EXPRESSION = require("../lang/$nuc/$EXPRESSION");

class CLASS extends Node {
  constructor(key) {
    super(key);
    this.methods = [];
    this.instances = {};
    this.declarations = {};
  }

  run(scope) {
    const cls = graph.retrieve(this.name);

    if (cls) {
      if (_.isEqual(this.methods, cls.methods)) {
        this.destroyed = true;
        return;
      }
    }

    state.assign(scope, this.name, new Evaluation(`class ${this.name}{}`));

    let list = [];

    if (!cls) {
      state.call(scope, "classes.push", [`state.${this.name}`]);

      const empty = { type: "ArrayExpression", elements: [] };
      const alias = $ALIAS(this.name.node, this.list.node, empty);
      list.push(alias);
    }

    list.push(
      $EXPRESSION({
        type: "Literal",
        value: null,
        raw: "null",
      })
    );

    return { next: list };
  }

  beforeGraph() {
    if (this.destroyed) {
      return { destroyed: true };
    }

    const cls = graph.retrieve(this.key);

    if (cls instanceof CLASS) {
      this.declarations = cls.declarations;
    }
  }
}

module.exports = CLASS;
