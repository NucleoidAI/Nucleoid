var $ = require("./$");
var DELETE = require("./delete");
var VARIABLE = require("./variable");
var graph = require("./graph");
var DELETE$VARIABLE = require("./delete$variable");
var DELETE$OBJECT = require("./delete$object");
var OBJECT = require("./object");

module.exports = function(key) {
  let statement = new $DELETE();
  statement.key = key;
  return statement;
};

class $DELETE extends $ {
  run() {
    if (graph[this.key] instanceof VARIABLE) {
      let statement = new DELETE$VARIABLE();
      statement.key = this.key;
      return statement;
    } else if (graph[this.key] instanceof OBJECT) {
      let statement = new DELETE$OBJECT();
      statement.key = this.key;
      return statement;
    } else {
      let statement = new DELETE();
      statement.key = this.key;
      return statement;
    }
  }
}
