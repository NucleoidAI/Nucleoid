const $BLOCK = require("./lang/$nuc/$block");
const Instruction = require("./instruction");
const $LET = require("./lang/$nuc/$let");
const state = require("./state");
const $EXP = require("./lang/$nuc/$expression");
const graph = require("./graph");

module.exports = class FOR {
  constructor() {
    this.index = 0;
  }

  run(scope) {
    let array = state.run(scope, `state.${this.array}`);

    if (this.index < array.length) {
      let list = [];
      let id = array[this.index].id;

      if (id !== undefined && graph[id]) {
        let object = array[this.index++].id;
        let context = $EXP(object);
        let statements = [$LET(this.variable, context.statement)];
        list.push($BLOCK(statements.concat(this.statements), true));
      } else {
        this.index++;
      }

      list.push(new Instruction(scope, this, false, true, false));
      return { next: list };
    }
  }
};
