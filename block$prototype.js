var BLOCK = require("./block");

class BLOCK$PROTOTYPE extends BLOCK {
  constructor() {
    super();
    this.statements = [];
  }

  run(scope) {
    scope.instance[this.class.name] = this.instance;
    return super.run(scope);
  }
}

BLOCK$PROTOTYPE.prototype.type = "PROTOTYPE";
module.exports = BLOCK$PROTOTYPE;
