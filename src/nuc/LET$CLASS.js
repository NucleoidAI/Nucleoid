const LET$INSTANCE = require("./LET$INSTANCE");
const _ = require("lodash");

class LET$CLASS {
  before() {}

  run(scope) {
    const instance = scope.$instance;

    if (instance) {
      let statement = new LET$INSTANCE();
      statement.class = this.class;
      statement.instance = instance;
      statement.name = this.name;
      statement.value = _.cloneDeep(this.value);
      return { next: statement };
    }
  }
}

LET$CLASS.prototype.type = "CLASS";
module.exports = LET$CLASS;
