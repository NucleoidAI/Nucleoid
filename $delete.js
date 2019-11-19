var $ = require("./$");
var DELETE = require("./delete");

module.exports = function(key) {
  let statement = new $DELETE();
  statement.key = key;
  return statement;
};

class $DELETE extends $ {
  run() {
    let statement = new DELETE();
    statement.key = this.key;
    return statement;
  }
}
