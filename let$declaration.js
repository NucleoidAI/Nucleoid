var LET$PROTOTYPE = require("./let$prototype");

class LET$DECLARATION {
  prepare() {}
  run(scope) {
    let instance = scope.retrieve(this.class.name);

    if (instance) {
      let statement = new LET$PROTOTYPE();
      statement.class = this.class;
      statement.instance = instance;
      statement.name = this.name;
      statement.declaration = this;
      return statement;
    }
  }

  graph() {}
}

LET$DECLARATION.prototype.type = "DECLARATION";
module.exports = LET$DECLARATION;
