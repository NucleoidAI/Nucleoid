var graph = require("./graph");
var Token = require("./token");
var STATEMENT = require("./statement");

module.exports = function(string, offset) {
  let context = Token.next(string, offset);

  if (context && context.token == "if") {
    context = Token.next(string, context.offset);
  }

  let dependencies = [];

  context = Token.each(
    string,
    context.offset,
    function(token) {
      if (graph.node[token]) {
        dependencies.push(token);
        return "state." + token;
      } else {
        return token;
      }
    },
    ")"
  );

  let statement = new IF();
  statement.expression = context.tokens;
  statement.dependencies = dependencies;

  context = Token.next(string, context.offset);
  context = Token.nextBlock(string, context.offset);
  statement.true = context.block;
  return { statement: statement, offset: context.offset };
};

class IF extends STATEMENT {}
module.exports.IF = IF;
