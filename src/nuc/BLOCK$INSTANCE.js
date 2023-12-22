const BLOCK = require("./BLOCK");

class BLOCK$INSTANCE extends BLOCK {
  before(scope) {
    super.before(scope);
  }

  run(scope) {
    scope.$instance = this.instance;

    if (this.break) {
      this.statements = this.declaration.statements;
      this.break = false;
    }

    return super.run(scope);
  }
}

BLOCK$INSTANCE.prototype.type = "INSTANCE";
module.exports = BLOCK$INSTANCE;
