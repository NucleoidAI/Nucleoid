import NODE from "./NODE.js";
import Instruction from "../Instruction.js";
import Scope from "../Scope.js";
import state from "../state.js";
import _ from "lodash";

class IF extends NODE {
  before(scope: Scope): void {
    this.condition.before(scope);
  }

  run(scope: Scope): { next: Instruction[] } {
    let local: Scope = new Scope(scope);
    let condition: boolean;

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

export default IF;
