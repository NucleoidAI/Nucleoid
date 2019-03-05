var graph = require("./graph");
var Token = require("./token");
var STATEMENT = require("./statement");

module.exports = function(string, offset) {
  let context = Token.next(string, offset);

  if (Token.check(string, context.offset) == "=") {
    let variable = context.token;
    let dependencies = [];

    context = Token.each(string, offset, function(token) {
      if (graph.node[token]) {
        dependencies.push(token);
        return "state." + token;
      } else {
        return token;
      }
    });

    let statement = new ASSIGNMENT();
    statement.variable = variable;
    statement.assignment = context.tokens;
    statement.dependencies = dependencies;
    return { statement: statement, offset: context.offset };
  }
};

class ASSIGNMENT extends STATEMENT {}
module.exports.ASSIGNMENT = ASSIGNMENT;
