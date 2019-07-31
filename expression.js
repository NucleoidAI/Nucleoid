var state = require("./state"); // eslint-disable-line no-unused-vars
var graph = require("./graph");
var Local = require("./local");

module.exports = class EXPRESSION {
  run(scope, instance) {
    let tokens = this.tokens.map(token => {
      let parts = token.split(".");

      if (instance && parts[0] == instance.class.name) {
        parts[0] = instance.variable;
        token = parts.join(".");
      }

      let value = Local.retrieve(scope, token);

      if (graph.node[parts[0]]) {
        return "state." + token;
      } else if (value) {
        if (typeof value == "string") {
          return '"' + value + '"';
        } else {
          return value;
        }
      } else {
        return token;
      }
    });

    return eval(tokens.join(""));
  }
};
