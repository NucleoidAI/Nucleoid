var state = require("./state"); // eslint-disable-line no-unused-vars
var graph = require("./graph");
var Node = require("./node");
var Token = require("./token");
var $EXP = require("./expression");
var crypto = require("crypto");

module.exports = function(string, offset) {
  let context = Token.next(string, offset);

  if (context && context.token == "if") {
    context = Token.next(string, context.offset);
  }

  if (context && context.token == "(") {
    context = $EXP(string, context.offset);

    let statement = new IF();
    statement.condition = context.statement;

    context = Token.next(string, context.offset);
    context = Token.nextBlock(string, context.offset);
    statement.true = context.block;
    return { statement: statement, offset: context.offset };
  }
};

class IF {
  run() {
    let condition = this.condition;

    let shasum = crypto
      .createHash("sha1")
      .update("if(" + condition.tokens.join("") + ")")
      .digest("hex");

    graph.node[shasum] = new Node(this);

    for (let i = 0; i < condition.tokens.length; i++) {
      let token = condition.tokens[i];

      if (graph.node[token]) {
        graph.node[token].edge[shasum] = graph.node[shasum];
      }

      if (graph.node[token.split(".")[0]]) {
        condition.tokens[i] = "state." + token;
      }
    }

    return eval(condition.tokens.join(""));
  }
}
module.exports.IF = IF;
