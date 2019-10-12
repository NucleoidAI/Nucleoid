var Token = require("./token");
var CLASS = require("./class");

module.exports = function(string, offset) {
  let context = Token.next(string, offset);
  let statement = new CLASS();

  if (context && context.token == "class") {
    context = Token.next(string, context.offset);

    statement.name = context.token;
    context = Token.next(string, context.offset);
    context = Token.nextBlock(string, context.offset);
  }

  return { statement: statement, offset: context.offset };
};
