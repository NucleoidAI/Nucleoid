var $ = require("./$");
var VARIABLE = require("./variable");
var Value = require("./value");

module.exports = function(name, value) {
  let statement = new $VARIABLE();
  statement.name = name;
  statement.value = value;
  return statement;
};

class $VARIABLE extends $ {
  run() {
    let value = this.value.run();

    if (value instanceof Value) {
      let statement = new VARIABLE();
      statement.name = this.name;
      statement.value = value;
      return statement;
    } else {
      let statement = value;
      statement.name = this.name;
      return statement;
    }
  }
}
