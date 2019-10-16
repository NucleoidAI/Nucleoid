var LET$INSTANCE = require("./let$instance");

class LET$CLASS {
  prepare() {}
  run(scope) {
    let instance = scope.retrieve(this.class.name);

    if (instance) {
      let statement = new LET$INSTANCE();
      statement.class = this.class;
      statement.instance = instance;
      statement.name = this.name;
      statement.object = scope.graph[this.object];
      statement.declaration = this;
      return statement;
    }
  }

  graph() {}
}

LET$CLASS.prototype.type = "CLASS";
module.exports = LET$CLASS;
