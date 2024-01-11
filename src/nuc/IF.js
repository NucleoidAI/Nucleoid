const NODE = require("./NODE");
const Instruction = require("../Instruction");
const Scope = require("../Scope");
const state = require("../state");
const _ = require("lodash");

class IF extends NODE {
  before(scope) {
    this.condition.before(scope);
  }

  run(scope) {
    let local = new Scope(scope);
    let condition;

    if (scope.block?.skip) {
      condition = this.condition.run(scope, true);
    } else {
      condition = this.condition.run(scope, true);
    }

    if (state.expression(scope, condition)) {
      const trueStatement = _.cloneDeep(this.true);
      return {
        next: [
          new Instruction(local, trueStatement, true, true, false, false),
          new Instruction(local, trueStatement, false, false, true, true),
        ],
      };
    } else if (this.false) {
      const falseStatement = _.cloneDeep(this.false);
      return {
        next: [
          new Instruction(local, falseStatement, true, true, false, false),
          new Instruction(local, falseStatement, false, false, true, true),
        ],
      };
    }
  }

  graph(scope) {
    return this.condition.graph(scope);
  }
}

module.exports = IF;
