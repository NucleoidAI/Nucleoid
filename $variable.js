var $ = require("./$");
var VARIABLE = require("./variable");
var EXPRESSION = require("./expression");

module.exports = function(name, value) {
  let statement = new $VARIABLE();
  statement.name = name;
  statement.value = value;
  return statement;
};

class $VARIABLE extends $ {
  run() {
    if (this.value instanceof EXPRESSION) {
      let statement = new VARIABLE();
      statement.name = this.name;
      statement.value = this.value;
      return statement;
    } else {
      let statement = this.value.run();
      statement.name = this.name;
      return statement;
    }
  }
  graph() {}
}
