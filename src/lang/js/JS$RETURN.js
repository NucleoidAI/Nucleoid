const Token = require("../../lib/token");
const $RETURN = require("../$nuc/$RETURN");
const JS = require("./JS");

function JS$RETURN(string, offset) {
  let context = Token.next(string, offset);

  if (context.token === "return") {
    context = Token.nextStatement(string, context.offset);

    const { statements } = JS.compile(context.statement);
    return { statement: $RETURN(statements), offset: context.offset };
  } else {
    throw SyntaxError("Missing return keyword");
  }
}

module.exports = JS$RETURN;
