var Token = require("./token");
var EXPRESSION = require("./expression");
var $ = require("./$");
var graph = require("./graph");
var REFERENCE = require("./reference");
var EXPRESSION$CLASS = require("./expression$class");

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

  if (context.tokens.length === 0) {
    return { offset: context.offset };
  }

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
      for (let token of this.tokens) {
        let prefix = token.split(".")[0];

        if (graph[prefix] && graph[prefix].instanceof === "CLASS") {
          let statement = new EXPRESSION$CLASS();
          statement.class = graph[prefix];
          statement.tokens = this.tokens;
          return statement;
        }
      }

      let statement = new EXPRESSION();
      statement.tokens = this.tokens;
      return statement;
    }
  }
}
