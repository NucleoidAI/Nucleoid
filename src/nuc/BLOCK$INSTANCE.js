const BLOCK = require("./BLOCK");

class BLOCK$INSTANCE extends BLOCK {
  constructor() {
    super();
    this.statements = [];
  }

  run(scope) {
    scope.instances[this.class.name] = this.instance;

    if (this.break) {
      this.statements = this.declaration.statements;
      this.break = false;
    }

    return super.run(scope);
  }
}

BLOCK$INSTANCE.prototype.type = "INSTANCE";
module.exports = BLOCK$INSTANCE;
