const BLOCK = require("./BLOCK");

class BLOCK$INSTANCE extends BLOCK {
  before(scope) {
    this.statements.forEach((statement) => {
      statement.class = this.class;
      statement.instance = this.instance;
    });

    super.before(scope);
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
