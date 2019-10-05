var $ = require("./$");
var graph = require("./graph");
var INSTANCE = require("./instance");

module.exports = function(cls) {
  let statement = new $INSTANCE();
  statement.class = cls;
  return statement;
};

class $INSTANCE extends $ {
  run() {
    let statement = new INSTANCE();
    statement.class = graph.node[this.class];
    return statement;
  }
}
