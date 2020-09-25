var $BLOCK = require("./$block");
var Instruction = require("./instruction");
var $LET = require("./$let");
var state = require("./state");
var $EXP = require("./$expression");

module.exports = class FOR {
  constructor() {
    this.index = 0;
  }

  run(scope) {
    let array = state.run(scope, `state.${this.array}`);

    if (this.index < array.length) {
      let object = array[this.index++].id;
      let context = $EXP(object, 0);

      let statements = [$LET(this.variable, context.statement)];
      let list = [$BLOCK(statements.concat(this.statements))];
      list.push(new Instruction(scope, this, false, true, false));
      return list;
    }
  }
};
