const Token = require("../../token");
const $THROW = require("../$/$throw");

module.exports = function ES6$THROW(string, offset) {
  let context = Token.next(string, offset);
  context = Token.next(string, context.offset);

  let exception;
  let token = context.token;

  if (token.charCodeAt(0) === 34) {
    exception = token.substring(1, token.length - 1);
  } else {
    exception = token;
  }

  return { statement: $THROW(exception), offset: context.offset };
};
