var Node = require("./node");

module.exports = class LET$INSTANCE extends Node {
  run(local) {
    local[this.variable] = this.expression.run(local, this.instance);
  }
};
