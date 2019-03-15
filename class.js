var state = require("./state"); // eslint-disable-line no-unused-vars
var graph = require("./graph");
var Node = require("./node");
var Token = require("./token");

module.exports = function(string, offset) {
  let context = Token.next(string, offset);
  let statement = new CLASS();

  if (context && context.token == "class") {
    context = Token.next(string, context.offset);

    statement.class = context.token;
    statement.definition = "state." + statement.class + "=" + "class";

    context = Token.next(string, context.offset);
    context = Token.nextBlock(string, context.offset);
    statement.definition += "{" + context.block + "}";
  }

  return { statement: statement, offset: context.offset };
};

class CLASS {
  run() {
    let definition = this.definition;

    graph.node[this.class] = new Node(this);
    eval(definition);
  }
}
module.exports.CLASS = CLASS;
