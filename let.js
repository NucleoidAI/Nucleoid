var Node = require("./node");

module.exports = class LET extends Node {
  run(local) {
    local[this.variable] = this.expression.run(local);
  }
};
