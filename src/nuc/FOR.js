const $BLOCK = require("../lang/$nuc/$BLOCK");
const Instruction = require("../instruction");
const $LET = require("../lang/$nuc/$LET");
const state = require("../state");
const $EXP = require("../lang/$nuc/$EXPRESSION");
const graph = require("../graph");
const Evaluation = require("../lang/Evaluation");
const _ = require("lodash");

class FOR {
  constructor() {
    this.index = 0;
  }

  run(scope) {
    const array = state.expression(
      scope,
      new Evaluation(`state.${this.array}`)
    );

    if (!Array.isArray(array)) {
      throw new TypeError(`${this.array} is not iterable`);
    }

    if (this.index < array.length) {
      let list = [];
      let key = array[this.index].id;

      if (key !== undefined && graph.graph[key]) {
        let object = array[this.index++].id;
        let statements = [$LET(this.variable.node, object)];
        list.push(
          $BLOCK(statements.concat(_.cloneDeep(this.statements)), true)
        );
      } else {
        this.index++;
      }

      list.push(new Instruction(scope, this, false, true, false));
      return { next: list };
    }
  }
}

module.exports = FOR;
