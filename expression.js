var state = require("./state"); // eslint-disable-line no-unused-vars
var graph = require("./graph");
var Local = require("./local");
var Value = require("./value");

module.exports = class EXPRESSION extends Value {
  constructor(tokens) {
    super();
    this.tokens = tokens;
  }

  run(scope) {
    let tokens = this.tokens.map(token => {
      if (token == "typeof") {
        return "typeof ";
      }

      let parts = token.split(".");
      let reference = Local.retrieve(scope, token);

      if (graph.node[parts[0]]) {
        return "state." + token;
      } else if (reference) {
        return reference;
      } else {
        return token;
      }
    });

    return eval(tokens.join(""));
  }

  graph() {
    return this.tokens.filter(token => {
      if (graph.node[token.split(".")[0]]) return true;
    });
  }
};
