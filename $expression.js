var Token = require("./token");
var EXPRESSION = require("./expression");

module.exports = function(string, offset) {
  let context = Token.each(string, offset, function(token) {
    if (token == "new") {
      return "new ";
    }

    if (token == "return") {
      return "return ";
    }

    return token;
  });

  let statement = new EXPRESSION();
  statement.tokens = context.tokens;

  return { statement: statement, offset: context.offset };
};
