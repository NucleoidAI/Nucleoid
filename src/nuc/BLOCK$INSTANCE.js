const BLOCK = require("./BLOCK");

class BLOCK$INSTANCE extends BLOCK {
  before(scope) {
    this.statements.forEach((statement) => {
      statement.class = this.class;
      statement.instance = this.instance;
    });

    super.before(scope);
  }
}

BLOCK$INSTANCE.prototype.type = "INSTANCE";
module.exports = BLOCK$INSTANCE;
