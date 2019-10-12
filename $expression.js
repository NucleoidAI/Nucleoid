var Token = require("./token");
var EXPRESSION = require("./expression");

module.exports = function(string, offset) {
  let context = Token.each(string, offset, function(token) {
    if (token == "new") {
      return "new ";
    }

    return token;
  });

  let statement = new EXPRESSION();
  statement.tokens = context.tokens;

  return { statement: statement, offset: context.offset };
};
