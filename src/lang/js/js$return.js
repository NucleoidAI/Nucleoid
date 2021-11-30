const Token = require("../../token");
const $RETURN = require("../$/$return");
const JS = require("./js");

module.exports = function JS$RETURN(string, offset) {
  let context = Token.next(string, offset);

  if (context.token === "return") {
    context = JS.compile(string, context.offset);
    return { statement: $RETURN(context.statements), offset: context.offset };
  } else {
    throw SyntaxError("Missing return keyword");
  }
};
