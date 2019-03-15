var state = require("./state"); // eslint-disable-line no-unused-vars
var graph = require("./graph");
var Token = require("./token");

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

class EXPRESSION {
  run() {
    let tokens = this.tokens;

    for (let i = 0; i < tokens.length; i++) {
      let token = tokens[i];

      if (graph.node[token.split(".")[0]]) {
        tokens[i] = "state." + token;
      }
    }

    return eval(tokens.join(""));
  }
}
module.exports.EXPRESSION = EXPRESSION;
