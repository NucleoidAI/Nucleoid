var state = require("./state"); // eslint-disable-line no-unused-vars
var graph = require("./graph");
var Local = require("./local");
var Value = require("./value");
var Identifier = require("./identifier");
var Node = require("./node");

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
    try {
      let tokens = this.tokens
        .map(token => (token = Local.reference(scope, token)))
        .map(token => {
          let parts = token.split(".");
          let reference = Local.retrieve(scope, token);

          if (reference) {
            let value = eval(reference);

            if (value == undefined || value == null) throw 0;
            return reference;
          } else if (graph[parts[0]]) {
            let reference = "state." + Identifier.reference(token);
            let value = eval(reference);

            if (value == undefined || value == null) throw 1;
            return reference;
          } else {
            return token;
          }
        });

      return eval(tokens.join(""));
    } catch (error) {
      if (error instanceof Error) throw error;
      return null;
    }
  }

  graph(scope) {
    return this.tokens
      .map(token => Local.reference(scope, token))
      .filter(token => {
        if (graph[token]) return true;
        else if (graph[token.split(".")[0]]) {
          graph[token] = new Node();
          return true;
        }
      })
      .map(token => Identifier.reference(token));
  }
};
