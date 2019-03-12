var graph = require("./graph");
var Token = require("./token");
var STATEMENT = require("./statement");

module.exports = function(string, offset) {
  let dependencies = [];

  let context = Token.each(string, offset, function(token) {
    if (token == "new") {
      return "new ";
    }

    let variable = token.split(".")[0];

    if (graph.node[token]) {
      dependencies.push(token);
    }

    if (graph.node[variable]) {
      return "state." + token;
    } else {
      return token;
    }
  });

  let statement = new EXPRESSION();
  statement.expression = context.tokens;
  statement.dependencies = dependencies;

  return { statement: statement, offset: context.offset };
};

class EXPRESSION extends STATEMENT {}
module.exports.EXPRESSION = EXPRESSION;
