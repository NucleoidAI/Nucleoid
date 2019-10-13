var state = require("./state"); // eslint-disable-line no-unused-vars
var graph = require("./graph");
var Local = require("./local");
var Node = require("./node");
var Value = require("./value");

module.exports = class EXPRESSION extends Value {
  run(scope, instance) {
    let tokens = this.tokens.map(token => {
      if (token == "typeof") {
        return "typeof ";
      }

      let parts = token.split(".");

      if (instance && parts[0] == instance.class.name) {
        parts[0] = instance.identifier();
        token = parts.join(".");
      }

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
      if (graph.node[token]) return true;
      else if (graph.node[token.split(".")[0]]) {
        graph.node[token] = new Node();
        return true;
      }
    });
  }
};
