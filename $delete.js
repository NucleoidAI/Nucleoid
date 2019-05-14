var Token = require("./token");
var DELETE = require("./delete");

module.exports = function(string, offset) {
  let context = Token.next(string, offset);
  let statement = new DELETE();

  if (context && context.token == "delete") {
    context = Token.next(string, context.offset);
    statement.variable = context.token;
  }

  return { statement: statement, offset: context.offset };
};
