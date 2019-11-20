var state = require("./state"); // eslint-disable-line no-unused-vars
var LET = require("./let");

module.exports = class LET$OBJECT extends LET {
  prepare() {}

  // eslint-disable-next-line no-unused-vars
  run(scope) {
    eval("scope.local." + this.name + "= new state." + this.class.name + "()");
  }

  // eslint-disable-next-line no-unused-vars
  graph(scope) {
    eval("scope.graph." + this.name + "=this");
  }
};
