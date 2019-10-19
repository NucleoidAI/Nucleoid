var state = require("./state"); // eslint-disable-line no-unused-vars
var graph = require("./graph");
var Local = require("./local");
var Value = require("./value");
var Identifier = require("./identifier");

module.exports = class EXPRESSION extends Value {
  constructor(tokens) {
    super();
    this.tokens = tokens;
  }

  prepare() {
    this.tokens = this.tokens.map(token => {
      let parts = Identifier.splitLast(token);
      if (parts[0] && parts[1] && parts[0] == "value" && graph[parts[1]])
        return JSON.stringify(eval("state." + parts[1]));
      else return token;
    });
  }

  run(scope) {
    let tokens = this.tokens
      .map(token => (token = Local.reference(scope, token)))
      .map(token => {
        if (token == "typeof") {
          return "typeof ";
        }

        let parts = token.split(".");
        let reference = Local.retrieve(scope, token);

        if (graph[parts[0]]) {
          return "state." + Identifier.reference(token);
        } else if (reference) {
          return reference;
        } else {
          return token;
        }
      });

    return eval(tokens.join(""));
  }

  graph(scope) {
    return this.tokens
      .map(token => Local.reference(scope, token))
      .filter(token => {
        if (graph[token.split(".")[0]]) return true;
      })
      .map(token => Identifier.reference(token));
  }
};
