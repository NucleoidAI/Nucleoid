var Token = require("../token");
var $LET = require("../$let");
var $EXP = require("../$expression");

module.exports = function ES6$LET(string, offset) {
  let context = Token.next(string, offset);
  context = Token.next(string, context.offset);
  let name = context.token;
  context = Token.next(string, context.offset);
  context = $EXP(string, context.offset);
  let expression = context.statement;
  return { statement: $LET(name, expression), offset: context.offset };
};
