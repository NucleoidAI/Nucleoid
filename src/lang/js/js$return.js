const Token = require("../../utils/token");
const $RETURN = require("../$nuc/$return");
const JS = require("./js");

module.exports = function JS$RETURN(string, offset) {
  let context = Token.next(string, offset);

  if (context.token === "return") {
    context = Token.nextStatement(string, context.offset);

    const { statements } = JS.compile(context.statement);
    return { statement: $RETURN(statements), offset: context.offset };
  } else {
    throw SyntaxError("Missing return keyword");
  }
};
