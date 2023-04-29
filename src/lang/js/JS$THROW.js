const Token = require("../../lib/token");
const $THROW = require("../$nuc/$THROW");

function JS$THROW(string, offset) {
  let context = Token.next(string, offset);
  context = Token.next(string, context.offset);

  const exception = context.token;

  return { statement: $THROW(exception), offset: context.offset };
}

module.exports = JS$THROW;
