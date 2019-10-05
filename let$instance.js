var Node = require("./node");

module.exports = class LET$INSTANCE extends Node {
  run(scope) {
    let expression = this.value.run(scope, this.instance); // eslint-disable-line no-unused-vars
    eval("scope.local." + this.name + "=expression");
  }
};
