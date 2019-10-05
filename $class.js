var CLASS = require("./class");
var $ = require("./$");

module.exports = function(name) {
  let statement = new $CLASS();
  statement.name = name;
  return statement;
};

class $CLASS extends $ {
  run() {
    let statement = new CLASS();
    statement.name = this.name;
    return statement;
  }
  graph() {}
}
