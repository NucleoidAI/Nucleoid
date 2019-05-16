var Token = require("./token");
var $EXP = require("./$expression");
var LET = require("./let");

module.exports = function(string, offset) {
  let context = Token.next(string, offset);
  let variable;

  if (context.token == "let") {
    context = Token.next(string, context.offset);
    variable = context.token;

    context = Token.next(string, context.offset);
    context = $EXP(string, context.offset);

    let statement = new LET();
    statement.variable = variable;
    statement.expression = context.statement;

    return {
      statement: statement,
      offset: context.offset
    };
  }
};
