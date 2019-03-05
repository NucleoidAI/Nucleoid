var graph = require("./graph");
var Token = require("./token");
var STATEMENT = require("./statement");

module.exports = function(string, offset) {
  let context = Token.each(string, offset, function(token) {
    if (graph.node[token]) {
      return "state." + token;
    } else {
      return token;
    }
  });

  let statement = new EXPRESSION();
  statement.expression = context.tokens;

  return { statement: statement, offset: context.offset };
};

class EXPRESSION extends STATEMENT {}
module.exports.EXPRESSION = EXPRESSION;
