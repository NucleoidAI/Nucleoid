var BLOCK = require("./block");

class BLOCK$INSTANCE extends BLOCK {
  constructor() {
    super();
    this.statements = [];
  }

  run(scope) {
    scope.instance[this.class.name] = this.instance;
    let list = this.statements;
    this.statements = [];
    return list;
  }
}

BLOCK$INSTANCE.prototype.type = "INSTANCE";
module.exports = BLOCK$INSTANCE;
