var Token = require("./token");
var CLASS = require("./class");

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

module.exports.$CLASS = class $CLASS {};
