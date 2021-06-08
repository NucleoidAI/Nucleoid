var LET$INSTANCE = require("./let$instance");
var Instance = require("./instance");

class LET$CLASS {
  before() {}

  run(scope) {
    let instance = Instance.retrieve(scope, this.class.name);

    if (instance) {
      let statement = new LET$INSTANCE();
      statement.class = this.class;
      statement.instance = instance;
      statement.name = this.name;
      statement.declaration = this;
      return statement;
    }
  }

  graph() {}
}

LET$CLASS.prototype.type = "CLASS";
module.exports = LET$CLASS;
