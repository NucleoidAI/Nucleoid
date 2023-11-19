const LET$INSTANCE = require("./LET$INSTANCE");

class LET$CLASS {
  before() {}

  run(scope) {
    let instance = scope.instance(this.class.name);

    if (instance) {
      let statement = new LET$INSTANCE();
      statement.class = this.class;
      statement.instance = instance;
      statement.name = this.name;
      statement.declaration = this;
      return { next: statement };
    }
  }
}

LET$CLASS.prototype.type = "CLASS";
module.exports = LET$CLASS;
