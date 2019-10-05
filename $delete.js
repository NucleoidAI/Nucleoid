var $ = require("./$");
var DELETE = require("./delete");

module.exports = function(name) {
  let statement = new $DELETE();
  statement.name = name;
  return statement;
};

class $DELETE extends $ {
  run() {
    let statement = new DELETE();
    statement.name = this.name;
    return statement;
  }
}
