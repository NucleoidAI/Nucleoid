var state = require("./state"); // eslint-disable-line no-unused-vars
var graph = require("./graph");

module.exports = class EXPRESSION {
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
};
