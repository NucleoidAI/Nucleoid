var Token = require("../token");
var $THROW = require("../$throw");

module.exports = function ES6$THROW(string, offset) {
  let context = Token.next(string, offset);
  context = Token.next(string, context.offset);
  return { statement: $THROW(context.token), offset: context.offset };
};
