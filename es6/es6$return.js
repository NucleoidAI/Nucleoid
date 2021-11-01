const Token = require("../token");
const $EXP = require("../$expression");
const $RETURN = require("../$return");

module.exports = function ES6$RETURN(string, offset) {
  let context = Token.next(string, offset);

  if (context.token === "return") {
    context = $EXP(string, context.offset);
    return { statement: $RETURN(context.statement), offset: context.offset };
  } else {
    throw SyntaxError("Missing return keyword");
  }
};
