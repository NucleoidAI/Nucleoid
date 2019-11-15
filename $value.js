var Token = require("./token");
var EXPRESSION = require("./expression");
var $ = require("./$");
var graph = require("./graph");
var REFERENCE = require("./reference");

module.exports = function(string, offset) {
  let context = Token.each(string, offset, function(token) {
    if (token === "new") {
      return "new ";
    }

    if (token === "return") {
      return "return ";
    }

    if (token === "typeof") {
      return "typeof ";
    }

    return token;
  });

  let statement = new $VALUE();
  statement.tokens = context.tokens;

  return { statement: statement, offset: context.offset };
};

class $VALUE extends $ {
  run() {
    if (this.tokens.length === 1 && graph[this.tokens[0]]) {
      let statement = new REFERENCE();
      statement.link = graph[this.tokens[0]];
      return statement;
    } else {
      let statement = new EXPRESSION();
      statement.tokens = this.tokens;
      return statement;
    }
  }
}
