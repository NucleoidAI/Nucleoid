const Token = require("../token");
const $RETURN = require("../$return");
const ES6 = require("./es6");

module.exports = function ES6$RETURN(string, offset) {
  let context = Token.next(string, offset);

  if (context.token === "return") {
    context = ES6.compile(string, context.offset);
    return { statement: $RETURN(context.statements), offset: context.offset };
  } else {
    throw SyntaxError("Missing return keyword");
  }
};
