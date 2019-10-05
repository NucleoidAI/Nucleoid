var Node = require("./node");

module.exports = class LET extends Node {
  run(scope) {
    scope.local[this.name] = this.value.run(scope);
  }
};
